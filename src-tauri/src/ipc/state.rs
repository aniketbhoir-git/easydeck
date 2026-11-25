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

    /// True when a widget with this id has a long-press `holdAction`.
    pub fn has_hold_action(&self, widget_id: &str) -> bool {
        use easydeck_core::deck::WidgetKind;
        self.deck.pages.iter().flat_map(|p| &p.widgets).any(|w| {
            w.id == widget_id
                && matches!(
                    &w.kind,
                    WidgetKind::Launch {
                        hold_action: Some(_),
                        ..
                    } | WidgetKind::Macro {
                        hold_action: Some(_),
                        ..
                    }
                )
        })
    }

    /// The action to run when a toggle switches to `to_on`: its `onAction` or `offAction`.
    /// `Ok(None)` is a switch to off on a toggle without an `offAction`: nothing to run.
    /// `Err` when the widget does not exist or is not a toggle.
    pub fn toggle_action(
        &self,
        widget_id: &str,
        to_on: bool,
    ) -> Result<Option<&easydeck_core::deck::Action>, AppError> {
        use easydeck_core::deck::WidgetKind;
        let widget = self
            .deck
            .pages
            .iter()
            .flat_map(|p| &p.widgets)
            .find(|w| w.id == widget_id);
        match widget.map(|w| &w.kind) {
            Some(WidgetKind::Toggle {
                on_action,
                off_action,
                ..
            }) => Ok(if to_on {
                Some(on_action)
            } else {
                off_action.as_ref()
            }),
            Some(_) => Err(AppError::Invalid {
                message: "the widget is not a toggle".into(),
                detail: Some(widget_id.to_string()),
            }),
            None => Err(AppError::NotFound {
                message: "no such widget".into(),
                detail: Some(widget_id.to_string()),
            }),
        }
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
