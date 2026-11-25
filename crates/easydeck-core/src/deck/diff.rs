//! Deck diff (R5-1, SPEC §5, §6.3): what a new deck adds, removes or changes against an old
//! one. There is no `apply`: an approved deck replaces the old one.
//!
//! Pages match by id, sources by key, widgets by id across the whole deck (see `Matcher`). A widget gives at most one entry, so a
//! widget that moved (even to another page) and changed its label is one `Changed` entry. Each entry has a path, a
//! short summary and `runs`, which tells the risk review (A-8) that the entry adds or changes
//! something that runs a program or reaches the network.

use super::{risk, Action, Deck, DeckStore, Page, Source, Widget};
use serde::Serialize;
use serde_json::Value;
use std::collections::BTreeSet;

/// What happened to the thing at `path`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ChangeKind {
    Added,
    Removed,
    Changed,
    /// Only the position or the page of a widget changed.
    Moved,
}

/// The kind of thing an entry is about.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum DiffScope {
    Deck,
    Page,
    Widget,
    Source,
}

/// One difference between two decks.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct DiffEntry {
    pub kind: ChangeKind,
    pub scope: DiffScope,
    /// Id path, for example `pages/home/widgets/play`, `sources/weather` or `settings`.
    pub path: String,
    /// One short human sentence.
    pub summary: String,
    /// True when the new side adds or changes a launch, key macro, shell action, an
    /// `http_poll` URL, an `embed` URL or a remote icon URL. Always false for `Removed` and `Moved`.
    pub runs: bool,
}

/// The ordered list of differences. Empty when the decks match.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize)]
pub struct DeckDiff {
    pub entries: Vec<DiffEntry>,
}

impl DeckDiff {
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Entries that add or change something that runs, for the review screen to show first.
    pub fn risky(&self) -> impl Iterator<Item = &DiffEntry> {
        self.entries.iter().filter(|e| e.runs)
    }
}

impl DeckStore {
    /// `DeckStore::diff(old, new)` of SPEC §5.
    pub fn diff(old: &Deck, new: &Deck) -> DeckDiff {
        diff(old, new)
    }
}

/// Compare `old` with `new`. Order: name, settings, sources, pages (new order, then removed).
pub fn diff(old: &Deck, new: &Deck) -> DeckDiff {
    let mut out = Vec::new();

    if old.name != new.name {
        out.push(entry(
            ChangeKind::Changed,
            DiffScope::Deck,
            "name",
            format!("deck renamed from \"{}\" to \"{}\"", old.name, new.name),
            false,
        ));
    }
    for (key, o, n) in [
        ("version", old.version.to_string(), new.version.to_string()),
        (
            "$schema",
            old.schema.clone().unwrap_or_default(),
            new.schema.clone().unwrap_or_default(),
        ),
    ] {
        if o != n {
            out.push(entry(
                ChangeKind::Changed,
                DiffScope::Deck,
                key,
                format!("{key} changed from \"{o}\" to \"{n}\""),
                false,
            ));
        }
    }
    let (so, sn) = (to_value(&old.settings), to_value(&new.settings));
    let keys = changed_keys(&so, &sn, &[]);
    if !keys.is_empty() {
        out.push(entry(
            ChangeKind::Changed,
            DiffScope::Deck,
            "settings",
            format!("settings changed: {}", keys.join(", ")),
            false,
        ));
    }

    diff_sources(old, new, &mut out);
    diff_pages(old, new, &mut out);
    DeckDiff { entries: out }
}

fn entry(
    kind: ChangeKind,
    scope: DiffScope,
    path: impl Into<String>,
    summary: String,
    runs: bool,
) -> DiffEntry {
    DiffEntry {
        kind,
        scope,
        path: path.into(),
        summary,
        runs,
    }
}

fn to_value<T: Serialize>(t: &T) -> Value {
    serde_json::to_value(t).expect("deck types serialize")
}

/// Sorted keys whose values differ between two JSON objects (`null` counts as an empty object).
fn changed_keys(a: &Value, b: &Value, skip: &[&str]) -> Vec<String> {
    let empty = serde_json::Map::new();
    let ma = a.as_object().unwrap_or(&empty);
    let mb = b.as_object().unwrap_or(&empty);
    let all: BTreeSet<&String> = ma.keys().chain(mb.keys()).collect();
    all.into_iter()
        .filter(|k| !skip.contains(&k.as_str()) && ma.get(*k) != mb.get(*k))
        .cloned()
        .collect()
}

// ---- sources ----

fn source_kind(s: &Source) -> &'static str {
    match s {
        Source::SystemStats { .. } => "system stats",
        Source::HttpPoll { .. } => "http poll",
        Source::GameWatch { .. } => "game watch",
    }
}

