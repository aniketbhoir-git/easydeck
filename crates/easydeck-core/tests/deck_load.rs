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
    ("invalid_bad_widget_id.deck.json", "/pages/0/widgets/0", ""),
    (
        "invalid_launch_missing_appid.deck.json",
        "/pages/0/widgets/2",
        "",
    ),
    ("invalid_empty_pages.deck.json", "/pages", ""),
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
