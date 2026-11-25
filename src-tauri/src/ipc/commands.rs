//! Tauri commands. `run_action` is a stub until the action runners land (R2).

use super::state::AppState;
use crate::error::AppError;
use easydeck_core::deck::{Deck, RiskReview};

pub fn load_deck_impl(state: &AppState) -> Deck {
    // Positions are filled in here, once, so the UI never places widgets itself.
    easydeck_core::layout::resolve(state.deck())
}

/// Stub: checks the widget exists, checks the approval ledger and returns Ok. Never launches
/// or sends anything (R1-5 and R3-1 add the runners behind this gate).
/// `hold` selects the long-press `holdAction` instead of `action`; a widget without one is
/// not found for a hold. An action that is not approved gives `needs_approval` (SPEC §6.1).
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
        // The action that really runs: `action` for a tap, `holdAction` for a hold.
        state.authorize(state.tap_actions(widget_id, hold))
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

/// Stub: resolves which action a toggle runs when it switches to `to_on`, checks the approval
/// ledger for that action and returns Ok. Never launches or sends anything. The UI sends the
/// target state; the backend picks `onAction` or `offAction` from the deck, so the UI never
/// names an action, and the gate checks the action that really runs.
/// Switching off a toggle that has no `offAction` runs nothing and is Ok.
pub fn run_toggle_impl(state: &AppState, widget_id: &str, to_on: bool) -> Result<(), AppError> {
    state.authorize(state.toggle_action(widget_id, to_on)?)
}

/// Validates a draft deck from the editor with the same rules as a deck file. An empty list
/// means the draft is valid. It reads nothing and changes nothing.
pub fn validate_deck_impl(deck: &serde_json::Value) -> Vec<easydeck_core::deck::ValidationError> {
    easydeck_core::deck::validate(deck)
}

/// Largest draft deck `validate_deck` accepts, as serialized JSON.
pub const MAX_DRAFT_BYTES: usize = 1024 * 1024;

/// Rejects a draft over `MAX_DRAFT_BYTES` before validating it.
pub fn validate_deck_checked(
    deck: &serde_json::Value,
) -> Result<Vec<easydeck_core::deck::ValidationError>, AppError> {
    let size = serde_json::to_vec(deck)
        .map(|v| v.len())
        .unwrap_or(usize::MAX);
    if size > MAX_DRAFT_BYTES {
        return Err(AppError::Invalid {
            message: "deck is larger than 1 MiB".into(),
            detail: Some(format!("{size} bytes")),
        });
    }
    Ok(validate_deck_impl(deck))
}

