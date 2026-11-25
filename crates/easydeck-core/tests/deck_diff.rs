//! Deck diff tests (R5-1). Fixtures live in `tests/fixtures/` at the repository root.

use easydeck_core::deck::{ChangeKind, Deck, DeckStore, Position};
use proptest::prelude::*;
use std::path::PathBuf;

fn load(name: &str) -> Deck {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../tests/fixtures")
        .join(name);
    DeckStore::load(path).unwrap_or_else(|e| panic!("{name}: {e:?}"))
}

fn text(d: &easydeck_core::deck::DeckDiff) -> String {
    d.entries
        .iter()
        .map(|e| {
            format!(
                "{:?}{} {}: {}",
                e.kind,
                if e.runs { "!" } else { "" },
                e.path,
                e.summary
            )
        })
        .collect::<Vec<_>>()
        .join("\n")
}

#[test]
fn acceptance_added_moved_changed_gives_three_entries() {
    let d = DeckStore::diff(&load("diff_old.deck.json"), &load("diff_new.deck.json"));
    assert_eq!(d.entries.len(), 3, "{}", text(&d));
    let kinds: Vec<_> = d.entries.iter().map(|e| e.kind).collect();
    assert!(kinds.contains(&ChangeKind::Added));
    assert!(kinds.contains(&ChangeKind::Moved));
    assert!(kinds.contains(&ChangeKind::Changed));
    // The moved tile is not a risk item; the added launch and changed macro are.
    let risky: Vec<_> = d.risky().map(|e| e.path.as_str()).collect();
    assert_eq!(
        risky,
        ["pages/home/widgets/boost", "pages/home/widgets/game"]
    );
    insta::assert_snapshot!("fixture_pair", text(&d));
}

#[test]
fn identical_decks_give_an_empty_diff() {
    for name in [
        "valid_full.deck.json",
        "valid_clock.deck.json",
        "valid_toggle.deck.json",
        "valid_timer.deck.json",
        "valid_volume.deck.json",
        "valid_counter.deck.json",
        "valid_no_positions.deck.json",
        "diff_old.deck.json",
    ] {
        let d = load(name);
        assert!(DeckStore::diff(&d, &d.clone()).is_empty(), "{name}");
    }
}

#[test]
fn table_of_edits() {
    let base = load("valid_full.deck.json");
    type Edit = fn(&mut Deck);
    let cases: Vec<(&str, Edit)> = vec![
        ("rename", |d| d.name = "Other".into()),
        ("remove_page", |d| {
            d.pages.pop();
        }),
        ("remove_source", |d| {
            d.sources.remove("games");
        }),
        ("page_title", |d| d.pages[0].title = "Start".into()),
        ("remove_widget", |d| {
            d.pages[0].widgets.pop();
        }),
        ("label", |d| {
            d.pages[0].widgets[0].label = Some("Time".into())
        }),
        ("http_url", |d| {
            if let Some(easydeck_core::deck::Source::HttpPoll { url, .. }) =
                d.sources.get_mut("api")
            {
                *url = "https://example.org/health".into();
            }
        }),
        ("reorder_pages", |d| d.pages.reverse()),
    ];
    let mut lines = Vec::new();
    for (name, edit) in cases {
        let mut changed = base.clone();
        edit(&mut changed);
        let d = DeckStore::diff(&base, &changed);
        assert!(!d.is_empty(), "{name}");
        lines.push(format!("## {name}\n{}", text(&d)));
        // Reverse direction is never empty either.
        assert!(
            !DeckStore::diff(&changed, &base).is_empty(),
            "{name} reverse"
        );
    }
    insta::assert_snapshot!("table_of_edits", lines.join("\n"));
}

#[test]
fn page_switch_and_removal_are_not_risky() {
    let old = load("diff_old.deck.json");
    let mut new = old.clone();
    new.pages[0].widgets.remove(1);
    let d = DeckStore::diff(&old, &new);
    assert_eq!(d.entries.len(), 1);
    assert!(!d.entries[0].runs);
}