/// What a source reaches, as the risk scan finds it.
fn source_payload(s: &Source) -> Option<Value> {
    risk::source_risk(s)
}

fn diff_sources(old: &Deck, new: &Deck, out: &mut Vec<DiffEntry>) {
    for (id, n) in &new.sources {
        let path = format!("sources/{id}");
        match old.sources.get(id) {
            None => out.push(entry(
                ChangeKind::Added,
                DiffScope::Source,
                path,
                format!("added {} source \"{id}\"", source_kind(n)),
                source_payload(n).is_some(),
            )),
            Some(o) if o != n => {
                let keys = changed_keys(&to_value(o), &to_value(n), &[]);
                out.push(entry(
                    ChangeKind::Changed,
                    DiffScope::Source,
                    path,
                    format!("source \"{id}\" changed: {}", keys.join(", ")),
                    source_payload(n).is_some() && source_payload(o) != source_payload(n),
                ));
            }
            Some(_) => {}
        }
    }
    for (id, o) in &old.sources {
        if !new.sources.contains_key(id) {
            out.push(entry(
                ChangeKind::Removed,
                DiffScope::Source,
                format!("sources/{id}"),
                format!("removed {} source \"{id}\"", source_kind(o)),
                false,
            ));
        }
    }
}

// ---- pages and widgets ----

fn diff_pages(old: &Deck, new: &Deck, out: &mut Vec<DiffEntry>) {
    let matcher = Matcher::new(old, new);
    for np in &new.pages {
        let path = format!("pages/{}", np.id);
        let old_page = old.pages.iter().find(|p| p.id == np.id);
        match old_page {
            None => out.push(entry(
                ChangeKind::Added,
                DiffScope::Page,
                path.clone(),
                format!(
                    "added page \"{}\" with {} widget(s)",
                    np.title,
                    np.widgets.len()
                ),
                // The widget entries below carry `runs`.
                false,
            )),
            Some(op) => diff_page_fields(op, np, &path, out),
        }
        for nw in &np.widgets {
            let wpath = format!("{path}/widgets/{}", nw.id);
            match matcher.old_of(&np.id, &nw.id) {
                None => out.push(entry(
                    ChangeKind::Added,
                    DiffScope::Widget,
                    wpath,
                    format!("added {} tile \"{}\"", type_name(nw), name_of(nw)),
                    !widget_payload(nw).is_empty(),
                )),
                Some((opage, ow)) => {
                    if ow != nw || opage != np.id {
                        out.push(widget_change(opage, ow, &np.id, nw, wpath));
                    }
                }
            }
        }
        // Tiles of a page both decks hold that no tile of the new deck continues. Tiles of
        // a removed page are covered by the page entry.
        if let Some(op) = old_page {
            for ow in &op.widgets {
                if !matcher.continues(&op.id, &ow.id) {
                    out.push(entry(
                        ChangeKind::Removed,
                        DiffScope::Widget,
                        format!("{path}/widgets/{}", ow.id),
                        format!("removed {} tile \"{}\"", type_name(ow), name_of(ow)),
                        false,
                    ));
                }
            }
        }
    }
    for op in &old.pages {
        if !new.pages.iter().any(|p| p.id == op.id) {
            out.push(entry(
                ChangeKind::Removed,
                DiffScope::Page,
                format!("pages/{}", op.id),
                format!(
                    "removed page \"{}\" with {} widget(s)",
                    op.title,
                    op.widgets.len()
                ),
                false,
            ));
        }
    }
    // Relative order of the pages both decks hold.
    let common = |d: &Deck, other: &Deck| -> Vec<String> {
        d.pages
            .iter()
            .filter(|p| other.pages.iter().any(|q| q.id == p.id))
            .map(|p| p.id.clone())
            .collect()
    };
    if common(old, new) != common(new, old) {
        out.push(entry(
            ChangeKind::Changed,
            DiffScope::Deck,
            "pages",
            "page order changed".into(),
            false,
        ));
    }
}

/// Pairs the widgets of two decks. A widget id that appears once in each deck matches across
/// pages, so a tile moved to another page is one entry. Validation only requires ids to be
/// unique on a page; an id that repeats across pages matches within its page only.
struct Matcher<'a> {
    old: &'a Deck,
    new: &'a Deck,
    global: BTreeSet<&'a str>,
}

impl<'a> Matcher<'a> {
    fn new(old: &'a Deck, new: &'a Deck) -> Self {
        let count = |d: &Deck, id: &str| {
            d.pages
                .iter()
                .flat_map(|p| &p.widgets)
                .filter(|w| w.id == id)
                .count()
        };
        let global = new
            .pages
            .iter()
            .flat_map(|p| &p.widgets)
            .map(|w| w.id.as_str())
            .filter(|id| count(old, id) == 1 && count(new, id) == 1)
            .collect();
        Matcher { old, new, global }
    }

