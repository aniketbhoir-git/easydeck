//! Risk items (SPEC §4.3, §0 rule 2): the parts of a deck that can act outside the deck
//! window, and the stable identity of each one.
//!
//! A risk item is a launch, key macro or shell action, or a network endpoint (an `http_poll`
//! URL, an `embed` URL, a remote icon URL). Its identity is the SHA-256 of a canonical form of
//! *exactly what would run or be reached*. The canonical form holds no widget id, page, label
//! or position, so moving or renaming a tile keeps its approval, and any change to the
//! action or URL gives a new hash. The deck diff (`diff.rs`) uses the same scan, so the two
//! cannot disagree about what runs.

use super::sha256::sha256_hex;
use super::{Action, Deck, LaunchKind, Source, Widget, WidgetKind};
use serde::Serialize;
use serde_json::Value;
use ts_rs::TS;

/// What a risk item does, for the review screen to group by.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, TS)]
#[serde(rename_all = "snake_case")]
pub enum RiskKind {
    /// Starts a program, game or link.
    Launch,
    /// Sends keys.
    KeyMacro,
    /// Runs an allow-list command.
    Shell,
    /// Reaches a URL: an `http_poll` source, an `embed` or a remote icon.
    Network,
}

/// One risk item of a deck.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(rename_all = "camelCase")]
pub struct RiskItem {
    pub kind: RiskKind,
    /// SHA-256 over the canonical form, 64 lowercase hex characters.
    pub hash: String,
    /// One line that says exactly what runs or what is reached.
    pub summary: String,
    /// True for a key macro that uses the Windows key or `alt+tab`.
    pub system_shortcut: bool,
    /// Where the deck uses this item, as id paths such as `pages/home/widgets/play/action` or
    /// `sources/weather`. One item can sit in several places; it needs one approval.
    pub locations: Vec<String>,
}

/// A risk thing found in a widget: the field that holds it and its canonical subject.
pub(crate) struct Found {
    pub field: &'static str,
    pub subject: Value,
}

fn to_value<T: Serialize>(t: &T) -> Value {
    serde_json::to_value(t).expect("deck types serialize")
}

fn endpoint(use_: &str, url: &str) -> Value {
    serde_json::json!({ "type": use_, "url": url })
}

/// What an action runs, or `None` for a page switch, which runs nothing outside the window.
pub(crate) fn action_subject(a: &Action) -> Option<Value> {
    (!matches!(a, Action::Page { .. })).then(|| to_value(a))
}

/// What a source reaches: only `http_poll` reaches the network. Interval and timeout are not
/// part of the identity: the approval covers the endpoint.
pub(crate) fn source_risk(s: &Source) -> Option<Value> {
    match s {
        Source::HttpPoll { url, .. } => Some(endpoint("http_poll", url)),
        _ => None,
    }
}

/// True only when an icon value cannot name a network endpoint (SPEC 4.3). This is an
/// allow-list: a browser or Windows fetches many forms (`//host/a.png`, `https:host/a.png`,
/// `https:\\evil.host\a.png`, `\\host\share\a.png`, `file://host/a.png`, `ftp://...`, any case),
/// so a deny-list on `http(s)://` is not safe. An icon is inert only if it is
/// (a) a pack path: `icons/` plus `[A-Za-z0-9_./-]`, with no empty, `.` or `..` segment, or
/// (b) plain glyph text (an emoji or a short word) with none of `/ \ : . % @`, and none of
/// the full-width or small-form look-alikes of those that NFKC turns into them.
/// Every other value counts as a network endpoint. The validator (`validate.rs`) accepts
/// only these two forms and a well-formed `https://` URL.
pub(crate) fn icon_is_inert(icon: &str) -> bool {
    if let Some(rest) = icon.strip_prefix("icons/") {
        return !rest.is_empty()
            && rest
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || matches!(c, '_' | '.' | '/' | '-'))
            && !rest
                .split('/')
                .any(|seg| seg.is_empty() || seg == "." || seg == "..");
    }
    !icon.chars().any(is_url_syntax_char)
}