proptest! {
    #[test]
    fn diff_of_a_deck_with_itself_is_empty(idx in 0usize..6) {
        let names = ["valid_full", "valid_clock", "valid_toggle", "valid_timer", "valid_volume", "valid_counter"];
        let d = load(&format!("{}.deck.json", names[idx]));
        prop_assert!(DeckStore::diff(&d, &d).is_empty());
    }

    #[test]
    fn moving_one_tile_is_one_moved_entry(x in 0u32..4, y in 0u32..2) {
        let old = load("diff_old.deck.json");
        let mut new = old.clone();
        let before = new.pages[0].widgets[0].position;
        let after = Some(Position { x, y, w: 1, h: 1 });
        new.pages[0].widgets[0].position = after;
        let d = DeckStore::diff(&old, &new);
        if before == after {
            prop_assert!(d.is_empty());
        } else {
            prop_assert_eq!(d.entries.len(), 1);
            prop_assert_eq!(d.entries[0].kind, ChangeKind::Moved);
            prop_assert!(!d.entries[0].runs);
        }
    }

    #[test]
    fn swapping_arguments_keeps_the_entry_count(x in 0u32..4, y in 0u32..2, label in "[a-z]{0,6}") {
        let old = load("diff_old.deck.json");
        let mut new = old.clone();
        new.pages[0].widgets[1].position = Some(Position { x, y, w: 1, h: 1 });
        new.pages[0].widgets[2].label = Some(label);
        prop_assert_eq!(
            DeckStore::diff(&old, &new).entries.len(),
            DeckStore::diff(&new, &old).entries.len()
        );
    }
}

// ---- review fixes ----

use serde_json::{json, Value};

fn base_json() -> Value {
    json!({
        "version": 1,
        "name": "Base",
        "sources": { "api": { "kind": "http_poll", "url": "https://example.com/a" } },
        "pages": [
            { "id": "home", "title": "Home", "grid": { "columns": 4, "rows": 2 }, "widgets": [
                { "id": "run", "type": "launch", "label": "Run",
                  "action": { "type": "launch", "kind": "exe", "path": "a.exe" },
                  "holdAction": { "type": "shell", "command": "one" } },
                { "id": "tog", "type": "toggle",
                  "onAction": { "type": "shell", "command": "on" },
                  "offAction": { "type": "shell", "command": "off" },
                  "on": { "label": "On" }, "off": { "label": "Off" } },
                { "id": "web", "type": "embed", "url": "https://example.com/e" },
                { "id": "note", "type": "text", "text": "hi" }
            ]},
            { "id": "second", "title": "Second", "grid": { "columns": 4, "rows": 2 }, "widgets": [
                { "id": "mac", "type": "macro", "action": { "type": "key_macro", "keys": ["a"] } }
            ]}
        ]
    })
}

fn deck(v: Value) -> Deck {
    serde_json::from_value(v).expect("deck json")
}

fn diff_of(new: Value) -> easydeck_core::deck::DeckDiff {
    DeckStore::diff(&deck(base_json()), &deck(new))
}

#[test]
fn remote_icon_on_a_widget_runs() {
    let mut n = base_json();
    n["pages"][0]["widgets"][3]["icon"] = json!("https://example.com/i.png");
    let d = diff_of(n.clone());
    assert_eq!(d.entries.len(), 1, "{}", text(&d));
    assert!(d.entries[0].runs);
    // Changing the URL runs again; going back to a pack path does not.
    let mut o = base_json();
    o["pages"][0]["widgets"][3]["icon"] = json!("http://example.com/old.png");
    assert!(DeckStore::diff(&deck(o.clone()), &deck(n.clone())).entries[0].runs);
    n["pages"][0]["widgets"][3]["icon"] = json!("icons/local.png");
    let d = DeckStore::diff(&deck(o), &deck(n));
    assert!(!d.entries[0].runs);
}

