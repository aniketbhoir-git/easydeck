//! The system media session (R2-3): what is playing and the three transport commands. Pure
//! logic behind a trait: the Windows code lives in `easydeck-platform-windows`, and tests use
//! [`FakeMediaSession`]. No account, no token, no network call.

use crate::deck::MediaControl;
use serde::Serialize;
use std::fmt;
use std::sync::Mutex;

/// Largest thumbnail we pass to the UI, in bytes. A bigger image is dropped.
pub const MAX_THUMBNAIL_BYTES: usize = 256 * 1024;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum MediaStatus {
    Playing,
    Paused,
    Stopped,
    /// No player reports to Windows.
    None,
}

/// What the media tile shows. `thumbnail` is a `data:image/...;base64,` URL.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MediaState {
    pub title: String,
    pub artist: String,
    pub status: MediaStatus,
    pub can_play_pause: bool,
    pub can_next: bool,
    pub can_previous: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub thumbnail: Option<String>,
}

impl MediaState {
    /// The neutral state: no player.
    pub fn none() -> Self {
        Self {
            title: String::new(),
            artist: String::new(),
            status: MediaStatus::None,
            can_play_pause: false,
            can_next: false,
            can_previous: false,
            thumbnail: None,
        }
    }
}

/// The session could not be read or the command was not sent.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MediaError(pub String);

impl fmt::Display for MediaError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "media session error: {}", self.0)
    }
}

impl std::error::Error for MediaError {}

/// The current system media session. Every call follows the session Windows calls current.
pub trait MediaSession: Send + Sync {
    /// The state of the current session; [`MediaState::none`] when no player runs.
    fn get(&self) -> Result<MediaState, MediaError>;
    fn send(&self, command: MediaControl) -> Result<(), MediaError>;
}

/// Poll helper: reads the session and returns the state only when it differs from `last`.
/// On error `last` is cleared, so the next good read is emitted again.
pub fn poll_change(
    session: &dyn MediaSession,
    last: &mut Option<MediaState>,
) -> Result<Option<MediaState>, MediaError> {
    match session.get() {
        Ok(now) => {
            if last.as_ref() == Some(&now) {
                Ok(None)
            } else {
                *last = Some(now.clone());
                Ok(Some(now))
            }
        }
        Err(e) => {
            *last = None;
            Err(e)
        }
    }
}

const B64: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

/// Standard base64 with padding. Std only, to keep the dependency list as SPEC §2 has it.
pub fn base64_encode(bytes: &[u8]) -> String {
    let mut out = String::with_capacity(bytes.len().div_ceil(3) * 4);
    for chunk in bytes.chunks(3) {
        let n = (u32::from(chunk[0]) << 16)
            | (u32::from(*chunk.get(1).unwrap_or(&0)) << 8)
            | u32::from(*chunk.get(2).unwrap_or(&0));
        out.push(B64[(n >> 18) as usize & 63] as char);
        out.push(B64[(n >> 12) as usize & 63] as char);
        out.push(if chunk.len() > 1 {
            B64[(n >> 6) as usize & 63] as char
        } else {
            '='
        });
        out.push(if chunk.len() > 2 {
            B64[n as usize & 63] as char
        } else {
            '='
        });
    }
    out
}

/// The image type from the file's first bytes (png, jpeg, gif, webp, bmp), or `None`.
pub fn sniff_image_type(bytes: &[u8]) -> Option<&'static str> {
    if bytes.starts_with(b"\x89PNG\r\n\x1a\n") {
        Some("image/png")
    } else if bytes.starts_with(&[0xff, 0xd8, 0xff]) {
        Some("image/jpeg")
    } else if bytes.starts_with(b"GIF87a") || bytes.starts_with(b"GIF89a") {
        Some("image/gif")
    } else if bytes.len() >= 12 && &bytes[..4] == b"RIFF" && &bytes[8..12] == b"WEBP" {
        Some("image/webp")
    } else if bytes.starts_with(b"BM") {
        Some("image/bmp")
    } else {
        None
    }
}

/// A `data:` URL for a thumbnail. The type comes from the bytes, not from what the player
/// claims; `None` when the bytes are not one of the five image types, or the image is empty
/// or larger than [`MAX_THUMBNAIL_BYTES`].
pub fn thumbnail_data_url(bytes: &[u8]) -> Option<String> {
    if bytes.len() > MAX_THUMBNAIL_BYTES {
        return None;
    }
    let ct = sniff_image_type(bytes)?;
    Some(format!("data:{ct};base64,{}", base64_encode(bytes)))
}

/// An in-memory session for tests. It never touches a real player.
pub struct FakeMediaSession {
    state: Mutex<Result<MediaState, MediaError>>,
    sent: Mutex<Vec<MediaControl>>,
}

impl FakeMediaSession {
    pub fn new(state: MediaState) -> Self {
        Self {
            state: Mutex::new(Ok(state)),
            sent: Mutex::new(Vec::new()),
        }
    }

    /// No player running.
    pub fn no_player() -> Self {
        Self::new(MediaState::none())
    }

    /// A session whose every call fails.
    pub fn unavailable() -> Self {
        Self {
            state: Mutex::new(Err(MediaError("no media session manager".into()))),
            sent: Mutex::new(Vec::new()),
        }
    }

    pub fn set_state(&self, state: MediaState) {
        *self.state.lock().unwrap_or_else(|e| e.into_inner()) = Ok(state);
    }

