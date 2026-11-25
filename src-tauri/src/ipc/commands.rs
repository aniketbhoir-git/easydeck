//! Tauri commands. `run_action` is a stub until the action runners land (R2).

use super::state::AppState;
use crate::error::AppError;
use easydeck_core::deck::Deck;

pub fn load_deck_impl(state: &AppState) -> Deck {
    // Positions are filled in here, once, so the UI never places widgets itself.
    easydeck_core::layout::resolve(state.deck())
}

/// Stub: checks the widget exists and returns Ok. Never launches or sends anything.
/// `hold` selects the long-press `holdAction` instead of `action`; a widget without one is
/// not found for a hold.
pub fn run_action_impl(state: &AppState, widget_id: &str, hold: bool) -> Result<(), AppError> {
    if state.is_toggle(widget_id) {
        return Err(AppError::Invalid {
            message: "use toggle: on|off".into(),
            detail: Some(widget_id.to_string()),
        });
    }
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

/// Stub: resolves which action a toggle runs when it switches to `to_on`, and returns Ok.
/// Never launches or sends anything. The UI sends the target state; the backend picks
/// `onAction` or `offAction` from the deck, so the UI never names an action. When approval
/// lands (R2), it must check this resolved action, the same as for `action` and `holdAction`.
/// Switching off a toggle that has no `offAction` runs nothing and is Ok.
pub fn run_toggle_impl(state: &AppState, widget_id: &str, to_on: bool) -> Result<(), AppError> {
    state.toggle_action(widget_id, to_on).map(|_action| ())
}

#[tauri::command]
pub fn load_deck(state: tauri::State<'_, AppState>) -> Deck {
    load_deck_impl(&state)
}

/// Dispatches a `run_action` call: plain, hold, or toggle. `hold` with `toggle` is invalid.
pub fn dispatch_impl(
    state: &AppState,
    widget_id: &str,
    hold: Option<bool>,
    toggle: Option<&str>,
) -> Result<(), AppError> {
    if toggle.is_some() && hold.unwrap_or(false) {
        return Err(AppError::Invalid {
            message: "hold cannot be combined with toggle".into(),
            detail: Some(widget_id.to_string()),
        });
    }
    match toggle {
        None => run_action_impl(state, widget_id, hold.unwrap_or(false)),
        Some("on") => run_toggle_impl(state, widget_id, true),
        Some("off") => run_toggle_impl(state, widget_id, false),
        Some(other) => Err(AppError::Invalid {
            message: "toggle must be \"on\" or \"off\"".into(),
            detail: Some(other.to_string()),
        }),
    }
}

#[tauri::command]
pub fn run_action(
    state: tauri::State<'_, AppState>,
    widget_id: String,
    hold: Option<bool>,
    toggle: Option<String>,
) -> Result<(), AppError> {
    dispatch_impl(&state, &widget_id, hold, toggle.as_deref())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn load_deck_returns_bundled_deck() {
        let state = AppState::bundled().unwrap();
        assert_eq!(load_deck_impl(&state).name, "Starter deck");
        assert!(load_deck_impl(&state)
            .pages
            .iter()
            .flat_map(|p| &p.widgets)
            .all(|w| w.position.is_some()));
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

    const TOGGLE_DECK: &str = r#"{"version":1,"name":"t","pages":[{"id":"home","title":"Home",
        "grid":{"columns":3,"rows":1},"widgets":[
        {"id":"both","type":"toggle","onAction":{"type":"key_macro","keys":["a"]},
         "offAction":{"type":"key_macro","keys":["b"]}},
        {"id":"one","type":"toggle","onAction":{"type":"key_macro","keys":["a"]}},
        {"id":"plain","type":"macro","action":{"type":"key_macro","keys":["c"]}}]}]}"#;

    #[test]
    fn toggle_picks_on_or_off_action() {
        use easydeck_core::deck::Action;
        let state = AppState::from_json(TOGGLE_DECK).unwrap();
        let keys = |to_on| match state.toggle_action("both", to_on).unwrap() {
            Some(Action::KeyMacro { keys, .. }) => keys.clone(),
            other => panic!("unexpected {other:?}"),
        };
        assert_eq!(keys(true), ["a"]);
        assert_eq!(keys(false), ["b"]);
        assert!(run_toggle_impl(&state, "both", true).is_ok());
    }

    #[test]
    fn toggle_off_without_off_action_runs_nothing() {
        let state = AppState::from_json(TOGGLE_DECK).unwrap();
        assert!(state.toggle_action("one", false).unwrap().is_none());
        assert!(run_toggle_impl(&state, "one", false).is_ok());
        assert!(state.toggle_action("one", true).unwrap().is_some());
    }

    #[test]
    fn plain_run_action_on_toggle_is_invalid() {
        let state = AppState::from_json(TOGGLE_DECK).unwrap();
        let err = run_action_impl(&state, "both", false).unwrap_err();
        assert_eq!(err.code(), "invalid");
    }

    #[test]
    fn hold_with_toggle_is_invalid() {
        let state = AppState::from_json(TOGGLE_DECK).unwrap();
        let err = dispatch_impl(&state, "both", Some(true), Some("on")).unwrap_err();
        assert_eq!(err.code(), "invalid");
        assert!(dispatch_impl(&state, "both", None, Some("on")).is_ok());
    }

    #[test]
    fn toggle_rejects_other_widgets() {
        let state = AppState::from_json(TOGGLE_DECK).unwrap();
        assert_eq!(
            run_toggle_impl(&state, "plain", true).unwrap_err().code(),
            "invalid"
        );
        assert_eq!(
            run_toggle_impl(&state, "ghost", true).unwrap_err().code(),
            "not_found"
        );
    }

    #[test]
    fn run_action_not_found_for_unknown_widget() {
        let state = AppState::bundled().unwrap();
        let err = run_action_impl(&state, "ghost", false).unwrap_err();
        assert_eq!(err.code(), "not_found");
    }
}
