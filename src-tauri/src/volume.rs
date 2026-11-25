//! Volume tile backend (G-4): three commands plus a poller that emits the Tauri event
//! `volume` with `{"level":0-100,"muted":bool}` when the value changes (also when Windows or
//! another app changes it). Polling every 500 ms was chosen over `IAudioEndpointVolumeCallback`
//! because it is simpler and also follows a default-device change. Not a risk item.

use crate::error::AppError;
use easydeck_core::audio::{poll_change, AudioEndpoint, AudioError, VolumeState};
use easydeck_core::deck::{Deck, WidgetKind};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tauri::{AppHandle, Emitter};

const POLL_EVERY: Duration = Duration::from_millis(500);

/// The audio endpoint, shared by the commands and the poller. Managed by Tauri.
pub struct VolumeService {
    endpoint: Arc<dyn AudioEndpoint>,
    /// The stop flag of the current poller; each `start_poller` installs a fresh one.
    stop: Mutex<Arc<AtomicBool>>,
}

impl VolumeService {
    pub fn new(endpoint: Arc<dyn AudioEndpoint>) -> Self {
        Self {
            endpoint,
            stop: Mutex::new(Arc::new(AtomicBool::new(false))),
        }
    }

    /// The real endpoint on Windows; elsewhere an endpoint that always reports "no device".
    pub fn system() -> Self {
        #[cfg(windows)]
        let endpoint: Arc<dyn AudioEndpoint> =
            Arc::new(easydeck_platform_windows::audio::WindowsAudioEndpoint::new());
        #[cfg(not(windows))]
        let endpoint: Arc<dyn AudioEndpoint> =
            Arc::new(easydeck_core::audio::FakeAudioEndpoint::unavailable());
        Self::new(endpoint)
    }

    /// Starts the poller on its own thread (it keeps one COM apartment for its lifetime).
    ///
    /// Starts only when `deck` has a `volume` widget, so a deck without one costs nothing.
    /// The deck is fixed at startup today; a deck reload must call `stop` and then this again.
    pub fn start_poller(&self, app: AppHandle, deck: &Deck) {
        let has_volume = deck
            .pages
            .iter()
            .flat_map(|p| &p.widgets)
            .any(|w| matches!(w.kind, WidgetKind::Volume {}));
        if !has_volume {
            return;
        }
        let stop = Arc::new(AtomicBool::new(false));
        // Stop any earlier poller, then give this one its own flag.
        let old = std::mem::replace(&mut *self.stop.lock().unwrap(), stop.clone());
        old.store(true, Ordering::Relaxed);
        let endpoint = self.endpoint.clone();
        std::thread::spawn(move || {
            let mut last = None;
            while !stop.load(Ordering::Relaxed) {
                // An error means "no data": the UI keeps its last value or shows no-data
                // after its own `volume_get` fails. The next good read is emitted again.
                if let Ok(Some(now)) = poll_change(endpoint.as_ref(), &mut last) {
                    if app.emit("volume", now).is_err() {
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

fn io(e: AudioError) -> AppError {
    AppError::Io {
        message: "the volume is not available".into(),
        detail: Some(e.0),
    }
}

pub fn get_impl(ep: &dyn AudioEndpoint) -> Result<VolumeState, AppError> {
    ep.get().map_err(io)
}

pub fn set_impl(ep: &dyn AudioEndpoint, level: i64) -> Result<(), AppError> {
    if !(0..=100).contains(&level) {
        return Err(AppError::Invalid {
            message: "volume level must be from 0 to 100".into(),
            detail: Some(level.to_string()),
        });
    }
    ep.set_level(level as u8).map_err(io)
}

pub fn mute_impl(ep: &dyn AudioEndpoint, muted: bool) -> Result<(), AppError> {
    ep.set_muted(muted).map_err(io)
}

/// Runs a blocking audio call off the async workers.
async fn blocking<T: Send + 'static>(
    state: &tauri::State<'_, VolumeService>,
    f: impl FnOnce(&dyn AudioEndpoint) -> Result<T, AppError> + Send + 'static,
) -> Result<T, AppError> {
    let ep = state.endpoint.clone();
    tauri::async_runtime::spawn_blocking(move || f(ep.as_ref()))
        .await
        .map_err(|e| AppError::Internal {
            message: "the volume call did not finish".into(),
            detail: Some(e.to_string()),
        })?
}

#[tauri::command]
pub async fn volume_get(state: tauri::State<'_, VolumeService>) -> Result<VolumeState, AppError> {
    blocking(&state, get_impl).await
}

#[tauri::command]
pub async fn volume_set(
    state: tauri::State<'_, VolumeService>,
    level: i64,
) -> Result<(), AppError> {
    blocking(&state, move |ep| set_impl(ep, level)).await
}

#[tauri::command]
pub async fn volume_mute(
    state: tauri::State<'_, VolumeService>,
    muted: bool,
) -> Result<(), AppError> {
    blocking(&state, move |ep| mute_impl(ep, muted)).await
}

#[cfg(test)]
mod tests {
    use super::*;
    use easydeck_core::audio::FakeAudioEndpoint;

    #[test]
    fn get_set_mute_use_the_endpoint() {
        let ep = FakeAudioEndpoint::new(20, false);
        set_impl(&ep, 65).unwrap();
        mute_impl(&ep, true).unwrap();
        assert_eq!(
            get_impl(&ep).unwrap(),
            VolumeState {
                level: 65,
                muted: true
            }
        );
    }

    #[test]
    fn set_rejects_out_of_range_levels() {
        let ep = FakeAudioEndpoint::new(20, false);
        for bad in [-1, 101, 5000] {
            assert_eq!(set_impl(&ep, bad).unwrap_err().code(), "invalid");
        }
        assert_eq!(get_impl(&ep).unwrap().level, 20);
    }

    #[test]
    fn no_device_is_a_retryable_io_error() {
        let ep = FakeAudioEndpoint::unavailable();
        let err = get_impl(&ep).unwrap_err();
        assert_eq!(err.code(), "io");
        assert!(err.retryable());
        assert!(set_impl(&ep, 10).is_err());
        assert!(mute_impl(&ep, true).is_err());
    }
}