/// True when an icon value has one of the three forms the validator allows: a pack path, plain
/// glyph text (both `icon_is_inert`), or a well-formed `https://` URL. The URL must be ASCII
/// without spaces, control characters or backslashes, and its host must be dot-separated
/// labels of letters, digits and `-`, with an optional numeric port and no user info. Every
/// other value, including `http://`, `//host/x`, `https:host`, `file://` and `\\host\x`, is
/// refused, so the only way to name a network icon is one plain URL that the risk scan lists.
pub(crate) fn icon_form_is_valid(icon: &str) -> bool {
    if icon_is_inert(icon) {
        return true;
    }
    let Some(rest) = icon.strip_prefix("https://") else {
        return false;
    };
    if !rest.bytes().all(|b| b.is_ascii_graphic() && b != b'\\') {
        return false;
    }
    let authority = rest.split(['/', '?', '#']).next().unwrap_or("");
    let (host, port) = match authority.split_once(':') {
        Some((h, p)) => (h, Some(p)),
        None => (authority, None),
    };
    port.is_none_or(|p| !p.is_empty() && p.len() <= 5 && p.bytes().all(|b| b.is_ascii_digit()))
        && !host.is_empty()
        && host.split('.').all(|label| {
            !label.is_empty()
                && !label.starts_with('-')
                && !label.ends_with('-')
                && label
                    .bytes()
                    .all(|b| b.is_ascii_alphanumeric() || b == b'-')
        })
}

/// A character that can make an icon a URL or a path, or that NFKC maps to one: the ASCII
/// set, the dot leaders U+2024..U+2026, the small forms U+FE50..U+FE6F and the full-width
/// forms U+FF01..U+FF5E, and the two slash look-alikes U+2044 and U+2215.
fn is_url_syntax_char(c: char) -> bool {
    matches!(c, '/' | '\\' | ':' | '.' | '%' | '@')
        || matches!(c, '\u{2024}'..='\u{2026}' | '\u{2044}' | '\u{2215}')
        || matches!(c, '\u{FE50}'..='\u{FE6F}' | '\u{FF01}'..='\u{FF5E}')
}

/// The risk things of one widget, in field order: actions, `embed` URL, icons.
pub(crate) fn widget_risks(w: &Widget) -> Vec<Found> {
    let mut out = Vec::new();
    let mut act = |field: &'static str, a: &Action| {
        if let Some(subject) = action_subject(a) {
            out.push(Found { field, subject });
        }
    };
    match &w.kind {
        WidgetKind::Launch {
            action,
            hold_action,
            ..
        }
        | WidgetKind::Macro {
            action,
            hold_action,
        } => {
            act("action", action);
            if let Some(h) = hold_action {
                act("holdAction", h);
            }
        }
        WidgetKind::Toggle {
            on_action,
            off_action,
            ..
        } => {
            act("onAction", on_action);
            if let Some(o) = off_action {
                act("offAction", o);
            }
        }
        _ => {}
    }
    if let WidgetKind::Embed { url, .. } = &w.kind {
        out.push(Found {
            field: "url",
            subject: endpoint("embed", url),
        });
    }
    let mut icons: Vec<(&'static str, &String)> = w.icon.iter().map(|i| ("icon", i)).collect();
    if let WidgetKind::Toggle { on, off, .. } = &w.kind {
        icons.extend(
            on.iter()
                .filter_map(|l| l.icon.as_ref())
                .map(|i| ("on/icon", i)),
        );
        icons.extend(
            off.iter()
                .filter_map(|l| l.icon.as_ref())
                .map(|i| ("off/icon", i)),
        );
    }
    out.extend(
        icons
            .into_iter()
            .filter(|(_, u)| !icon_is_inert(u))
            .map(|(field, u)| Found {
                field,
                subject: endpoint("icon", u),
            }),
    );
    out
}

/// Canonical JSON: object keys sorted, no whitespace. serde_json's own map order depends on a
/// feature flag, so the order is fixed here.
pub(crate) fn canonical_json(v: &Value) -> String {
    fn write(v: &Value, out: &mut String) {
        match v {
            Value::Object(map) => {
                let mut keys: Vec<&String> = map.keys().collect();
                keys.sort();
                out.push('{');
                for (i, k) in keys.into_iter().enumerate() {
                    if i > 0 {
                        out.push(',');
                    }
                    out.push_str(&Value::String(k.clone()).to_string());
                    out.push(':');
                    write(&map[k], out);
                }
                out.push('}');
            }
            Value::Array(items) => {
                out.push('[');
                for (i, item) in items.iter().enumerate() {
                    if i > 0 {
                        out.push(',');
                    }
                    write(item, out);
                }
                out.push(']');
            }
            other => out.push_str(&other.to_string()),
        }
    }
    let mut s = String::new();
    write(v, &mut s);
    s
}

