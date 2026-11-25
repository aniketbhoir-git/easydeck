//! The system media session (R2-3): what is playing and the three transport commands. Pure
//! logic behind a trait: the Windows code lives in `easydeck-platform-windows`, and tests use
//! [`FakeMediaSession`]. No account, no token, no network call.

use serde::{Deserialize, Serialize};
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MediaCommand {
    PlayPause,
    Next,
    Previous,
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
    fn send(&self, command: MediaCommand) -> Result<(), MediaError>;
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

/// A `data:` URL for a thumbnail, or `None` when the type is not an image or the image is
/// empty or larger than [`MAX_THUMBNAIL_BYTES`].
pub fn thumbnail_data_url(content_type: &str, bytes: &[u8]) -> Option<String> {
    let ct = content_type.trim().to_ascii_lowercase();
    let ok_type = ct.starts_with("image/")
        && ct.len() > 6
        && ct[6..]
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '+' || c == '.' || c == '-');
    if !ok_type || bytes.is_empty() || bytes.len() > MAX_THUMBNAIL_BYTES {
        return None;
    }
    Some(format!("data:{ct};base64,{}", base64_encode(bytes)))
}

/// An in-memory session for tests. It never touches a real player.
pub struct FakeMediaSession {
    state: Mutex<Result<MediaState, MediaError>>,
    sent: Mutex<Vec<MediaCommand>>,
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
    pub fn sent(&self) -> Vec<MediaCommand> {
        self.sent.lock().unwrap_or_else(|e| e.into_inner()).clone()
    }
}

impl MediaSession for FakeMediaSession {
    fn get(&self) -> Result<MediaState, MediaError> {
        self.state.lock().unwrap_or_else(|e| e.into_inner()).clone()
    }

    fn send(&self, command: MediaCommand) -> Result<(), MediaError> {
        let state = self.get()?;
        let allowed = match command {
            MediaCommand::PlayPause => state.can_play_pause,
            MediaCommand::Next => state.can_next,
            MediaCommand::Previous => state.can_previous,
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

    #[test]
    fn thumbnail_url_accepts_small_images_only() {
        assert_eq!(
            thumbnail_data_url("image/PNG", b"foo").unwrap(),
            "data:image/png;base64,Zm9v"
        );
        assert!(thumbnail_data_url("text/html", b"foo").is_none());
        assert!(thumbnail_data_url("image/png;x", b"foo").is_none());
        assert!(thumbnail_data_url("image/png", b"").is_none());
        assert!(thumbnail_data_url("image/png", &vec![0; MAX_THUMBNAIL_BYTES + 1]).is_none());
        assert!(thumbnail_data_url("image/png", &vec![0; MAX_THUMBNAIL_BYTES]).is_some());
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
        let c: MediaCommand = serde_json::from_str("\"play_pause\"").unwrap();
        assert_eq!(c, MediaCommand::PlayPause);
        assert!(serde_json::from_str::<MediaCommand>("\"stop\"").is_err());
    }

    #[test]
    fn fake_records_allowed_commands_only() {
        let s = FakeMediaSession::new(playing());
        s.send(MediaCommand::PlayPause).unwrap();
        s.send(MediaCommand::Next).unwrap();
        assert!(s.send(MediaCommand::Previous).is_err());
        assert_eq!(s.sent(), vec![MediaCommand::PlayPause, MediaCommand::Next]);
        assert!(FakeMediaSession::no_player()
            .send(MediaCommand::Next)
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