    /// The old widget (and its page id) that the new widget `id` on `page` continues.
    fn old_of(&self, page: &str, id: &str) -> Option<(&'a str, &'a Widget)> {
        if self.global.contains(id) {
            self.old.pages.iter().find_map(|p| {
                p.widgets
                    .iter()
                    .find(|w| w.id == id)
                    .map(|w| (p.id.as_str(), w))
            })
        } else {
            let p = self.old.pages.iter().find(|p| p.id == page)?;
            let w = p.widgets.iter().find(|w| w.id == id)?;
            Some((p.id.as_str(), w))
        }
    }

    /// Whether the new deck holds a widget that continues old widget `id` of `page`.
    fn continues(&self, page: &str, id: &str) -> bool {
        self.global.contains(id)
            || self
                .new
                .pages
                .iter()
                .find(|p| p.id == page)
                .is_some_and(|p| p.widgets.iter().any(|w| w.id == id))
    }
}

fn diff_page_fields(op: &Page, np: &Page, path: &str, out: &mut Vec<DiffEntry>) {
    let keys = changed_keys(&to_value(op), &to_value(np), &["widgets"]);
    if !keys.is_empty() {
        out.push(entry(
            ChangeKind::Changed,
            DiffScope::Page,
            path,
            format!("page \"{}\" changed: {}", np.title, keys.join(", ")),
            false,
        ));
    }
}

fn type_name(w: &Widget) -> String {
    to_value(w)["type"].as_str().unwrap_or("?").to_string()
}

fn name_of(w: &Widget) -> String {
    w.label.clone().unwrap_or_else(|| w.id.clone())
}

fn widget_change(opage: &str, ow: &Widget, npage: &str, nw: &Widget, path: String) -> DiffEntry {
    let (vo, vn) = (to_value(ow), to_value(nw));
    let keys = changed_keys(&vo, &vn, &[]);
    let name = name_of(nw);
    let page_moved = opage != npage;
    let from_page = if page_moved {
        format!("page \"{opage}\" ")
    } else {
        String::new()
    };
    if keys.is_empty() || keys == ["position"] {
        return entry(
            ChangeKind::Moved,
            DiffScope::Widget,
            path,
            format!(
                "moved tile \"{name}\" from {from_page}{} to {}",
                pos(ow),
                pos(nw)
            ),
            false,
        );
    }
    let mut parts: Vec<String> = Vec::new();
    if page_moved {
        parts.push(format!("moved from page \"{opage}\" to page \"{npage}\""));
    }
    parts.extend(keys.iter().map(|k| match k.as_str() {
        "position" => format!("moved {} -> {}", pos(ow), pos(nw)),
        "action" | "holdAction" | "onAction" | "offAction" => {
            format!("{k}: {} -> {}", action_text(&vo[k]), action_text(&vn[k]))
        }
        other => other.to_string(),
    }));
    let runs = !widget_payload(nw).is_empty() && widget_payload(ow) != widget_payload(nw);
    entry(
        ChangeKind::Changed,
        DiffScope::Widget,
        path,
        format!(
            "changed {} tile \"{name}\": {}",
            type_name(nw),
            parts.join("; ")
        ),
        runs,
    )
}

fn pos(w: &Widget) -> String {
    match &w.position {
        Some(p) => format!("({},{} {}x{})", p.x, p.y, p.w, p.h),
        None => "auto".to_string(),
    }
}

/// A short text for an action value, `none` when absent.
fn action_text(v: &Value) -> String {
    if v.is_null() {
        return "none".into();
    }
    match serde_json::from_value::<Action>(v.clone()) {
        Ok(Action::Launch {
            kind,
            path,
            app_id,
            target,
            ..
        }) => {
            let what = path
                .or(target)
                .or(app_id.map(|i| i.to_string()))
                .unwrap_or_default();
            format!("launch {} {what}", to_value(&kind).as_str().unwrap_or("?"))
                .trim_end()
                .to_string()
        }
        Ok(Action::KeyMacro { keys, .. }) => format!("key macro [{}]", keys.join("+")),
        Ok(Action::Shell { command, .. }) => format!("shell {command}"),
        Ok(Action::Page { page }) => format!("go to page {page}"),
        Err(_) => "?".into(),
    }
}

/// What a widget runs or reaches, as the risk scan (`risk.rs`) finds it: its launch, key macro
/// and shell actions, its `embed` URL and any icon that is not inert (a network endpoint,
/// SPEC 4.3). Page switches run nothing. A change here is a change to what runs.
fn widget_payload(w: &Widget) -> Vec<Value> {
    risk::widget_risks(w)
        .into_iter()
        .map(|f| f.subject)
        .collect()
}