/// The hash of a risk subject: SHA-256 over a version tag, a NUL and the canonical JSON.
pub(crate) fn subject_hash(subject: &Value) -> String {
    let mut bytes = b"easydeck.risk.v1\0".to_vec();
    bytes.extend_from_slice(canonical_json(subject).as_bytes());
    sha256_hex(&bytes)
}

fn kind_of(subject: &Value) -> RiskKind {
    match subject["type"].as_str() {
        Some("launch") => RiskKind::Launch,
        Some("key_macro") => RiskKind::KeyMacro,
        Some("shell") => RiskKind::Shell,
        _ => RiskKind::Network,
    }
}

/// The risk hash of an action, or `None` for a page switch (it runs nothing outside the
/// window, so it needs no approval). This is the hash the gate checks before a run.
pub fn action_hash(a: &Action) -> Option<String> {
    action_subject(a).map(|s| subject_hash(&s))
}

/// The hash of a source's endpoint, or `None` for a source that reaches no network. The
/// source runner starts an `http_poll` source only when the ledger holds this hash.
pub fn source_hash(s: &Source) -> Option<String> {
    source_risk(s).map(|v| subject_hash(&v))
}

/// A short description for the review screen: exactly what would run or be reached.
pub fn action_summary(a: &Action) -> String {
    match a {
        Action::Launch {
            kind,
            app_id,
            path,
            args,
            fallback,
            target,
        } => {
            let mut s = match kind {
                LaunchKind::Steam => format!("launch Steam app {}", app_id.unwrap_or_default()),
                LaunchKind::Exe => format!("start program {}", path.as_deref().unwrap_or("?")),
                LaunchKind::Uri => format!("open URI {}", target.as_deref().unwrap_or("?")),
                LaunchKind::Url => format!("open link {}", target.as_deref().unwrap_or("?")),
            };
            if let Some(args) = args.as_ref().filter(|a| !a.is_empty()) {
                s += &format!(" with arguments {}", to_value(args));
            }
            if let Some(f) = fallback {
                s += &format!(", fallback {f}");
            }
            s
        }
        Action::KeyMacro {
            keys,
            cycle_ms,
            key_gap_ms,
            hold_ms,
            toggle,
        } => {
            let mut s = format!("send keys {}", keys.join(", "));
            if let Some(c) = cycle_ms.filter(|c| *c > 0) {
                s += &format!(", repeat every {c} ms");
            }
            if let Some(g) = key_gap_ms {
                s += &format!(", {g} ms between keys");
            }
            if let Some(h) = hold_ms {
                s += &format!(", hold {h} ms");
            }
            if *toggle == Some(true) {
                s += ", a tap starts and stops the loop";
            }
            s
        }
        Action::Shell { command, args } => {
            let mut s = format!("run shell command \"{command}\"");
            if let Some(args) = args.as_ref().filter(|a| !a.is_empty()) {
                s += &format!(" with arguments {}", to_value(args));
            }
            s
        }
        Action::Page { page } => format!("go to page {page}"),
    }
}

/// True for a key macro that has an entry with the Windows key or `alt+tab`.
pub fn is_system_shortcut(a: &Action) -> bool {
    let Action::KeyMacro { keys, .. } = a else {
        return false;
    };
    keys.iter().any(|entry| {
        let parts: Vec<String> = entry.split('+').map(|p| p.trim().to_lowercase()).collect();
        let has = |names: &[&str]| parts.iter().any(|p| names.contains(&p.as_str()));
        has(&["win", "windows", "lwin", "rwin", "meta", "super", "cmd"])
            || (has(&["alt", "lalt", "ralt"]) && has(&["tab"]))
    })
}

fn describe(subject: &Value) -> (String, bool) {
    if let Ok(a) = serde_json::from_value::<Action>(subject.clone()) {
        return (action_summary(&a), is_system_shortcut(&a));
    }
    let url = subject["url"].as_str().unwrap_or("?");
    let text = match subject["type"].as_str() {
        Some("http_poll") => format!("poll {url} on a timer"),
        Some("embed") => format!("show the web page {url} (it can go elsewhere from there)"),
        _ => format!("load the icon from {url}"),
    };
    (text, false)
}

