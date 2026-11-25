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