    /// The commands sent so far, in order.
    pub fn sent(&self) -> Vec<MediaControl> {
        self.sent.lock().unwrap_or_else(|e| e.into_inner()).clone()
    }
}

impl MediaSession for FakeMediaSession {
    fn get(&self) -> Result<MediaState, MediaError> {
        self.state.lock().unwrap_or_else(|e| e.into_inner()).clone()
    }

    fn send(&self, command: MediaControl) -> Result<(), MediaError> {
        let state = self.get()?;
        let allowed = match command {
            MediaControl::PlayPause => state.can_play_pause,
            MediaControl::Next => state.can_next,
            MediaControl::Previous => state.can_previous,
        };
        if !allowed {
            return Err(MediaError("the session cannot do that".into()));
        }
        self.sent
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .push(command);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn playing() -> MediaState {
        MediaState {
            title: "Song".into(),
            artist: "Band".into(),
            status: MediaStatus::Playing,
            can_play_pause: true,
            can_next: true,
            can_previous: false,
            thumbnail: None,
        }
    }

    #[test]
    fn base64_matches_the_rfc_4648_vectors() {
        for (i, o) in [
            ("", ""),
            ("f", "Zg=="),
            ("fo", "Zm8="),
            ("foo", "Zm9v"),
            ("foob", "Zm9vYg=="),
            ("fooba", "Zm9vYmE="),
            ("foobar", "Zm9vYmFy"),
        ] {
            assert_eq!(base64_encode(i.as_bytes()), o);
        }
        assert_eq!(base64_encode(&[0xfb, 0xff, 0xfe]), "+//+");
    }

    const PNG: &[u8] = b"\x89PNG\r\n\x1a\nrest";

    #[test]
    fn sniff_recognises_the_five_types() {
        assert_eq!(sniff_image_type(PNG), Some("image/png"));
        assert_eq!(
            sniff_image_type(&[0xff, 0xd8, 0xff, 0xe0]),
            Some("image/jpeg")
        );
        assert_eq!(sniff_image_type(b"GIF89a.."), Some("image/gif"));
        assert_eq!(sniff_image_type(b"GIF87a.."), Some("image/gif"));
        assert_eq!(
            sniff_image_type(b"RIFF\x00\x00\x00\x00WEBPVP8 "),
            Some("image/webp")
        );
        assert_eq!(sniff_image_type(b"BM\x00\x00"), Some("image/bmp"));
    }

    #[test]
    fn sniff_rejects_everything_else() {
        assert_eq!(sniff_image_type(b""), None);
        assert_eq!(sniff_image_type(b"<svg xmlns="), None);
        assert_eq!(sniff_image_type(b"RIFF\x00\x00\x00\x00WAVEfmt "), None);
        assert_eq!(sniff_image_type(b"RIFF"), None);
        assert_eq!(sniff_image_type(b"\x89PNG"), None);
    }

    #[test]
    fn thumbnail_url_uses_the_sniffed_type_and_a_size_cap() {
        let url = thumbnail_data_url(PNG).unwrap();
        assert!(url.starts_with("data:image/png;base64,"));
        assert!(thumbnail_data_url(b"foo").is_none());
        assert!(thumbnail_data_url(b"").is_none());
        let mut big = PNG.to_vec();
        big.resize(MAX_THUMBNAIL_BYTES, 0);
        assert!(thumbnail_data_url(&big).is_some());
        big.push(0);
        assert!(thumbnail_data_url(&big).is_none());
    }

    #[test]
    fn state_serializes_for_the_ui() {
        let v = serde_json::to_value(playing()).unwrap();
        assert_eq!(v["status"], "playing");
        assert_eq!(v["canPlayPause"], true);
        assert_eq!(v["canPrevious"], false);
        assert!(v.get("thumbnail").is_none());
        assert_eq!(
            serde_json::to_value(MediaState::none()).unwrap()["status"],
            "none"
        );
    }

    #[test]
    fn commands_parse_from_snake_case() {
        let c: MediaControl = serde_json::from_str("\"play_pause\"").unwrap();
        assert_eq!(c, MediaControl::PlayPause);
        assert!(serde_json::from_str::<MediaControl>("\"stop\"").is_err());
    }

    #[test]
    fn fake_records_allowed_commands_only() {
        let s = FakeMediaSession::new(playing());
        s.send(MediaControl::PlayPause).unwrap();
        s.send(MediaControl::Next).unwrap();
        assert!(s.send(MediaControl::Previous).is_err());
        assert_eq!(s.sent(), vec![MediaControl::PlayPause, MediaControl::Next]);
        assert!(FakeMediaSession::no_player()
            .send(MediaControl::Next)
            .is_err());
    }

    #[test]
    fn poll_change_reports_only_changes_and_re_emits_after_error() {
        let s = FakeMediaSession::new(playing());
        let mut last = None;
        assert!(poll_change(&s, &mut last).unwrap().is_some());
        assert!(poll_change(&s, &mut last).unwrap().is_none());
        let mut next = playing();
        next.title = "Other".into();
        s.set_state(next.clone());
        assert_eq!(poll_change(&s, &mut last).unwrap(), Some(next));
        let bad = FakeMediaSession::unavailable();
        assert!(poll_change(&bad, &mut last).is_err());
        assert!(poll_change(&s, &mut last).unwrap().is_some());
    }
}
