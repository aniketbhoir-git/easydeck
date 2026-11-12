//! Shared app state held by Tauri.

use crate::error::AppError;
use easydeck_core::deck::{Deck, DeckStore};

/// Default deck, bundled at build time. SPEC §4 does not name a deck location yet, so the
/// renderer starts from the example deck until the app settings issue adds a user path.
const BUNDLED_DECK: &str = include_str!("../../../examples/starter.deck.json");

pub struct AppState {
    deck: Deck,
}

impl AppState {
    pub fn from_json(text: &str) -> Result<Self, AppError> {
        let deck = DeckStore::load_str(text).map_err(|errors| AppError::Invalid {
            message: "the deck file is not valid".into(),
            detail: Some(
                errors
                    .iter()
                    .map(|e| format!("{}: {}", e.path, e.message))
                    .collect::<Vec<_>>()
                    .join("; "),
            ),
        })?;
        Ok(Self { deck })
    }

    pub fn bundled() -> Result<Self, AppError> {
        Self::from_json(BUNDLED_DECK)
    }

    pub fn deck(&self) -> &Deck {
        &self.deck
    }

    /// True when any page has a widget with this id.
    pub fn has_widget(&self, widget_id: &str) -> bool {
        self.deck
            .pages
            .iter()
            .flat_map(|p| &p.widgets)
            .any(|w| w.id == widget_id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bundled_deck_loads() {
        let state = AppState::bundled().unwrap();
        assert!(!state.deck().pages.is_empty());
        assert!(state.has_widget("notes"));
        assert!(!state.has_widget("nope"));
    }

    #[test]
    fn bad_json_is_invalid_error() {
        let err = AppState::from_json("{").err().unwrap();
        assert_eq!(err.code(), "invalid");
    }
}
