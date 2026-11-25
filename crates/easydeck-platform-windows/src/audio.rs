//! [`AudioEndpoint`] for Windows: `IAudioEndpointVolume` on the default output device.
//!
//! Every call looks up the current default render endpoint, so a default-device change is
//! picked up on the next call. COM is initialised once per calling thread (MTA); run the
//! calls from a blocking thread, not from an async worker.

use easydeck_core::audio::{
    level_to_scalar, scalar_to_level, AudioEndpoint, AudioError, VolumeState,
};
use std::ptr;
use windows::Win32::Media::Audio::Endpoints::IAudioEndpointVolume;
use windows::Win32::Media::Audio::{eConsole, eRender, IMMDeviceEnumerator, MMDeviceEnumerator};
use windows::Win32::System::Com::{
    CoCreateInstance, CoInitializeEx, CLSCTX_ALL, COINIT_MULTITHREADED,
};

fn err(e: windows::core::Error) -> AudioError {
    AudioError(e.message())
}

thread_local! {
    /// Result of the one COM init on this thread.
    static COM_READY: Result<(), AudioError> = {
        // SAFETY: plain COM init with no reserved pointer. The thread keeps COM until it
        // exits; we never uninitialise, so no COM object outlives it. S_FALSE (already
        // initialised) and RPC_E_CHANGED_MODE (thread is STA) both leave COM usable and are
        // `Ok` or ignored below.
        let hr = unsafe { CoInitializeEx(None, COINIT_MULTITHREADED) };
        if hr.is_ok() || hr == windows::Win32::Foundation::RPC_E_CHANGED_MODE {
            Ok(())
        } else {
            Err(AudioError(format!("COM init failed: {hr:?}")))
        }
    };
}

/// The default output device's volume. Holds no COM pointers, so it is `Send + Sync`.
#[derive(Debug, Default, Clone, Copy)]
pub struct WindowsAudioEndpoint;

impl WindowsAudioEndpoint {
    pub fn new() -> Self {
        Self
    }

    fn volume() -> Result<IAudioEndpointVolume, AudioError> {
        COM_READY.with(|r| r.clone())?;
        // SAFETY: COM is initialised on this thread (above). The COM calls take no raw
        // pointers from us except `None`, and the returned interfaces are owned wrappers.
        unsafe {
            let enumerator: IMMDeviceEnumerator =
                CoCreateInstance(&MMDeviceEnumerator, None, CLSCTX_ALL).map_err(err)?;
            let device = enumerator
                .GetDefaultAudioEndpoint(eRender, eConsole)
                .map_err(err)?;
            device
                .Activate::<IAudioEndpointVolume>(CLSCTX_ALL, None)
                .map_err(err)
        }
    }
}

impl AudioEndpoint for WindowsAudioEndpoint {
    fn get(&self) -> Result<VolumeState, AudioError> {
        let v = Self::volume()?;
        // SAFETY: `v` is a live interface; both calls only return values.
        unsafe {
            let level = scalar_to_level(v.GetMasterVolumeLevelScalar().map_err(err)?);
            let muted = v.GetMute().map_err(err)?.as_bool();
            Ok(VolumeState { level, muted })
        }
    }

    fn set_level(&self, level: u8) -> Result<(), AudioError> {
        let v = Self::volume()?;
        // SAFETY: `v` is live; a null event-context GUID pointer is allowed.
        unsafe {
            v.SetMasterVolumeLevelScalar(level_to_scalar(level), ptr::null())
                .map_err(err)
        }
    }

    fn set_muted(&self, muted: bool) -> Result<(), AudioError> {
        let v = Self::volume()?;
        // SAFETY: `v` is live; a null event-context GUID pointer is allowed.
        unsafe { v.SetMute(muted, ptr::null()).map_err(err) }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Changes the real system volume and mute, then restores them. Runs only with
    /// `EASYDECK_PROBE_TESTS=1`; a plain `cargo test` never touches the system volume.
    #[test]
    fn set_level_and_mute_round_trip_on_real_device() {
        if std::env::var("EASYDECK_PROBE_TESTS").as_deref() != Ok("1") {
            return;
        }
        let ep = WindowsAudioEndpoint::new();
        let original = ep.get().expect("an output device is required");
        let target = if original.level == 37 { 38 } else { 37 };
        // Restores the original level and mute on drop, so a failed assert (a panic) cannot
        // leave the machine changed.
        struct Restore<'a> {
            ep: &'a WindowsAudioEndpoint,
            original: VolumeState,
        }
        impl Drop for Restore<'_> {
            fn drop(&mut self) {
                let _ = self.ep.set_level(self.original.level);
                let _ = self.ep.set_muted(self.original.muted);
            }
        }
        let _restore = Restore { ep: &ep, original };
        ep.set_level(target).unwrap();
        ep.set_muted(!original.muted).unwrap();
        let now = ep.get().unwrap();
        assert_eq!(now.level, target);
        assert_eq!(now.muted, !original.muted);
    }
}
