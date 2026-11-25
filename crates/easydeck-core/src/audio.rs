//! The default output device's volume and mute (G-4). Pure logic behind a trait: the Windows
//! code lives in `easydeck-platform-windows`, and tests use [`FakeAudioEndpoint`].

use serde::Serialize;
use std::fmt;
use std::sync::Mutex;

/// What the volume tile shows. `level` is 0 to 100.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct VolumeState {
    pub level: u8,
    pub muted: bool,
}

/// The endpoint could not be read or changed (no output device, device removed, COM error).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AudioError(pub String);

impl fmt::Display for AudioError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "audio endpoint error: {}", self.0)
    }
}

impl std::error::Error for AudioError {}

/// The default output (render) endpoint. Every call reads the current default device, so a
/// device change is picked up on the next call.
pub trait AudioEndpoint: Send + Sync {
    fn get(&self) -> Result<VolumeState, AudioError>;
    /// Sets the master level. Values above 100 are clamped to 100.
    fn set_level(&self, level: u8) -> Result<(), AudioError>;
    fn set_muted(&self, muted: bool) -> Result<(), AudioError>;
}

/// Maps a scalar 0.0..=1.0 to 0..=100, rounded and clamped. NaN gives 0.
pub fn scalar_to_level(scalar: f32) -> u8 {
    if scalar.is_nan() {
        return 0;
    }
    (scalar.clamp(0.0, 1.0) * 100.0).round() as u8
}

/// Maps a level 0..=100 (clamped) to a scalar 0.0..=1.0.
pub fn level_to_scalar(level: u8) -> f32 {
    f32::from(level.min(100)) / 100.0
}

/// Poll helper: reads the endpoint and returns the state only when it differs from `last`
/// (the first successful read always counts as a change). On error `last` is cleared, so the
/// next good read is emitted again, and the error is returned.
pub fn poll_change(
    endpoint: &dyn AudioEndpoint,
    last: &mut Option<VolumeState>,
) -> Result<Option<VolumeState>, AudioError> {
    match endpoint.get() {
        Ok(now) => {
            if *last == Some(now) {
                Ok(None)
            } else {
                *last = Some(now);
                Ok(Some(now))
            }
        }
        Err(e) => {
            *last = None;
            Err(e)
        }
    }
}

/// An in-memory endpoint for tests. It never touches the real system volume.
pub struct FakeAudioEndpoint {
    state: Mutex<Result<VolumeState, AudioError>>,
}

impl FakeAudioEndpoint {
    pub fn new(level: u8, muted: bool) -> Self {
        Self {
            state: Mutex::new(Ok(VolumeState {
                level: level.min(100),
                muted,
            })),
        }
    }

    /// An endpoint with no device: every call fails.
    pub fn unavailable() -> Self {
        Self {
            state: Mutex::new(Err(AudioError("no output device".into()))),
        }
    }

    fn with<T>(
        &self,
        f: impl FnOnce(&mut VolumeState) -> T,
    ) -> Result<T, AudioError> {
        let mut guard = self.state.lock().unwrap_or_else(|e| e.into_inner());
        match &mut *guard {
            Ok(s) => Ok(f(s)),
            Err(e) => Err(e.clone()),
        }
    }
}

impl AudioEndpoint for FakeAudioEndpoint {
    fn get(&self) -> Result<VolumeState, AudioError> {
        self.with(|s| *s)
    }
    fn set_level(&self, level: u8) -> Result<(), AudioError> {
        self.with(|s| s.level = level.min(100))
    }
    fn set_muted(&self, muted: bool) -> Result<(), AudioError> {
        self.with(|s| s.muted = muted)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn set_and_get_round_trip() {
        let e = FakeAudioEndpoint::new(30, false);
        e.set_level(75).unwrap();
        e.set_muted(true).unwrap();
        assert_eq!(e.get().unwrap(), VolumeState { level: 75, muted: true });
    }

    #[test]
    fn level_is_clamped_to_100() {
        let e = FakeAudioEndpoint::new(0, false);
        e.set_level(250).unwrap();
        assert_eq!(e.get().unwrap().level, 100);
    }

    #[test]
    fn unavailable_endpoint_fails_every_call() {
        let e = FakeAudioEndpoint::unavailable();
        assert!(e.get().is_err());
        assert!(e.set_level(10).is_err());
        assert!(e.set_muted(true).is_err());
    }

    #[test]
    fn scalar_mapping_rounds_and_clamps() {
        assert_eq!(scalar_to_level(0.0), 0);
        assert_eq!(scalar_to_level(0.505), 51);
        assert_eq!(scalar_to_level(1.0), 100);
        assert_eq!(scalar_to_level(1.7), 100);
        assert_eq!(scalar_to_level(-0.2), 0);
        assert_eq!(scalar_to_level(f32::NAN), 0);
        assert_eq!(level_to_scalar(50), 0.5);
        assert_eq!(level_to_scalar(200), 1.0);
        for l in 0..=100u8 {
            assert_eq!(scalar_to_level(level_to_scalar(l)), l);
        }
    }

    #[test]
    fn poll_change_reports_only_changes() {
        let e = FakeAudioEndpoint::new(40, false);
        let mut last = None;
        assert_eq!(
            poll_change(&e, &mut last).unwrap(),
            Some(VolumeState { level: 40, muted: false })
        );
        assert_eq!(poll_change(&e, &mut last).unwrap(), None);
        e.set_muted(true).unwrap(); // an outside change
        assert_eq!(
            poll_change(&e, &mut last).unwrap(),
            Some(VolumeState { level: 40, muted: true })
        );
        assert_eq!(poll_change(&e, &mut last).unwrap(), None);
    }

    #[test]
    fn poll_change_emits_again_after_an_error() {
        let ok = FakeAudioEndpoint::new(40, false);
        let bad = FakeAudioEndpoint::unavailable();
        let mut last = None;
        poll_change(&ok, &mut last).unwrap();
        assert!(poll_change(&bad, &mut last).is_err());
        assert!(poll_change(&ok, &mut last).unwrap().is_some());
    }
}