/// The risk items of a deck: sources first, then widgets in page order. Items with the same
/// hash (the same thing in two places) are one item with several locations.
pub fn risk_items(deck: &Deck) -> Vec<RiskItem> {
    let mut items: Vec<RiskItem> = Vec::new();
    let mut add = |subject: Value, location: String| {
        let hash = subject_hash(&subject);
        match items.iter_mut().find(|i| i.hash == hash) {
            Some(item) => item.locations.push(location),
            None => {
                let (summary, system_shortcut) = describe(&subject);
                items.push(RiskItem {
                    kind: kind_of(&subject),
                    hash,
                    summary,
                    system_shortcut,
                    locations: vec![location],
                });
            }
        }
    };
    for (id, source) in &deck.sources {
        if let Some(subject) = source_risk(source) {
            add(subject, format!("sources/{id}"));
        }
    }
    for page in &deck.pages {
        for w in &page.widgets {
            for f in widget_risks(w) {
                add(
                    f.subject,
                    format!("pages/{}/widgets/{}/{}", page.id, w.id, f.field),
                );
            }
        }
    }
    items
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::deck::DeckStore;

    fn deck(widgets: &str, sources: &str) -> Deck {
        DeckStore::load_str(&format!(
            r#"{{"version":1,"name":"t","sources":{{{sources}}},"pages":[{{"id":"home","title":"H",
            "grid":{{"columns":4,"rows":2}},"widgets":[{widgets}]}}]}}"#
        ))
        .unwrap()
    }

    #[test]
    fn lists_every_kind() {
        let d = deck(
            r#"{"id":"a","type":"launch","action":{"type":"launch","kind":"exe","path":"C:/x.exe","args":["-w"]}},
               {"id":"b","type":"macro","action":{"type":"key_macro","keys":["win+d"]}},
               {"id":"c","type":"toggle","onAction":{"type":"shell","command":"lock-screen"},
                "offAction":{"type":"page","page":"home"}},
               {"id":"e","type":"embed","url":"https://example.com/p","icon":"https://example.com/i.png"}"#,
            r#""w":{"kind":"http_poll","url":"https://example.com/api"}"#,
        );
        let items = risk_items(&d);
        let kinds: Vec<_> = items.iter().map(|i| i.kind).collect();
        assert_eq!(
            kinds,
            [
                RiskKind::Network,
                RiskKind::Launch,
                RiskKind::KeyMacro,
                RiskKind::Shell,
                RiskKind::Network,
                RiskKind::Network
            ]
        );
        // The page action is no risk item.
        assert!(items
            .iter()
            .all(|i| !i.locations.iter().any(|l| l.ends_with("offAction"))));
        assert!(items[2].system_shortcut);
        assert!(!items[1].system_shortcut);
        assert_eq!(items[0].locations, ["sources/w"]);
        assert_eq!(
            items[1].summary,
            "start program C:/x.exe with arguments [\"-w\"]"
        );
    }

    #[test]
    fn same_item_in_two_places_is_one_item() {
        let d = deck(
            r#"{"id":"a","type":"macro","action":{"type":"key_macro","keys":["1"]}},
               {"id":"b","type":"toggle","onAction":{"type":"key_macro","keys":["1"]}}"#,
            "",
        );
        let items = risk_items(&d);
        assert_eq!(items.len(), 1);
        assert_eq!(
            items[0].locations,
            [
                "pages/home/widgets/a/action",
                "pages/home/widgets/b/onAction"
            ]
        );
    }

    #[test]
    fn hash_ignores_key_order_and_position_and_label() {
        let a = deck(
            r#"{"id":"a","type":"launch","label":"A","position":{"x":0,"y":0},
                "action":{"type":"launch","kind":"exe","path":"C:/x.exe","args":["1"]}}"#,
            "",
        );
        let b = deck(
            r#"{"position":{"y":1,"x":3},"label":"Renamed","type":"launch","id":"zz",
                "action":{"args":["1"],"path":"C:/x.exe","kind":"exe","type":"launch"}}"#,
            "",
        );
        assert_eq!(risk_items(&a)[0].hash, risk_items(&b)[0].hash);
    }

    #[test]
    fn hash_changes_with_what_runs() {
        let base = |action: &str| {
            risk_items(&deck(
                &format!(r#"{{"id":"a","type":"launch","action":{action}}}"#),
                "",
            ))[0]
                .hash
                .clone()
        };
        let h = base(r#"{"type":"launch","kind":"exe","path":"C:/x.exe"}"#);
        assert_ne!(
            h,
            base(r#"{"type":"launch","kind":"exe","path":"C:/y.exe"}"#)
        );
        assert_ne!(
            h,
            base(r#"{"type":"launch","kind":"exe","path":"C:/x.exe","args":["-a"]}"#)
        );
        assert_ne!(
            h,
            base(r#"{"type":"launch","kind":"exe","path":"C:/X.exe"}"#)
        );
        assert_eq!(h.len(), 64);
    }

    #[test]
    fn url_use_is_part_of_identity() {
        let d = deck(
            r#"{"id":"e","type":"embed","url":"https://example.com/a"}"#,
            r#""w":{"kind":"http_poll","url":"https://example.com/a"}"#,
        );
        let items = risk_items(&d);
        assert_eq!(items.len(), 2);
        assert_ne!(items[0].hash, items[1].hash);
    }

    #[test]
    fn poll_interval_does_not_change_the_endpoint_hash() {
        let h = |extra: &str| {
            let d = deck(
                "",
                &format!(r#""w":{{"kind":"http_poll","url":"https://e.com/a"{extra}}}"#),
            );
            source_hash(&d.sources["w"]).unwrap()
        };
        assert_eq!(h(""), h(r#","intervalSeconds":30"#));
    }

    #[test]
    fn system_shortcuts() {
        let m = |k: &str| {
            is_system_shortcut(&Action::KeyMacro {
                keys: vec![k.into()],
                cycle_ms: None,
                key_gap_ms: None,
                hold_ms: None,
                toggle: None,
            })
        };
        assert!(m("win+d") && m("Win") && m("ctrl+WIN+l") && m("alt+tab") && m("Alt + Tab"));
        assert!(!m("ctrl+q") && !m("alt+f4") && !m("tab") && !m("1"));
    }

    #[test]
    fn inert_icons() {
        for ok in ["🎮", "play", "icons/a.png", "icons/sub/a-b_c.svg", ""] {
            assert!(icon_is_inert(ok), "{ok}");
        }
        for bad in [
            "https://e.com/a.png",
            "//e.com/a.png",
            "https:e.com/a",
            "https:\\\\e.com\\a",
            "\\\\host\\share\\a.png",
            "file://h/a",
            "icons/../a.png",
            "icons/",
            "icons//a.png",
            "icons/a b.png",
            "a.png",
            "https\u{FF1A}\u{FF0F}\u{FF0F}e\u{FF0E}com",
            "x\u{2024}y",
        ] {
            assert!(!icon_is_inert(bad), "{bad}");
        }
    }

    #[test]
    fn icon_forms() {
        for ok in [
            "🎮",
            "icons/a.png",
            "https://example.com/a.png",
            "https://cdn.example.com:8443/a/b.png?x=1#f",
            "https://localhost/a.png",
        ] {
            assert!(icon_form_is_valid(ok), "{ok}");
        }
        for bad in [
            "http://example.com/a.png",
            "HTTPS://example.com/a.png",
            "https:example.com/a.png",
            "https:/example.com/a.png",
            "https:\\\\example.com\\a.png",
            "//example.com/a.png",
            "\\\\host\\share\\a.png",
            "file:///c:/a.png",
            "ftp://example.com/a.png",
            "data:image/png;base64,AAAA",
            "https://",
            "https:///a.png",
            "https://user@example.com/a.png",
            "https://example.com:/a.png",
            "https://example.com:80a/a.png",
            "https://exa mple.com/a.png",
            "https://example..com/a.png",
            "https://-example.com/a.png",
            "https://example.com\\@evil.com/a.png",
            "https://exämple.com/a.png",
            "https\u{FF1A}//example.com/a.png",
            "icons/../a.png",
            "a.png",
        ] {
            assert!(!icon_form_is_valid(bad), "{bad}");
        }
    }

    #[test]
    fn remote_icons_are_network_items() {
        let d = deck(
            r#"{"id":"a","type":"toggle","icon":"https://e.com/a.png","onAction":{"type":"key_macro","keys":["1"]},
               "on":{"icon":"https://e.com/on.png"},"off":{"icon":"🎤"}}"#,
            "",
        );
        let paths: Vec<_> = risk_items(&d)
            .iter()
            .flat_map(|i| i.locations.clone())
            .collect();
        assert_eq!(
            paths,
            [
                "pages/home/widgets/a/onAction",
                "pages/home/widgets/a/icon",
                "pages/home/widgets/a/on/icon"
            ]
        );
    }
}
