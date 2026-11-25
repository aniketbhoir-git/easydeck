//! Risk item and approval tests (A-8). Fixtures live in `tests/fixtures/` at the repository
//! root. No fixture holds a secret: the paths and URLs are made-up examples.

use easydeck_core::deck::{
    review, risk_items, Action, ApprovalLedger, Deck, DeckStore, RiskKind, WidgetKind,
};
use proptest::prelude::*;
use serde_json::{json, Value};
use std::path::PathBuf;

fn load(name: &str) -> Deck {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../tests/fixtures")
        .join(name);
    DeckStore::load(path).unwrap_or_else(|e| panic!("{name}: {e:?}"))
}

fn deck(v: Value) -> Deck {
    DeckStore::load_str(&v.to_string()).expect("valid deck")
}

fn hashes(d: &Deck) -> Vec<String> {
    let mut h: Vec<String> = risk_items(d).into_iter().map(|i| i.hash).collect();
    h.sort();
    h
}

#[test]
fn fixture_items_by_kind_are_stable() {
    // The snapshot pins the hashes: a change to the canonical form would unapprove every
    // item that users approved before, so it must be a deliberate, reviewed change.
    let items = risk_items(&load("valid_risk.deck.json"));
    insta::assert_json_snapshot!("risk_items_fixture", items);
    let kinds: Vec<_> = items.iter().map(|i| i.kind).collect();
    assert_eq!(kinds.iter().filter(|k| **k == RiskKind::Network).count(), 3);
    assert_eq!(kinds.iter().filter(|k| **k == RiskKind::Launch).count(), 2);
    assert!(items.iter().any(|i| i.system_shortcut));
}

#[test]
fn starter_deck_items_are_launch_and_macro_only() {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../examples/starter.deck.json");
    let items = risk_items(&DeckStore::load(path).unwrap());
    assert_eq!(items.len(), 3);
    assert!(items.iter().all(|i| i.kind != RiskKind::Network));
}

fn base() -> Value {
    json!({
        "version": 1, "name": "p",
        "sources": { "api": { "kind": "http_poll", "url": "https://example.com/a" } },
        "pages": [{ "id": "home", "title": "Home", "grid": { "columns": 4, "rows": 3 }, "widgets": [
            { "id": "l", "type": "launch", "label": "L", "icon": "🎮",
              "action": { "type": "launch", "kind": "exe", "path": "C:/a.exe", "args": ["x"] },
              "holdAction": { "type": "launch", "kind": "url", "target": "https://example.com/h" },
              "position": { "x": 0, "y": 0 } },
            { "id": "m", "type": "macro",
              "action": { "type": "key_macro", "keys": ["1", "ctrl+q"], "cycleMs": 100 },
              "position": { "x": 1, "y": 0 } },
            { "id": "t", "type": "toggle", "icon": "https://example.com/i.png",
              "onAction": { "type": "shell", "command": "lock-screen" },
              "offAction": { "type": "shell", "command": "lock-screen", "args": ["-q"] },
              "on": { "icon": "https://example.com/on.png" },
              "position": { "x": 2, "y": 0 } },
            { "id": "e", "type": "embed", "url": "https://example.com/e" }
        ]}]
    })
}

/// One edit to something that runs, chosen by index. Every edit must change a hash.
fn risky_edit(v: &mut Value, which: usize, s: &str) {
    match which {
        0 => v["pages"][0]["widgets"][0]["action"]["path"] = json!(format!("C:/x-{s}.exe")),
        1 => v["pages"][0]["widgets"][0]["action"]["args"] = json!([format!("x-{s}")]),
        2 => {
            v["pages"][0]["widgets"][0]["holdAction"]["target"] =
                json!(format!("https://example.com/x-{s}"))
        }
        3 => v["pages"][0]["widgets"][1]["action"]["keys"] = json!([format!("x-{s}")]),
        4 => v["pages"][0]["widgets"][1]["action"]["cycleMs"] = json!(101 + s.len()),
        5 => v["pages"][0]["widgets"][2]["onAction"]["command"] = json!(format!("c-x-{s}")),
        6 => v["pages"][0]["widgets"][2]["offAction"]["args"] = json!([format!("x-{s}")]),
        7 => v["pages"][0]["widgets"][2]["icon"] = json!(format!("https://example.com/x-{s}.png")),
        8 => {
            v["pages"][0]["widgets"][2]["on"]["icon"] =
                json!(format!("https://example.com/x-{s}.png"))
        }
        9 => v["pages"][0]["widgets"][3]["url"] = json!(format!("https://example.com/x-{s}")),
        _ => v["sources"]["api"]["url"] = json!(format!("https://example.com/x-{s}")),
    }
}

