//! [`MediaSession`] for Windows: `GlobalSystemMediaTransportControlsSessionManager`.
//!
//! Follows the session Windows calls current, so any player that reports to the system media
//! overlay works. No account, no token, no network call. The WinRT calls block with `.join()`;
//! run them from a blocking thread, not from an async worker. COM is initialised once per
//! calling thread (MTA).

use easydeck_core::deck::MediaControl;
use easydeck_core::media::{
    thumbnail_data_url, MediaError, MediaSession, MediaState, MediaStatus, MAX_THUMBNAIL_BYTES,
};
use std::cell::RefCell;
use std::sync::Mutex;
use windows::Media::Control::{
    GlobalSystemMediaTransportControlsSession, GlobalSystemMediaTransportControlsSessionManager,
    GlobalSystemMediaTransportControlsSessionPlaybackStatus as Playback,
};
use windows::Storage::Streams::DataReader;
use windows::Win32::System::Com::{CoInitializeEx, COINIT_MULTITHREADED};

fn err(e: windows::core::Error) -> MediaError {
    MediaError(e.message())
}

thread_local! {
    /// Result of the one COM init on this thread.
    static COM_READY: Result<(), MediaError> = {
        // SAFETY: plain COM init with no reserved pointer. The thread keeps COM until it
        // exits; we never uninitialise. S_FALSE (already initialised) is `Ok`; a thread that
        // is already an STA (RPC_E_CHANGED_MODE) can still use WinRT, so it is ignored.
        let hr = unsafe { CoInitializeEx(None, COINIT_MULTITHREADED) };
        if hr.is_ok() || hr == windows::Win32::Foundation::RPC_E_CHANGED_MODE {
            Ok(())
        } else {
            Err(MediaError(format!("COM init failed: {hr:?}")))
        }
    };
}

thread_local! {
    /// The session manager of this thread, built once. Dropped on an error so the next call
    /// asks for a new one.
    static MANAGER: RefCell<Option<GlobalSystemMediaTransportControlsSessionManager>> =
        const { RefCell::new(None) };
}

/// A track's thumbnail is read on this many polls after the track changes. The first reads may
/// fail or still return the previous track's art; a later good read replaces the cached one.
const THUMB_TRIES: u8 = 5;

#[derive(Default)]
struct ThumbCache {
    /// Title, artist and source app id of the track the cache belongs to.
    key: (String, String, String),
    thumb: Option<String>,
    tries: u8,
}

/// The current system media session. Holds no WinRT pointers, so it is `Send + Sync`. The
/// thumbnail is read once per track change and kept, so a poll costs no image read.
#[derive(Default)]
pub struct WindowsMediaSession {
    cache: Mutex<ThumbCache>,
}

impl WindowsMediaSession {
    pub fn new() -> Self {
        Self::default()
    }

    fn manager() -> Result<GlobalSystemMediaTransportControlsSessionManager, MediaError> {
        COM_READY.with(|r| r.clone())?;
        MANAGER.with(|m| {
            let mut m = m.borrow_mut();
            if let Some(manager) = m.as_ref() {
                return Ok(manager.clone());
            }
            let manager = GlobalSystemMediaTransportControlsSessionManager::RequestAsync()
                .map_err(err)?
                .join()
                .map_err(err)?;
            *m = Some(manager.clone());
            Ok(manager)
        })
    }

    fn current() -> Result<Option<GlobalSystemMediaTransportControlsSession>, MediaError> {
        let manager = Self::manager()?;
        // No current session is an error from `GetCurrentSession` (a null result).
        Ok(manager.GetCurrentSession().ok())
    }

    /// Forget the cached manager after a failed call, so the next call builds a new one.
    fn reset_manager<T>(r: Result<T, MediaError>) -> Result<T, MediaError> {
        if r.is_err() {
            MANAGER.with(|m| *m.borrow_mut() = None);
        }
        r
    }