#[test]
fn remote_toggle_icons_run() {
    for side in ["on", "off"] {
        let mut n = base_json();
        n["pages"][0]["widgets"][1][side]["icon"] = json!("https://example.com/t.png");
        let d = diff_of(n);
        assert_eq!(d.entries.len(), 1, "{side}");
        assert!(d.entries[0].runs, "{side}");
    }
    let mut n = base_json();
    n["pages"][0]["widgets"][1]["on"]["icon"] = json!("icons/on.png");
    assert!(!diff_of(n).entries[0].runs);
}

fn move_widget(v: &mut Value, from: usize, idx: usize, to: usize) {
    let w = v["pages"][from]["widgets"]
        .as_array_mut()
        .unwrap()
        .remove(idx);
    v["pages"][to]["widgets"].as_array_mut().unwrap().push(w);
}

#[test]
fn tile_moved_to_another_page_is_one_moved_entry() {
    let mut n = base_json();
    move_widget(&mut n, 0, 0, 1);
    let d = diff_of(n);
    assert_eq!(d.entries.len(), 1, "{}", text(&d));
    assert_eq!(d.entries[0].kind, ChangeKind::Moved);
    assert!(!d.entries[0].runs);
    assert_eq!(d.entries[0].path, "pages/second/widgets/run");
}

#[test]
fn tile_moved_and_changed_is_one_changed_entry() {
    let mut n = base_json();
    move_widget(&mut n, 0, 0, 1);
    n["pages"][1]["widgets"][1]["label"] = json!("Renamed");
    let d = diff_of(n.clone());
    assert_eq!(d.entries.len(), 1, "{}", text(&d));
    assert_eq!(
        (d.entries[0].kind, d.entries[0].runs),
        (ChangeKind::Changed, false)
    );
    n["pages"][1]["widgets"][1]["action"]["path"] = json!("b.exe");
    let d = diff_of(n);
    assert_eq!(d.entries.len(), 1, "{}", text(&d));
    assert_eq!(
        (d.entries[0].kind, d.entries[0].runs),
        (ChangeKind::Changed, true)
    );
}

#[test]
fn added_page_lists_each_widget_and_its_risk() {
    let mut n = base_json();
    n["pages"].as_array_mut().unwrap().push(json!({
        "id": "extra", "title": "Extra", "grid": { "columns": 2, "rows": 1 }, "widgets": [
            { "id": "x1", "type": "text", "text": "x" },
            { "id": "x2", "type": "launch",
              "action": { "type": "launch", "kind": "exe", "path": "z.exe" } }
        ]
    }));
    let d = diff_of(n);
    assert_eq!(d.entries.len(), 3, "{}", text(&d));
    let risky: Vec<_> = d.risky().map(|e| e.path.as_str()).collect();
    assert_eq!(risky, ["pages/extra/widgets/x2"]);
}

#[test]
fn version_and_schema_changes_are_entries() {
    let mut n = base_json();
    n["$schema"] = json!("https://example.com/schema.json");
    let d = diff_of(n);
    assert_eq!(d.entries.len(), 1);
    assert_eq!(d.entries[0].path, "$schema");
    assert!(!d.entries[0].runs);
    let mut a = deck(base_json());
    let mut b = a.clone();
    b.version = 2;
    let d = DeckStore::diff(&a, &b);
    assert_eq!(d.entries.len(), 1);
    assert_eq!(d.entries[0].path, "version");
    a.version = 2;
    assert!(DeckStore::diff(&a, &b).is_empty());
}

const BAD_ICONS: &[&str] = &[
    "//evil.host/a.png",
    "https:evil.host/a.png",
    r"https:\\evil.host\a.png",
    r"\\host\share\a.png",
    "file://host/a.png",
    "ftp://evil.host/a.png",
    "HTTPS://evil.host/a.png",
    "icons/../x.png",
    "icons//x.png",
    "icons/./x.png",
    "icons/a/",
    "icons/a b.png",
    "x.png",
    "a@b",
];

