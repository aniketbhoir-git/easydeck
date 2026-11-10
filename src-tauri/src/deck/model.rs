//! Typed mirror of `schema/deck.schema.json`.
//!
//! The schema is the gatekeeper (required fields, per-kind rules, unknown fields). These
//! types are deserialized only after the schema check passes. Grid coordinates are zero-based.

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Deck {
    #[serde(rename = "$schema", default, skip_serializing_if = "Option::is_none")]
    pub schema: Option<String>,
    pub version: u32,
    pub name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub settings: Option<Settings>,
    #[serde(default)]
    pub sources: BTreeMap<String, Source>,
    pub pages: Vec<Page>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Settings {
    #[serde(rename = "homePage", default, skip_serializing_if = "Option::is_none")]
    pub home_page: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub theme: Option<Theme>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub accent: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Theme {
    Dark,
    Light,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Page {
    pub id: String,
    pub title: String,
    pub grid: Grid,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub trigger: Option<Trigger>,
    pub widgets: Vec<Widget>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Grid {
    pub columns: u32,
    pub rows: u32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Trigger {
    pub source: String,
    pub when: TriggerWhen,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TriggerWhen {
    pub process: String,
}

/// Grid position, zero-based. `w` and `h` default to 1.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Position {
    pub x: u32,
    pub y: u32,
    #[serde(default = "one")]
    pub w: u32,
    #[serde(default = "one")]
    pub h: u32,
}

fn one() -> u32 {
    1
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Widget {
    pub id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub icon: Option<String>,
    pub position: Position,
    #[serde(flatten)]
    pub kind: WidgetKind,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum WidgetKind {
    Launch {
        action: Action,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        order: Option<Order>,
    },
    Macro {
        action: Action,
    },
    Toggle {
        #[serde(rename = "onAction")]
        on_action: Action,
        #[serde(rename = "offAction", default, skip_serializing_if = "Option::is_none")]
        off_action: Option<Action>,
    },
    Gauge {
        source: String,
        metric: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        min: Option<f64>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        max: Option<f64>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        unit: Option<String>,
    },
    Status {
        source: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        path: Option<String>,
        #[serde(rename = "okWhen", default, skip_serializing_if = "Option::is_none")]
        ok_when: Option<String>,
    },
    Text {
        #[serde(default, skip_serializing_if = "Option::is_none")]
        kind: Option<TextKind>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        text: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        format: Option<String>,
    },
    Media {
        provider: MediaProvider,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        controls: Option<Vec<MediaControl>>,
    },
    Embed {
        url: String,
        #[serde(
            rename = "refreshSeconds",
            default,
            skip_serializing_if = "Option::is_none"
        )]
        refresh_seconds: Option<u32>,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Order {
    Fixed,
    Recent,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TextKind {
    Label,
    Clock,
    Date,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MediaProvider {
    Spotify,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MediaControl {
    PlayPause,
    Next,
    Previous,
    Volume,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LaunchKind {
    Steam,
    Exe,
    Uri,
    Url,
}

/// Any action, tagged by `type`. Per-kind required fields are enforced by the schema.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case", deny_unknown_fields)]
pub enum Action {
    Launch {
        kind: LaunchKind,
        #[serde(rename = "appId", default, skip_serializing_if = "Option::is_none")]
        app_id: Option<u64>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        path: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        args: Option<Vec<String>>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        fallback: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        target: Option<String>,
    },
    KeyMacro {
        keys: Vec<String>,
        #[serde(rename = "cycleMs", default, skip_serializing_if = "Option::is_none")]
        cycle_ms: Option<u64>,
        #[serde(rename = "keyGapMs", default, skip_serializing_if = "Option::is_none")]
        key_gap_ms: Option<u64>,
        #[serde(rename = "holdMs", default, skip_serializing_if = "Option::is_none")]
        hold_ms: Option<u64>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        toggle: Option<bool>,
    },
    Shell {
        command: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        args: Option<Vec<String>>,
    },
    Page {
        page: String,
    },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum Source {
    SystemStats {
        #[serde(
            rename = "intervalMs",
            default,
            skip_serializing_if = "Option::is_none"
        )]
        interval_ms: Option<u64>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        metrics: Option<Vec<String>>,
    },
    HttpPoll {
        url: String,
        #[serde(
            rename = "intervalSeconds",
            default,
            skip_serializing_if = "Option::is_none"
        )]
        interval_seconds: Option<u64>,
        #[serde(
            rename = "timeoutSeconds",
            default,
            skip_serializing_if = "Option::is_none"
        )]
        timeout_seconds: Option<u64>,
    },
    GameWatch {
        processes: Vec<String>,
        #[serde(
            rename = "intervalMs",
            default,
            skip_serializing_if = "Option::is_none"
        )]
        interval_ms: Option<u64>,
    },
}