/// Rebuilds every object with its keys in a seed-dependent order.
fn shuffle(v: &Value, seed: u64) -> Value {
    match v {
        Value::Object(m) => {
            let mut keys: Vec<&String> = m.keys().collect();
            keys.sort_by_key(|k| {
                k.bytes()
                    .fold(seed, |a, b| a.wrapping_mul(31).wrapping_add(b as u64))
            });
            let mut s = String::from("{");
            for (i, k) in keys.iter().enumerate() {
                if i > 0 {
                    s.push(',');
                }
                s += &format!("{}:{}", Value::String((*k).clone()), shuffle(&m[*k], seed));
            }
            s.push('}');
            serde_json::from_str(&s).unwrap()
        }
        Value::Array(a) => Value::Array(a.iter().map(|x| shuffle(x, seed)).collect()),
        other => other.clone(),
    }
}

proptest! {
    #[test]
    fn changing_any_risky_field_changes_a_hash_and_unapproves(
        which in 0usize..11, seed in "[a-z0-9]{1,8}"
    ) {
        let old = deck(base());
        let mut v = base();
        risky_edit(&mut v, which, &seed);
        let new = deck(v);
        let mut ledger = ApprovalLedger::new();
        ledger.approve(&risk_items(&old));
        prop_assert_ne!(hashes(&old), hashes(&new), "edit {}", which);
        prop_assert!(review(&new, &ledger).iter().any(|r| !r.approved), "edit {}", which);
        // Approving the new deck's items approves all of it.
        ledger.approve(&risk_items(&new));
        prop_assert!(review(&new, &ledger).iter().all(|r| r.approved));
    }

    #[test]
    fn inert_changes_keep_every_hash(
        x in 0u32..4, y in 1u32..3, label in "[a-z ]{0,12}", id in "[a-z][a-z0-9]{0,8}",
        title in "[A-Za-z ]{1,10}", rename in "[A-Za-z ]{1,10}", glyph in "[a-z]{1,6}"
    ) {
        let old = deck(base());
        let mut v = base();
        v["name"] = json!(rename);
        v["pages"][0]["title"] = json!(title);
        let w = &mut v["pages"][0]["widgets"][0];
        w["position"] = json!({ "x": x, "y": y });
        w["label"] = json!(label);
        w["id"] = json!(format!("w-{id}"));
        w["icon"] = json!(glyph);
        let new = deck(v);
        prop_assert_eq!(hashes(&old), hashes(&new));
        let mut ledger = ApprovalLedger::new();
        ledger.approve(&risk_items(&old));
        prop_assert!(review(&new, &ledger).iter().all(|r| r.approved));
    }

    #[test]
    fn key_order_does_not_change_hashes(seed in 0u64..1000) {
        prop_assert_eq!(hashes(&deck(base())), hashes(&deck(shuffle(&base(), seed))));
    }
}

#[test]
fn a_deck_on_disk_with_one_exe_changed_locks_only_that_tile() {
    let original = load("valid_risk.deck.json");
    let mut ledger = ApprovalLedger::new();
    ledger.approve(&risk_items(&original));
    let mut edited = original.clone();
    if let WidgetKind::Launch {
        hold_action: Some(Action::Launch { path, .. }),
        ..
    } = &mut edited.pages[0].widgets[0].kind
    {
        *path = Some("C:/Games/evil.exe".into());
    }
    let locked: Vec<_> = review(&edited, &ledger)
        .into_iter()
        .filter(|r| !r.approved)
        .flat_map(|r| r.item.locations)
        .collect();
    assert_eq!(locked, ["pages/home/widgets/game/holdAction"]);
}