#[test]
fn non_inert_icon_forms_run_and_glyphs_do_not() {
    for bad in BAD_ICONS {
        let mut n = base_json();
        n["pages"][0]["widgets"][3]["icon"] = json!(bad);
        let d = diff_of(n);
        assert_eq!(d.entries.len(), 1, "{bad}");
        assert!(d.entries[0].runs, "{bad}");
    }
    for ok in ["icons/a/b-c_1.png", "🎮", "play", "icons/x"] {
        let mut n = base_json();
        n["pages"][0]["widgets"][3]["icon"] = json!(ok);
        let d = diff_of(n);
        assert_eq!(d.entries.len(), 1, "{ok}");
        assert!(!d.entries[0].runs, "{ok}");
    }
}

/// One edit to a risky field of `base_json`, chosen by index and a seed.
fn risky_edit(v: &mut Value, which: usize, seed: &str) {
    let url = format!("https://example.org/{seed}");
    let icon = match seed.bytes().next().unwrap_or(0) as usize % (BAD_ICONS.len() + 1) {
        0 => url.clone(),
        n => BAD_ICONS[n - 1].to_string(),
    };
    let cmd = format!("cmd-{seed}");
    match which {
        0 => v["pages"][0]["widgets"][0]["action"] = json!({"type":"shell","command":cmd}),
        1 => v["pages"][0]["widgets"][0]["holdAction"] = json!({"type":"shell","command":cmd}),
        2 => v["pages"][0]["widgets"][1]["onAction"] = json!({"type":"shell","command":cmd}),
        3 => v["pages"][0]["widgets"][1]["offAction"] = json!({"type":"shell","command":cmd}),
        4 => v["pages"][0]["widgets"][2]["url"] = json!(url),
        5 => {
            if let Some(s) = v["sources"].get_mut("api") {
                s["url"] = json!(url);
            }
        }
        6 => v["pages"][0]["widgets"][3]["icon"] = json!(icon),
        7 => v["pages"][0]["widgets"][1]["on"]["icon"] = json!(icon),
        8 => v["pages"][0]["widgets"][1]["off"]["icon"] = json!(icon),
        // Type change: the text tile becomes a launch tile.
        9 => {
            v["pages"][0]["widgets"][3] = json!({"id":"note","type":"launch",
                "action":{"type":"launch","kind":"exe","path":cmd}})
        }
        // Renamed source key keeps the same URL: the new key is an added endpoint.
        _ => {
            if let Some(s) = v["sources"].as_object_mut().unwrap().remove("api") {
                v["sources"]["api2"] = s;
            }
        }
    }
}

proptest! {
    #[test]
    fn every_risky_edit_runs(which in 0usize..11, seed in "[a-z0-9]{1,8}") {
        let old = deck(base_json());
        let mut v = base_json();
        risky_edit(&mut v, which, &seed);
        let new = deck(v);
        let d = DeckStore::diff(&old, &new);
        prop_assert!(!d.is_empty(), "edit {} gave no entry", which);
        prop_assert!(d.risky().count() >= 1, "edit {}: {}", which, text(&d));
        prop_assert!(DeckStore::diff(&new, &new.clone()).is_empty());
    }

    #[test]
    fn two_edits_still_run_and_self_diff_is_empty(a in 0usize..11, b in 0usize..11, seed in "[a-z0-9]{1,8}") {
        let old = deck(base_json());
        let mut v = base_json();
        risky_edit(&mut v, a, &seed);
        risky_edit(&mut v, b, &seed);
        let new = deck(v);
        prop_assert!(DeckStore::diff(&old, &new).risky().count() >= 1);
        prop_assert!(DeckStore::diff(&new, &new).is_empty());
    }

    #[test]
    fn moving_any_tile_across_pages_never_runs(idx in 0usize..4) {
        let old = deck(base_json());
        let mut v = base_json();
        move_widget(&mut v, 0, idx, 1);
        let d = DeckStore::diff(&old, &deck(v));
        prop_assert_eq!(d.entries.len(), 1);
        prop_assert_eq!(d.entries[0].kind, ChangeKind::Moved);
        prop_assert!(!d.entries[0].runs);
    }
}
