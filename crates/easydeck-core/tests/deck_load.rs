//! Table-driven deck load and validate tests. Fixtures live in `tests/fixtures/` at the repository root.

use easydeck_core::deck::{Deck, DeckStore, ValidationError};
use std::path::PathBuf;

fn fixture(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../tests/fixtures")
        .join(name)
}

fn load(name: &str) -> Result<Deck, Vec<ValidationError>> {
    DeckStore::load(fixture(name))
}

#[test]
fn starter_example_loads() {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../examples/starter.deck.json");
    let deck = DeckStore::load(path).expect("starter deck should load");
    assert_eq!(deck.pages.len(), 2);
}

#[test]
fn full_valid_fixture_loads() {
    let deck = load("valid_full.deck.json").expect("valid fixture should load");
    assert_eq!(deck.pages.len(), 2);
    assert_eq!(deck.pages[1].widgets.len(), 5);
}

#[test]
fn clock_fixture_loads() {
    let deck = load("valid_clock.deck.json").expect("clock fixture should load");
    assert_eq!(deck.pages[0].widgets.len(), 4);
}

#[test]
fn counter_fixture_loads() {
    let deck = load("valid_counter.deck.json").expect("counter fixture should load");
    assert_eq!(deck.pages[0].widgets.len(), 2);
}

#[test]
fn volume_fixture_loads() {
    let deck = load("valid_volume.deck.json").expect("volume fixture should load");
    assert_eq!(deck.pages[0].widgets.len(), 2);
}

#[test]
fn timer_fixture_loads() {
    let deck = load("valid_timer.deck.json").expect("timer fixture should load");
    assert_eq!(deck.pages[0].widgets.len(), 2);
}

#[test]
fn toggle_fixture_loads() {
    let deck = load("valid_toggle.deck.json").expect("toggle fixture should load");
    assert_eq!(deck.pages[0].widgets.len(), 2);
}

/// (fixture, expected error path, expected message substring)
const INVALID: &[(&str, &str, &str)] = &[
    (
        "invalid_version.deck.json",
        "/version",
        "unsupported deck version 2; this build supports 1",
    ),
    ("invalid_missing_name.deck.json", "", "name"),
    ("invalid_unknown_top_field.deck.json", "", "extra"),
    ("invalid_widget_type.deck.json", "/pages/0/widgets/0", ""),
    (
        "invalid_bad_widget_id.deck.json",
        "/pages/0/widgets/0/id",
        "does not match",
    ),
    (
        "invalid_widget_unknown_field.deck.json",
        "/pages/0/widgets/0",
        "colour",
    ),
    (
        "invalid_widget_wrong_action.deck.json",
        "/pages/0/widgets/2",
        "not a launch action",
    ),
    (
        "invalid_widget_wrong_hold_action.deck.json",
        "/pages/0/widgets/2",
        "its holdAction is not a launch action",
    ),
    (
        "invalid_launch_missing_appid.deck.json",
        "/pages/0/widgets/2",
        "",
    ),
    ("invalid_empty_pages.deck.json", "/pages", ""),
    (
        "invalid_overlap.deck.json",
        "/pages/0/widgets/1/position",
        "overlap",
    ),
    (
        "invalid_no_space.deck.json",
        "/pages/0/widgets/2",
        "no free cell",
    ),
    (
        "invalid_dup_widget_id.deck.json",
        "/pages/0/widgets/1/id",
        "duplicate",
    ),
    (
        "invalid_widget_off_grid_x.deck.json",
        "/pages/0/widgets/1/position",
        "grid",
    ),
    (
        "invalid_widget_off_grid_y.deck.json",
        "/pages/0/widgets/2/position",
        "grid",
    ),
    (
        "invalid_trigger_not_game_watch.deck.json",
        "/pages/1/trigger/source",
        "game_watch",
    ),
    (
        "invalid_trigger_unknown_source.deck.json",
        "/pages/1/trigger/source",
        "game_watch",
    ),
    (
        "invalid_gauge_unknown_source.deck.json",
        "/pages/0/widgets/1/source",
        "not defined",
    ),
    (
        "invalid_status_unknown_source.deck.json",
        "/pages/1/widgets/2/source",
        "not defined",
    ),
    (
        "invalid_home_page_missing.deck.json",
        "/settings/homePage",
        "not a page id",
    ),
    (
        "invalid_null_optional.deck.json",
        "/pages/0/widgets/0/label",
        "null is not of type",
    ),
    (
        "invalid_counter_step.deck.json",
        "/pages/0/widgets/0",
        "step 0",
    ),
    (
        "invalid_counter_range.deck.json",
        "/pages/0/widgets/0",
        "outside its min and max",
    ),
    (
        "invalid_counter_minmax.deck.json",
        "/pages/0/widgets/0",
        "min above max",
    ),
    (
        "invalid_counter_unsafe.deck.json",
        "/pages/0/widgets/0",
        "safe whole-number range",
    ),
    (
        "invalid_timer_duration.deck.json",
        "/pages/0/widgets/0",
        "outside 1 to 86400",
    ),
    (
        "invalid_timer_stopwatch.deck.json",
        "/pages/0/widgets/0",
        "must not set durationS",
    ),
    (
        "invalid_toggle_color.deck.json",
        "/pages/0/widgets/0",
        "not a hex colour",
    ),
    (
        "invalid_volume_field.deck.json",
        "/pages/0/widgets/0",
        "step",
    ),
    ("invalid_not_json.deck.json", "", "not valid JSON"),
    ("does_not_exist.deck.json", "", "could not read"),
];

