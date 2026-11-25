//! Media tile backend (R2-3): two commands plus a poller that emits the Tauri event `media`
//! with the `media_get` payload when the track, the playback status or the available
//! controls change. Polling every 1000 ms was chosen over the session manager's change events
//! for the same reason as the volume poller: it is simpler, and it also follows a change of
//! the current session. The thumbnail is read once per track change inside the session, so a
//! poll costs no image read. Not a risk item: no network call, no account.

use crate::error::AppError;
use easydeck_core::deck::{Deck, WidgetKind};
use easydeck_core::media::{poll_change, MediaCommand, MediaError, MediaSession, MediaState};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tauri::{AppHandle, Emitter};

const POLL_EVERY: Duration = Duration::from_millis(1000);

/// The media session, shared by the commands and the poller. Managed by Tauri.
pub struct MediaService {
    session: Arc<dyn MediaSession>,
    /// The stop flag of the current poller; each `start_poller` installs a fresh one.
    stop: Mutex<Arc<AtomicBool>>,
}

impl MediaService {
    pub fn new(session: Arc<dyn MediaSession>) -> Self {
        Self {
            session,
            stop: Mutex::new(Arc::new(AtomicBool::new(false))),
        }
    }

    /// The real session on Windows; elsewhere a session with no player.
    pub fn system() -> Self {
        #[cfg(windows)]
        let session: Arc<dyn MediaSession> =
            Arc::new(easydeck_platform_windows::media::WindowsMediaSession::new());
        #[cfg(not(windows))]
        let session: Arc<dyn MediaSession> =
            Arc::new(easydeck_core::media::FakeMediaSession::no_player());
        Self::new(session)
    }

    /// Starts the poller on its own thread (it keeps one COM apartment for its lifetime).
    ///
    /// Starts only when `deck` has a `media` widget, so a deck without one costs nothing.
    /// The deck is fixed at startup today; a deck reload must call `stop` and then this again.
    pub fn start_poller(&self, app: AppHandle, deck: &Deck) {
        let has_media = deck
            .pages
            .iter()
            .flat_map(|p| &p.widgets)
            .any(|w| matches!(w.kind, WidgetKind::Media { .. }));
        if !has_media {
            return;
        }
        let stop = Arc::new(AtomicBool::new(false));
        let old = std::mem::replace(&mut *self.stop.lock().unwrap(), stop.clone());
        old.store(true, Ordering::Relaxed);
        let session = self.session.clone();
        std::thread::spawn(move || {
            let mut last = None;
            while !stop.load(Ordering::Relaxed) {
                // An error means "no data": the UI keeps its last state. The next good read
                // is emitted again.
                if let Ok(Some(now)) = poll_change(session.as_ref(), &mut last) {
                    if app.emit("media", now).is_err() {
                        break;
                    }
                }
                std::thread::sleep(POLL_EVERY);
            }
        });
    }

    pub fn stop(&self) {
        self.stop.lock().unwrap().store(true, Ordering::Relaxed);
    }
}

fn io(e: MediaError) -> AppError {
    AppError::Io {
        message: "the media session is not available".into(),
        detail: Some(e.0),
    }
}

pub fn get_impl(s: &dyn MediaSession) -> Result<MediaState, AppError> {
    s.get().map_err(io)
}

pub fn command_impl(s: &dyn MediaSession, command: MediaCommand) -> Result<(), AppError> {
    s.send(command).map_err(io)
}

/// Runs a blocking session call off the async workers (the WinRT calls wait for the player).
async fn blocking<T: Send + 'static>(
    state: &tauri::State<'_, MediaService>,
    f: impl FnOnce(&dyn MediaSession) -> Result<T, AppError> + Send + 'static,
) -> Result<T, AppError> {
    let s = state.session.clone();
    tauri::async_runtime::spawn_blocking(move || f(s.as_ref()))
        .await
        .map_err(|e| AppError::Internal {
            message: "the media call did not finish".into(),
            detail: Some(e.to_string()),
        })?
}

#[tauri::command]
pub async fn media_get(state: tauri::State<'_, MediaService>) -> Result<MediaState, AppError> {
    blocking(&state, get_impl).await
}

/// `command` is `play_pause`, `next` or `previous`; any other value is rejected by the
/// argument parser before this runs.
#[tauri::command]
pub async fn media_command(
    state: tauri::State<'_, MediaService>,
    command: MediaCommand,
) -> Result<(), AppError> {
    blocking(&state, move |s| command_impl(s, command)).await
}

#[cfg(test)]
mod tests {
    use super::*;
    use easydeck_core::media::{FakeMediaSession, MediaStatus};

    fn playing() -> MediaState {
        MediaState {
            title: "Song".into(),
            artist: "Band".into(),
            status: MediaStatus::Playing,
            can_play_pause: true,
            can_next: true,
            can_previous: true,
            thumbnail: None,
        }
    }

    #[test]
    fn get_and_command_use_the_session() {
        let s = FakeMediaSession::new(playing());
        assert_eq!(get_impl(&s).unwrap().title, "Song");
        command_impl(&s, MediaCommand::Next).unwrap();
        assert_eq!(s.sent(), vec![MediaCommand::Next]);
    }

    #[test]
    fn no_player_is_neutral_and_commands_fail() {
        let s = FakeMediaSession::no_player();
        assert_eq!(get_impl(&s).unwrap().status, MediaStatus::None);
        assert_eq!(
            command_impl(&s, MediaCommand::PlayPause)
                .unwrap_err()
                .code(),
            "io"
        );
    }

    #[test]
    fn unavailable_session_is_a_retryable_io_error() {
        let err = get_impl(&FakeMediaSession::unavailable()).unwrap_err();
        assert_eq!(err.code(), "io");
        assert!(err.retryable());
    }
}