    fn thumbnail(
        &self,
        key: (String, String, String),
        props: &windows::Media::Control::GlobalSystemMediaTransportControlsSessionMediaProperties,
    ) -> Option<String> {
        let mut c = self.cache.lock().unwrap_or_else(|e| e.into_inner());
        if c.key != key {
            *c = ThumbCache {
                key,
                ..ThumbCache::default()
            };
        }
        if c.tries < THUMB_TRIES {
            c.tries += 1;
            if let Some(t) = read_thumbnail(props) {
                c.thumb = Some(t);
            }
        }
        c.thumb.clone()
    }
}

fn read_thumbnail(
    props: &windows::Media::Control::GlobalSystemMediaTransportControlsSessionMediaProperties,
) -> Option<String> {
    let stream = props.Thumbnail().ok()?.OpenReadAsync().ok()?.join().ok()?;
    let size = stream.Size().ok()?;
    if size == 0 || size > MAX_THUMBNAIL_BYTES as u64 {
        return None;
    }
    let reader = DataReader::CreateDataReader(&stream).ok()?;
    let loaded = reader.LoadAsync(size as u32).ok()?.join().ok()?;
    let mut buf = vec![0u8; loaded as usize];
    reader.ReadBytes(&mut buf).ok()?;
    thumbnail_data_url(&buf)
}

impl MediaSession for WindowsMediaSession {
    fn get(&self) -> Result<MediaState, MediaError> {
        Self::reset_manager(self.read())
    }

    fn send(&self, command: MediaControl) -> Result<(), MediaError> {
        Self::reset_manager(Self::send_command(command))
    }
}

impl WindowsMediaSession {
    fn read(&self) -> Result<MediaState, MediaError> {
        let Some(session) = Self::current()? else {
            return Ok(MediaState::none());
        };
        let props = session
            .TryGetMediaPropertiesAsync()
            .map_err(err)?
            .join()
            .map_err(err)?;
        let title = props.Title().map(|t| t.to_string()).unwrap_or_default();
        let artist = props.Artist().map(|t| t.to_string()).unwrap_or_default();
        let info = session.GetPlaybackInfo().map_err(err)?;
        let status = match info.PlaybackStatus() {
            Ok(Playback::Playing) => MediaStatus::Playing,
            Ok(Playback::Paused) => MediaStatus::Paused,
            _ => MediaStatus::Stopped,
        };
        let c = info.Controls().map_err(err)?;
        let on = |r: windows::core::Result<bool>| r.unwrap_or(false);
        let can_play_pause =
            on(c.IsPlayPauseToggleEnabled()) || on(c.IsPlayEnabled()) || on(c.IsPauseEnabled());
        let app = session
            .SourceAppUserModelId()
            .map(|t| t.to_string())
            .unwrap_or_default();
        let thumbnail = self.thumbnail((title.clone(), artist.clone(), app), &props);
        Ok(MediaState {
            title,
            artist,
            status,
            can_play_pause,
            can_next: on(c.IsNextEnabled()),
            can_previous: on(c.IsPreviousEnabled()),
            thumbnail,
        })
    }

    fn send_command(command: MediaControl) -> Result<(), MediaError> {
        let session = Self::current()?.ok_or_else(|| MediaError("no player is running".into()))?;
        let accepted = match command {
            MediaControl::PlayPause => session.TryTogglePlayPauseAsync(),
            MediaControl::Next => session.TrySkipNextAsync(),
            MediaControl::Previous => session.TrySkipPreviousAsync(),
        }
        .map_err(err)?
        .join()
        .map_err(err)?;
        if accepted {
            Ok(())
        } else {
            Err(MediaError("the player did not accept the command".into()))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Reads the real session (read-only). Gated like the other real-device tests: it needs
    /// the WinRT media manager, which a CI machine may not have. Runs only with
    /// `EASYDECK_PROBE_TESTS=1`.
    #[test]
    fn get_reads_the_real_session() {
        if std::env::var("EASYDECK_PROBE_TESTS").as_deref() != Ok("1") {
            return;
        }
        let state = WindowsMediaSession::new().get().expect("session manager");
        if state.status == MediaStatus::None {
            assert!(state.title.is_empty() && !state.can_play_pause);
        }
    }
}