#[test]
fn invalid_fixtures_report_expected_path() {
    for (name, path, msg) in INVALID {
        let errors = load(name).expect_err(name);
        assert!(
            errors
                .iter()
                .any(|e| e.path == *path && e.message.contains(msg)),
            "{name}: expected path {path:?} with {msg:?}, got {errors:#?}"
        );
    }
}

#[test]
fn validate_accepts_value_and_matches_load() {
    let text = std::fs::read_to_string(fixture("invalid_dup_widget_id.deck.json")).unwrap();
    let value: serde_json::Value = serde_json::from_str(&text).unwrap();
    let errors = DeckStore::validate(&value);
    assert_eq!(errors.len(), 1);
    assert_eq!(errors[0].path, "/pages/0/widgets/1/id");
}

/// Snapshots hold the messages an LLM reads: path, rule, message and fix.
#[test]
fn invalid_fixture_messages_are_stable() {
    for (name, ..) in INVALID {
        if *name == "does_not_exist.deck.json" {
            continue;
        }
        let errors = load(name).expect_err(name);
        insta::assert_json_snapshot!(name.trim_end_matches(".deck.json"), errors);
    }
}

fn valid_value() -> serde_json::Value {
    let text = std::fs::read_to_string(fixture("valid_full.deck.json")).unwrap();
    serde_json::from_str(&text).unwrap()
}

#[test]
fn every_error_has_a_rule_and_a_fix() {
    for (name, ..) in INVALID {
        for e in load(name).expect_err(name) {
            assert!(!e.rule.is_empty() && !e.fix.is_empty(), "{name}: {e:?}");
        }
    }
}

#[test]
fn launch_kinds_need_their_field() {
    for (kind, field) in [("exe", "path"), ("uri", "target"), ("url", "target")] {
        let mut deck = valid_value();
        deck["pages"][0]["widgets"][2]["action"] =
            serde_json::json!({"type": "launch", "kind": kind});
        let errors = DeckStore::validate(&deck);
        assert!(
            errors.iter().any(|e| e.path == "/pages/0/widgets/2"
                && e.rule == "launch-required-field"
                && e.message.contains(field)),
            "{kind}: {errors:#?}"
        );
    }
}

#[test]
fn media_has_no_provider_and_no_volume() {
    let mut deck = valid_value();
    deck["pages"][1]["widgets"][3]["provider"] = serde_json::json!("spotify");
    let errors = DeckStore::validate(&deck);
    assert!(errors
        .iter()
        .any(|e| e.path == "/pages/1/widgets/3" && e.rule == "unknown-field"));

    let mut deck = valid_value();
    deck["pages"][1]["widgets"][3]["controls"] = serde_json::json!(["volume"]);
    assert!(!DeckStore::validate(&deck).is_empty());
}

#[test]
fn deck_without_positions_loads_and_is_placed_in_reading_order() {
    use easydeck_core::layout::place;
    let deck = load("valid_no_positions.deck.json").expect("deck without positions is valid");
    let placed = place(&deck.pages[0]).unwrap();
    let at: Vec<_> = placed.iter().map(|p| (p.id.as_str(), p.x, p.y)).collect();
    assert_eq!(
        at,
        [
            ("a", 0, 0),
            ("b", 1, 0),
            ("c", 2, 0),
            ("d", 0, 1),
            ("e", 1, 1)
        ]
    );
}

#[test]
fn overlap_error_names_both_widgets() {
    let errors = load("invalid_overlap.deck.json").unwrap_err();
    assert!(errors.iter().any(|e| e.rule == "overlap"
        && e.path == "/pages/0/widgets/1/position"
        && e.message.contains("\"a\"")
        && e.message.contains("\"b\"")));
}