// Async so the validation runs off the main thread.
#[tauri::command]
pub async fn validate_deck(
    deck: serde_json::Value,
) -> Result<Vec<easydeck_core::deck::ValidationError>, AppError> {
    validate_deck_checked(&deck)
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

/// The risk items of the loaded deck, each marked approved or not (SPEC §4.3, §6.5). The
/// notice, if any (a damaged ledger file), comes once.
#[tauri::command]
pub fn risk_review(state: tauri::State<'_, AppState>) -> RiskReview {
    state.risk_review()
}

/// Approves the items with these hashes, then starts the sources that were waiting for it.
pub fn approve_impl(state: &AppState, hashes: &[String]) -> Result<RiskReview, AppError> {
    state.approve(hashes)?;
    Ok(state.risk_review())
}

/// Revokes the approvals with these hashes.
pub fn revoke_impl(state: &AppState, hashes: &[String]) -> Result<RiskReview, AppError> {
    state.revoke(hashes)?;
    Ok(state.risk_review())
}

#[tauri::command]
pub fn approve_risk_items(
    app: tauri::AppHandle,
    state: tauri::State<'_, AppState>,
    runner: tauri::State<'_, crate::sources::SourceRunner>,
    hashes: Vec<String>,
) -> Result<RiskReview, AppError> {
    let review = approve_impl(&state, &hashes)?;
    runner.restart(&app, state.deck(), |s| state.source_allowed(s));
    Ok(review)
}

#[tauri::command]
pub fn revoke_risk_items(
    app: tauri::AppHandle,
    state: tauri::State<'_, AppState>,
    runner: tauri::State<'_, crate::sources::SourceRunner>,
    hashes: Vec<String>,
) -> Result<RiskReview, AppError> {
    let review = revoke_impl(&state, &hashes)?;
    runner.restart(&app, state.deck(), |s| state.source_allowed(s));
    Ok(review)
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
    fn validate_deck_accepts_the_loaded_deck_and_flags_a_broken_one() {
        let state = AppState::bundled().unwrap();
        let good = serde_json::to_value(load_deck_impl(&state)).unwrap();
        assert!(validate_deck_impl(&good).is_empty());
        let mut bad = good.clone();
        bad["pages"][0]["widgets"][0]["bogus"] = serde_json::json!(1);
        assert!(!validate_deck_impl(&bad).is_empty());
    }

    #[test]
    fn validate_deck_rejects_input_over_1_mib() {
        let big = serde_json::json!({ "pad": "x".repeat(MAX_DRAFT_BYTES) });
        let err = validate_deck_checked(&big).unwrap_err();
        assert_eq!(err.code(), "invalid");
        assert!(validate_deck_checked(&serde_json::json!({})).is_ok());
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

    /// Approves every risk item of the deck: the state of a user who approved the whole deck.
    fn approved(state: AppState) -> AppState {
        let hashes: Vec<String> = easydeck_core::deck::risk_items(state.deck())
            .into_iter()
            .map(|i| i.hash)
            .collect();
        state.approve(&hashes).unwrap();
        state
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
        let state = approved(AppState::from_json(TOGGLE_DECK).unwrap());
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
        // Nothing to run, so nothing to approve.
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
        let state = approved(AppState::from_json(TOGGLE_DECK).unwrap());
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

    // ---- the approval gate (A-8) ----

    const GATE_DECK: &str = r#"{"version":1,"name":"g","pages":[{"id":"home","title":"Home",
        "grid":{"columns":4,"rows":2},"widgets":[
        {"id":"game","type":"launch",
         "action":{"type":"launch","kind":"exe","path":"C:/Games/a.exe"},
         "holdAction":{"type":"launch","kind":"exe","path":"C:/Games/b.exe"}},
        {"id":"keys","type":"macro","action":{"type":"key_macro","keys":["1"]},
         "holdAction":{"type":"key_macro","keys":["2"]}},
        {"id":"sw","type":"toggle","onAction":{"type":"key_macro","keys":["3"]},
         "offAction":{"type":"shell","command":"lock-screen"}},
        {"id":"nav","type":"toggle","onAction":{"type":"page","page":"home"}},
        {"id":"cpu","type":"text","kind":"clock"}]}]}"#;

    fn hash_of(state: &AppState, summary_part: &str) -> String {
        easydeck_core::deck::risk_items(state.deck())
            .into_iter()
            .find(|i| i.summary.contains(summary_part))
            .unwrap_or_else(|| panic!("no item with {summary_part}"))
            .hash
    }

    fn code(r: Result<(), AppError>) -> &'static str {
        match r {
            Ok(()) => "ok",
            Err(e) => e.code(),
        }
    }

    #[test]
    fn an_unapproved_action_is_refused_on_every_dispatch_path() {
        let s = AppState::from_json(GATE_DECK).unwrap();
        let d = |id, hold, toggle| dispatch_impl(&s, id, hold, toggle);
        assert_eq!(code(d("game", None, None)), "needs_approval");
        assert_eq!(code(d("game", Some(true), None)), "needs_approval");
        assert_eq!(code(d("keys", None, None)), "needs_approval");
        assert_eq!(code(d("keys", Some(true), None)), "needs_approval");
        assert_eq!(code(d("sw", None, Some("on"))), "needs_approval");
        assert_eq!(code(d("sw", None, Some("off"))), "needs_approval");
        // What the user sees names the action.
        let err = d("sw", None, Some("off")).unwrap_err();
        assert_eq!(err.detail(), Some("run shell command \"lock-screen\""));
        assert!(!err.retryable());
    }

    #[test]
    fn the_gate_checks_the_action_that_really_runs() {
        let s = AppState::from_json(GATE_DECK).unwrap();
        // Approve only the tap action of `game`: a hold still needs approval.
        s.approve(&[hash_of(&s, "a.exe")]).unwrap();
        assert_eq!(code(dispatch_impl(&s, "game", None, None)), "ok");
        assert_eq!(
            code(dispatch_impl(&s, "game", Some(true), None)),
            "needs_approval"
        );
        // Approve only the hold action of `keys`: a tap still needs approval.
        s.approve(&[hash_of(&s, "keys 2")]).unwrap();
        assert_eq!(code(dispatch_impl(&s, "keys", Some(true), None)), "ok");
        assert_eq!(
            code(dispatch_impl(&s, "keys", None, None)),
            "needs_approval"
        );
        // Approve only onAction of the toggle: off still needs approval.
        s.approve(&[hash_of(&s, "keys 3")]).unwrap();
        assert_eq!(code(dispatch_impl(&s, "sw", None, Some("on"))), "ok");
        assert_eq!(
            code(dispatch_impl(&s, "sw", None, Some("off"))),
            "needs_approval"
        );
        s.approve(&[hash_of(&s, "lock-screen")]).unwrap();
        assert_eq!(code(dispatch_impl(&s, "sw", None, Some("off"))), "ok");
    }

    #[test]
    fn page_switches_and_widgets_that_run_nothing_need_no_approval() {
        let s = AppState::from_json(GATE_DECK).unwrap();
        assert_eq!(code(dispatch_impl(&s, "nav", None, Some("on"))), "ok");
        assert_eq!(code(dispatch_impl(&s, "cpu", None, None)), "ok");
    }

    #[test]
    fn existing_errors_come_before_the_gate() {
        let s = AppState::from_json(GATE_DECK).unwrap();
        assert_eq!(code(dispatch_impl(&s, "ghost", None, None)), "not_found");
        assert_eq!(
            code(dispatch_impl(&s, "cpu", Some(true), None)),
            "not_found"
        );
        assert_eq!(code(dispatch_impl(&s, "game", None, Some("on"))), "invalid");
        assert_eq!(code(dispatch_impl(&s, "sw", None, None)), "invalid");
    }

    #[test]
    fn a_widget_id_on_two_pages_needs_both_approved() {
        let deck = r#"{"version":1,"name":"g","pages":[
          {"id":"p1","title":"1","grid":{"columns":2,"rows":1},"widgets":[
            {"id":"same","type":"macro","action":{"type":"key_macro","keys":["1"]}}]},
          {"id":"p2","title":"2","grid":{"columns":2,"rows":1},"widgets":[
            {"id":"same","type":"macro","action":{"type":"key_macro","keys":["9"]}}]}]}"#;
        let s = AppState::from_json(deck).unwrap();
        s.approve(&[hash_of(&s, "keys 1")]).unwrap();
        assert_eq!(
            code(dispatch_impl(&s, "same", None, None)),
            "needs_approval"
        );
        s.approve(&[hash_of(&s, "keys 9")]).unwrap();
        assert_eq!(code(dispatch_impl(&s, "same", None, None)), "ok");
    }

    #[test]
    fn revoke_locks_the_action_again() {
        let s = approved(AppState::from_json(GATE_DECK).unwrap());
        assert_eq!(code(dispatch_impl(&s, "game", None, None)), "ok");
        s.revoke(&[hash_of(&s, "a.exe")]).unwrap();
        assert_eq!(
            code(dispatch_impl(&s, "game", None, None)),
            "needs_approval"
        );
        assert_eq!(s.revoke(&["nope".into()]).unwrap_err().code(), "invalid");
    }

    #[test]
    fn only_hashes_of_the_loaded_deck_can_be_approved() {
        let s = AppState::from_json(GATE_DECK).unwrap();
        for bad in ["", "abc", &"0".repeat(64)] {
            let err = approve_impl(&s, &[bad.to_string()]).unwrap_err();
            assert_eq!(err.code(), "invalid");
        }
        // A bad hash in a list approves none of the list.
        let good = hash_of(&s, "a.exe");
        assert!(approve_impl(&s, &[good.clone(), "x".into()]).is_err());
        assert!(s.risk_review().items.iter().all(|i| !i.approved));
        let review = approve_impl(&s, std::slice::from_ref(&good)).unwrap();
        assert!(review
            .items
            .iter()
            .any(|i| i.approved && i.item.hash == good));
    }

    #[test]
    fn the_bundled_starter_deck_is_approved_and_a_loaded_deck_is_not() {
        let state = AppState::bundled().unwrap();
        assert!(state.risk_review().items.iter().all(|i| i.approved));
        assert!(AppState::from_json(GATE_DECK)
            .unwrap()
            .risk_review()
            .items
            .iter()
            .all(|i| !i.approved));
    }

    // ---- the ledger file ----

    fn temp_dir(tag: &str) -> std::path::PathBuf {
        use std::sync::atomic::{AtomicUsize, Ordering};
        static N: AtomicUsize = AtomicUsize::new(0);
        let dir = std::env::temp_dir().join(format!(
            "easydeck-app-{tag}-{}-{}",
            std::process::id(),
            N.fetch_add(1, Ordering::Relaxed)
        ));
        let _ = std::fs::remove_dir_all(&dir);
        dir
    }

    #[test]
    fn first_start_seeds_the_starter_items_and_the_next_start_keeps_them() {
        let dir = temp_dir("first");
        let state = AppState::open(Some(&dir)).unwrap();
        let review = state.risk_review();
        assert!(review.notice.is_none());
        assert!(review.items.iter().all(|i| i.approved));
        let text = std::fs::read_to_string(dir.join("approvals.json")).unwrap();
        assert!(text.contains("\"version\": 1"));
        // Revoke one, restart: the file is read, not seeded again.
        let first = review.items[0].item.hash.clone();
        state.revoke(std::slice::from_ref(&first)).unwrap();
        let again = AppState::open(Some(&dir)).unwrap().risk_review();
        assert!(again
            .items
            .iter()
            .any(|i| i.item.hash == first && !i.approved));
        assert!(again.notice.is_none());
        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn a_corrupt_ledger_is_empty_with_one_notice_and_a_copy() {
        let dir = temp_dir("corrupt");
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("approvals.json"), "garbage").unwrap();
        let state = AppState::open(Some(&dir)).unwrap();
        let first = state.risk_review();
        assert!(first
            .notice
            .as_deref()
            .unwrap()
            .contains("approvals.json.corrupt"));
        assert!(first.items.iter().all(|i| !i.approved));
        assert!(
            state.risk_review().notice.is_none(),
            "the notice shows once"
        );
        assert!(dir.join("approvals.json.corrupt").exists());
        assert_eq!(
            code(dispatch_impl(&state, "notes", None, None)),
            "needs_approval"
        );
        // Approving writes a fresh file.
        let hashes: Vec<String> = first.items.iter().map(|i| i.item.hash.clone()).collect();
        state.approve(&hashes).unwrap();
        assert_eq!(code(dispatch_impl(&state, "notes", None, None)), "ok");
        assert!(AppState::open(Some(&dir))
            .unwrap()
            .risk_review()
            .items
            .iter()
            .all(|i| i.approved));
        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn a_failed_save_approves_nothing() {
        // The "folder" is a file, so the ledger cannot be written.
        let dir = temp_dir("nosave");
        std::fs::create_dir_all(&dir).unwrap();
        let blocker = dir.join("blocker");
        std::fs::write(&blocker, "x").unwrap();
        let state = AppState::from_json(GATE_DECK)
            .unwrap()
            .with_file(easydeck_core::deck::LedgerFile::in_dir(blocker.join("sub")));
        let h = hash_of(&state, "a.exe");
        assert_eq!(state.approve(&[h]).unwrap_err().code(), "io");
        assert_eq!(
            code(dispatch_impl(&state, "game", None, None)),
            "needs_approval"
        );
        let _ = std::fs::remove_dir_all(dir);
    }
}
