//! Tauri commands. `run_action` is a stub until the action runners land (R2).

use super::state::AppState;
use crate::error::AppError;
use easydeck_core::deck::Deck;

pub fn load_deck_impl(state: &AppState) -> Deck {
    state.deck().clone()
}

/// Stub: checks the widget exists and returns Ok. Never launches or sends anything.
/// `hold` selects the long-press `holdAction` instead of `action`; a widget without one is
/// not found for a hold.
pub fn run_action_impl(state: &AppState, widget_id: &str, hold: bool) -> Result<(), AppError> {
    let found = if hold {
        state.has_hold_action(widget_id)
    } else {
        state.has_widget(widget_id)
    };
    if found {
        Ok(())
    } else {
        Err(AppError::NotFound {
            message: if hold {
                "no such widget or no hold action".into()
            } else {
                "no such widget".into()
            },
            detail: Some(widget_id.to_string()),
        })
    }
}

#[tauri::command]
pub fn load_deck(state: tauri::State<'_, AppState>) -> Deck {
    load_deck_impl(&state)
}

#[tauri::command]
pub fn run_action(
    state: tauri::State<'_, AppState>,
    widget_id: String,
    hold: Option<bool>,
) -> Result<(), AppError> {
    run_action_impl(&state, &widget_id, hold.unwrap_or(false))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn load_deck_returns_bundled_deck() {
        let state = AppState::bundled().unwrap();
        assert_eq!(load_deck_impl(&state).name, "Starter deck");
    }

    #[test]
    fn run_action_ok_for_known_widget() {
        let state = AppState::bundled().unwrap();
        assert!(run_action_impl(&state, "notes", false).is_ok());
    }

    #[test]
    fn run_action_hold_needs_a_hold_action() {
        let state = AppState::bundled().unwrap();
        let err = run_action_impl(&state, "notes", true).unwrap_err();
        assert_eq!(err.code(), "not_found");
    }

    #[test]
    fn run_action_not_found_for_unknown_widget() {
        let state = AppState::bundled().unwrap();
        let err = run_action_impl(&state, "ghost", false).unwrap_err();
        assert_eq!(err.code(), "not_found");
    }
}
