use serde::ser::SerializeStruct;
use serde::{Serialize, Serializer};

/// App error. Serializes to `{ "code", "message", "detail", "retryable" }` for the UI.
#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("{message}")]
    Invalid {
        message: String,
        detail: Option<String>,
    },
    #[error("{message}")]
    NotFound {
        message: String,
        detail: Option<String>,
    },
    #[error("{message}")]
    Io {
        message: String,
        detail: Option<String>,
    },
    #[error("{message}")]
    Internal {
        message: String,
        detail: Option<String>,
    },
}

impl AppError {
    pub fn code(&self) -> &'static str {
        match self {
            Self::Invalid { .. } => "invalid",
            Self::NotFound { .. } => "not_found",
            Self::Io { .. } => "io",
            Self::Internal { .. } => "internal",
        }
    }

    pub fn detail(&self) -> Option<&str> {
        match self {
            Self::Invalid { detail, .. }
            | Self::NotFound { detail, .. }
            | Self::Io { detail, .. }
            | Self::Internal { detail, .. } => detail.as_deref(),
        }
    }

    /// True when trying the same call again may succeed.
    pub fn retryable(&self) -> bool {
        matches!(self, Self::Io { .. })
    }
}

impl Serialize for AppError {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut s = serializer.serialize_struct("AppError", 4)?;
        s.serialize_field("code", self.code())?;
        s.serialize_field("message", &self.to_string())?;
        s.serialize_field("detail", &self.detail())?;
        s.serialize_field("retryable", &self.retryable())?;
        s.end()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn serializes_to_json_shape() {
        let err = AppError::Io {
            message: "could not read deck".into(),
            detail: Some("file is locked".into()),
        };
        let v = serde_json::to_value(&err).unwrap();
        assert_eq!(
            v,
            serde_json::json!({
                "code": "io",
                "message": "could not read deck",
                "detail": "file is locked",
                "retryable": true
            })
        );
    }

    #[test]
    fn missing_detail_is_null_and_not_retryable() {
        let err = AppError::Invalid {
            message: "bad deck".into(),
            detail: None,
        };
        let v = serde_json::to_value(&err).unwrap();
        assert_eq!(v["code"], "invalid");
        assert!(v["detail"].is_null());
        assert_eq!(v["retryable"], false);
    }
}
