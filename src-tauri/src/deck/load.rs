//! Read a deck file from disk.

use super::model::Deck;
use super::validate::{check, validate, ValidationError};
use serde_json::Value;
use std::path::Path;

/// Entry point for deck files (SPEC §5). `save` and `diff` arrive in later issues.
pub struct DeckStore;

impl DeckStore {
    /// Load, validate and type a deck file. Errors use the empty path for file-level problems.
    pub fn load(path: impl AsRef<Path>) -> Result<Deck, Vec<ValidationError>> {
        let path = path.as_ref();
        let text = std::fs::read_to_string(path).map_err(|e| {
            vec![ValidationError::new(
                "",
                format!("could not read {}: {e}", path.display()),
            )]
        })?;
        let value: Value = serde_json::from_str(&text)
            .map_err(|e| vec![ValidationError::new("", format!("not valid JSON: {e}"))])?;
        check(&value)
    }

    /// See [`validate`].
    pub fn validate(value: &Value) -> Vec<ValidationError> {
        validate(value)
    }
}
