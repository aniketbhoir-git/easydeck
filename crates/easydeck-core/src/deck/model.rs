//! The deck contract. These types are the single source of `schema/deck.schema.json` and of
//! the TypeScript types in `ui/src/lib/generated/` (SPEC §4.1). Run `cargo xtask gen` (or
//! `cargo run -p easydeck-cli -- gen`) after a change; never edit the generated files.
//!
//! Doc comments here become schema descriptions and are read by an LLM that writes decks.
//! Write them for that reader. Grid coordinates are zero-based.
//!
//! Rules the schema cannot express live in the semantic validator (`validate.rs`): the
//! required fields of each launch kind, and unknown fields on a widget.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use ts_rs::TS;

/// An EasyDeck deck: pages of widgets. A widget runs an action or shows a value from a source.
/// An LLM may write or change this file only; it never changes app code.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema, TS)]
#[serde(deny_unknown_fields)]
#[schemars(title = "EasyDeck deck")]
pub struct Deck {
    /// Optional pointer to this schema, for editors.
    #[serde(rename = "$schema", default, skip_serializing_if = "Option::is_none")]
    #[ts(optional, rename = "$schema")]
    pub schema: Option<String>,
    /// Deck format version. Always 1.
    #[schemars(extend("const" = 1))]
    #[ts(type = "1")]
    pub version: u32,
    /// Deck name, shown to the user. 1 to 80 characters.
    #[schemars(length(min = 1, max = 80))]
    pub name: String,
    /// Deck-wide options.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub settings: Option<Settings>,
    /// Named data feeds. Each key is a source id that gauge and status widgets and page
    /// triggers refer to. A key matches `^[a-z0-9][a-z0-9_-]{0,63}$`.
    #[serde(default)]
    #[schemars(extend("propertyNames" = {"pattern": "^[a-z0-9][a-z0-9_-]{0,63}$"}))]
    pub sources: BTreeMap<String, Source>,
    /// The pages of the deck, at least one. The first page is the home page unless
    /// `settings.homePage` names another.
    #[schemars(length(min = 1))]
    pub pages: Vec<Page>,
}

/// Deck-wide options. Every field is optional.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema, TS)]
#[serde(deny_unknown_fields)]
pub struct Settings {
    /// Id of the page shown at start and after a context page closes. It must be the id of
    /// a page in this deck.
    #[serde(rename = "homePage", default, skip_serializing_if = "Option::is_none")]
    #[schemars(regex(pattern = "^[a-z0-9][a-z0-9_-]{0,63}$"))]
    #[ts(optional, rename = "homePage")]
    pub home_page: Option<String>,
    /// Color theme. Defaults to `dark`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub theme: Option<Theme>,
    /// Accent color as `#rrggbb`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[schemars(regex(pattern = "^#[0-9a-fA-F]{6}$"))]
    #[ts(optional)]
    pub accent: Option<String>,
}

/// Color theme.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema, TS)]
#[serde(rename_all = "snake_case")]
pub enum Theme {
    Dark,
    Light,
}

/// One screen of widgets on a grid.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema, TS)]
#[serde(deny_unknown_fields)]
pub struct Page {
    /// Page id, unique in the deck. Matches `^[a-z0-9][a-z0-9_-]{0,63}$`. A `page` action
    /// and `settings.homePage` refer to it.
    #[schemars(regex(pattern = "^[a-z0-9][a-z0-9_-]{0,63}$"))]
    pub id: String,
    /// Title shown to the user. 1 to 60 characters.
    #[schemars(length(min = 1, max = 60))]
    pub title: String,
    /// Size of the widget grid.
    pub grid: Grid,
    /// Context rule. When set, the page opens when the rule becomes true and the deck
    /// returns to the home page when it becomes false.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub trigger: Option<Trigger>,
    /// Widgets on this page. Two widgets must not overlap and each must fit in the grid.
    /// Widget ids must be unique on the page.
    pub widgets: Vec<Widget>,
}

/// Grid size in cells.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema, TS)]
#[serde(deny_unknown_fields)]
pub struct Grid {
    /// Number of columns, 1 to 24.
    #[schemars(range(min = 1, max = 24))]
    pub columns: u32,
    /// Number of rows, 1 to 24.
    #[schemars(range(min = 1, max = 24))]
    pub rows: u32,
}

/// Context rule that opens a page while a game runs.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema, TS)]
#[serde(deny_unknown_fields)]
pub struct Trigger {
    /// Id of a `game_watch` source in `sources`. Any other kind of source is an error.
    #[schemars(regex(pattern = "^[a-z0-9][a-z0-9_-]{0,63}$"))]
    pub source: String,
    /// The condition that opens the page.
    pub when: TriggerWhen,
}

/// Condition of a trigger.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema, TS)]
#[serde(deny_unknown_fields)]
pub struct TriggerWhen {
    /// Process name, for example `game.exe`.
    #[schemars(length(min = 1))]
    pub process: String,
}

/// Grid position, zero-based. The widget covers `w` columns and `h` rows from cell (`x`, `y`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema, TS)]
#[serde(deny_unknown_fields)]
pub struct Position {
    /// Column of the left edge, from 0.
    #[schemars(range(min = 0))]
    pub x: u32,
    /// Row of the top edge, from 0.
    #[schemars(range(min = 0))]
    pub y: u32,
    /// Width in columns. Defaults to 1.
    #[serde(default = "one")]
    #[schemars(range(min = 1))]
    #[ts(optional, as = "Option<u32>")]
    pub w: u32,
    /// Height in rows. Defaults to 1.
    #[serde(default = "one")]
    #[schemars(range(min = 1))]
    #[ts(optional, as = "Option<u32>")]
    pub h: u32,
}

fn one() -> u32 {
    1
}

/// A tile on a page. The `type` field picks the kind; each kind adds its own fields. A field
/// that the kind does not define is an error.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema, TS)]
pub struct Widget {
    /// Widget id, unique on its page. Matches `^[a-z0-9][a-z0-9_-]{0,63}$`.
    #[schemars(regex(pattern = "^[a-z0-9][a-z0-9_-]{0,63}$"))]
    pub id: String,
    /// Text under the tile. At most 60 characters.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[schemars(length(max = 60))]
    #[ts(optional)]
    pub label: Option<String>,
    /// Icon: a path inside the pack (`icons/...`), plain glyph text, or an https URL. A remote
    /// icon is a network endpoint: a risk item that needs approval (SPEC §4.3).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub icon: Option<String>,
    /// Where the widget sits on the grid. Omit it and the app places the widget in the first
    /// free cell in reading order (row by row, left to right); it then covers one cell.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub position: Option<Position>,
    /// The widget kind and its fields.
    #[serde(flatten)]
    pub kind: WidgetKind,
}

/// How a toggle looks in one state. Every field is optional; a missing one falls back to the
/// widget's own `label` and `icon`, and to the default tile colour.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema, TS)]
#[serde(deny_unknown_fields)]
pub struct ToggleLook {
    /// Text under the tile in this state. At most 60 characters.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[schemars(length(max = 60))]
    #[ts(optional)]
    pub label: Option<String>,
    /// Icon in this state: a path inside the pack (`icons/...`), plain glyph text, or an https
    /// URL. A remote icon is a network endpoint: a risk item that needs approval (SPEC §4.3).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub icon: Option<String>,
    /// Tile colour in this state: a hex colour, `#rgb` or `#rrggbb`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub color: Option<String>,
}

/// The kind of a widget, tagged by `type`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema, TS)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum WidgetKind {
    /// Starts a program, game or link. `action` must be a `launch` action.
    Launch {
        /// The launch action to run on tap.
        action: Action,
        /// `fixed` keeps deck order. `recent` sorts by local play history at render time.
        /// Defaults to `fixed`.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        #[ts(optional)]
        order: Option<Order>,
        /// Optional second action that runs on a long press (held at least 600 ms without
        /// moving). It must be a `launch` action, like `action`. It is a risk item like
        /// `action`: it runs only after the user approves it. A long press never also runs
        /// `action`.
        #[serde(
            rename = "holdAction",
            default,
            skip_serializing_if = "Option::is_none"
        )]
        #[ts(optional, rename = "holdAction")]
        hold_action: Option<Action>,
    },
    /// Sends keys. `action` must be a `key_macro` action.
    Macro {
        /// The key macro to run on tap.
        action: Action,
        /// Optional second action that runs on a long press (held at least 600 ms without
        /// moving). It must be a `key_macro` action, like `action`. It is a risk item like
        /// `action`: it runs only after the user approves it. A long press never also runs
        /// `action`.
        #[serde(
            rename = "holdAction",
            default,
            skip_serializing_if = "Option::is_none"
        )]
        #[ts(optional, rename = "holdAction")]
        hold_action: Option<Action>,
    },
    /// A tile with two states. A tap while off runs `onAction` and switches the tile on; a tap
    /// while on runs `offAction` (if set) and switches it off. The state is kept on this
    /// device between runs, never in the deck file. It starts off. Each action is a risk item.
    Toggle {
        /// Action that runs when the tile switches on.
        #[serde(rename = "onAction")]
        #[ts(rename = "onAction")]
        on_action: Action,
        /// Action that runs when the tile switches off. Without it, switching off runs nothing.
        #[serde(rename = "offAction", default, skip_serializing_if = "Option::is_none")]
        #[ts(optional, rename = "offAction")]
        off_action: Option<Action>,
        /// Look while on. Missing fields use the widget's own `label` and `icon`, and the
        /// default tile colour.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        #[ts(optional)]
        on: Option<ToggleLook>,
        /// Look while off. Missing fields use the widget's own `label` and `icon`, and the
        /// default tile colour.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        #[ts(optional)]
        off: Option<ToggleLook>,
    },
    /// Shows a number from a source as a gauge.
    Gauge {
        /// Id of a source in `sources`.
        #[schemars(regex(pattern = "^[a-z0-9][a-z0-9_-]{0,63}$"))]
        source: String,
        /// Metric name: a key of the source's flat payload, for example `cpu.load` or
        /// `gpu.temp`. It is not a JSON path. A missing or null value shows "no data".
        metric: String,
        /// Value at the empty end. Defaults to 0.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        #[ts(optional)]
        min: Option<f64>,
        /// Value at the full end. Defaults to 100.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        #[ts(optional)]
        max: Option<f64>,
        /// Unit shown after the value. At most 8 characters.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        #[schemars(length(max = 8))]
        #[ts(optional)]
        unit: Option<String>,
    },
    /// Shows OK or a problem from a source value.
    Status {
        /// Id of a source in `sources`.
        #[schemars(regex(pattern = "^[a-z0-9][a-z0-9_-]{0,63}$"))]
        source: String,
        /// Metric name: a key of the source's flat payload, for example `cpu.load`. It is not
        /// a JSON path. A missing or null value shows "no data".
        #[serde(default, skip_serializing_if = "Option::is_none")]
        #[ts(optional)]
        path: Option<String>,
        /// Value that counts as OK. Anything else shows as a problem. A numeric `okWhen`
        /// compares as a number, so `1` matches `1.0`. Any other `okWhen` compares as text.
        /// When omitted, any non-zero value is OK.
        #[serde(rename = "okWhen", default, skip_serializing_if = "Option::is_none")]
        #[ts(optional, rename = "okWhen")]
        ok_when: Option<String>,
    },
    /// A label, a clock or a date.
    Text {
        /// `label` shows `text`. `clock` and `date` show the time. Defaults to `label`.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        #[ts(optional)]
        kind: Option<TextKind>,
        /// Text for the `label` kind. At most 200 characters.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        #[schemars(length(max = 200))]
        #[ts(optional)]
        text: Option<String>,
        /// Clock only. `true` shows 12-hour time with AM or PM. `false` shows 24-hour time.
        /// Omit it to follow the system setting.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        #[ts(optional)]
        hour12: Option<bool>,
        /// Clock only. `true` shows seconds. Defaults to `false`.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        #[ts(optional)]
        seconds: Option<bool>,
        /// Clock only. `true` shows the date in a smaller line under the time. Defaults to
        /// `false`. The `date` kind always shows the date.
        #[serde(rename = "showDate", default, skip_serializing_if = "Option::is_none")]
        #[ts(optional)]
        show_date: Option<bool>,
    },
    /// A counter tile. A tap adds `step` to the value; a long press (held at least 600 ms)
    /// resets it to `start`. The value is kept on this device between runs, never in the deck
    /// file. It runs no action and is not a risk item. The tile shows the widget `label`.
    Counter {
        /// First value, and the value after a reset. Defaults to 0. When `min` or `max` is set,
        /// `start` must lie between them.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        #[ts(optional, type = "number")]
        start: Option<i64>,
        /// Amount added by each tap. May be negative. Must not be 0. Defaults to 1.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        #[ts(optional, type = "number")]
        step: Option<i64>,
        /// Lowest value. The count stops here. Optional.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        #[ts(optional, type = "number")]
        min: Option<i64>,
        /// Highest value. The count stops here. Optional.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        #[ts(optional, type = "number")]
        max: Option<i64>,
    },
    /// A countdown or stopwatch tile. A tap starts, pauses and resumes it; a long press (held
    /// at least 600 ms) resets it. At the end of a countdown the tile changes colour and
    /// pulses until the next tap. No sound. The state is kept in memory only: it is lost when
    /// the app closes. It runs no action and is not a risk item. The tile shows the widget
    /// `label`.
    Timer {
        /// `countdown` counts down from `durationS`; `stopwatch` counts up.
        mode: TimerMode,
        /// Countdown only: length in seconds, 1 to 86400. Required for a countdown. A
        /// stopwatch must not set it.
        #[serde(rename = "durationS", default, skip_serializing_if = "Option::is_none")]
        #[ts(optional, rename = "durationS")]
        duration_s: Option<u32>,
    },
    /// A volume slider and mute button for the default output device. The slider is
    /// horizontal on a tile that is wider than it is tall, and vertical otherwise. It runs no
    /// action and is not a risk item. The tile shows the widget `label`.
    Volume {},
    /// Controls the system media session (any player that reports to Windows). It has no
    /// provider and no volume control.
    Media {
        /// Controls to show. Each appears at most once.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        #[schemars(extend("uniqueItems" = true))]
        #[ts(optional)]
        controls: Option<Vec<MediaControl>>,
    },
    /// Shows a web page. The page gets no access to the app.
    Embed {
        /// An http or https URL.
        #[schemars(regex(pattern = "^https?://"), extend("format" = "uri"))]
        url: String,
        /// Seconds between reloads. 0 or absent means no reload.
        #[serde(
            rename = "refreshSeconds",
            default,
            skip_serializing_if = "Option::is_none"
        )]
        #[schemars(range(min = 0))]
        #[ts(optional, rename = "refreshSeconds")]
        refresh_seconds: Option<u32>,
    },
}

/// Order of tiles.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema, TS)]
#[serde(rename_all = "snake_case")]
pub enum Order {
    Fixed,
    Recent,
}

/// What a text widget shows.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema, TS)]
#[serde(rename_all = "snake_case")]
pub enum TextKind {
    Label,
    Clock,
    Date,
}

/// What a timer tile does.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema, TS)]
#[serde(rename_all = "snake_case")]
pub enum TimerMode {
    Countdown,
    Stopwatch,
}

/// A media control button.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema, TS)]
#[serde(rename_all = "snake_case")]
pub enum MediaControl {
    PlayPause,
    Next,
    Previous,
}

/// What a launch action starts. Each kind needs one field: `steam` needs `appId`, `exe` needs
/// `path`, `uri` and `url` need `target`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema, TS)]
#[serde(rename_all = "snake_case")]
pub enum LaunchKind {
    Steam,
    Exe,
    Uri,
    Url,
}

/// An action, tagged by `type`. An action runs only after the user approves it in the app.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema, TS)]
#[serde(tag = "type", rename_all = "snake_case", deny_unknown_fields)]
pub enum Action {
    /// Starts a program or opens a link. The field that the `kind` needs is required:
    /// `appId` for `steam`, `path` for `exe`, `target` for `uri` and `url`.
    Launch {
        /// What to start.
        kind: LaunchKind,
        /// Steam app id. Required for kind `steam`.
        #[serde(rename = "appId", default, skip_serializing_if = "Option::is_none")]
        #[schemars(range(min = 1))]
        #[ts(optional, rename = "appId", type = "number")]
        app_id: Option<u64>,
        /// Program path. Required for kind `exe`.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        #[ts(optional)]
        path: Option<String>,
        /// Arguments for the program.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        #[ts(optional)]
        args: Option<Vec<String>>,
        /// Path or URI to try when `path` is missing. Kind `exe` only.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        #[ts(optional)]
        fallback: Option<String>,
        /// URI or URL. Required for kinds `uri` and `url`.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        #[ts(optional)]
        target: Option<String>,
    },
    /// Presses keys in order, once or in a loop.
    KeyMacro {
        /// Key names pressed in order, for example `["1", "2", "ctrl+q"]`. At least one.
        #[schemars(length(min = 1))]
        keys: Vec<String>,
        /// Repeat period in ms. 0 or absent runs the sequence once.
        #[serde(rename = "cycleMs", default, skip_serializing_if = "Option::is_none")]
        #[schemars(range(min = 0))]
        #[ts(optional, rename = "cycleMs", type = "number")]
        cycle_ms: Option<u64>,
        /// Pause between keys in ms. Defaults to 50.
        #[serde(rename = "keyGapMs", default, skip_serializing_if = "Option::is_none")]
        #[schemars(range(min = 0))]
        #[ts(optional, rename = "keyGapMs", type = "number")]
        key_gap_ms: Option<u64>,
        /// How long each key stays down in ms. Defaults to 30.
        #[serde(rename = "holdMs", default, skip_serializing_if = "Option::is_none")]
        #[schemars(range(min = 0))]
        #[ts(optional, rename = "holdMs", type = "number")]
        hold_ms: Option<u64>,
        /// When true, a tap starts the loop and the next tap stops it. Defaults to false.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        #[ts(optional)]
        toggle: Option<bool>,
    },
    /// Runs a command from the allow-list in the app settings. Never a raw command line.
    Shell {
        /// Allow-list id, for example `lock-screen`.
        #[schemars(regex(pattern = "^[a-z0-9][a-z0-9_-]{0,63}$"))]
        command: String,
        /// Arguments for the command.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        #[ts(optional)]
        args: Option<Vec<String>>,
    },
    /// Switches to another page of this deck.
    Page {
        /// Id of the page to show.
        #[schemars(regex(pattern = "^[a-z0-9][a-z0-9_-]{0,63}$"))]
        page: String,
    },
}

/// A named data feed, tagged by `kind`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema, TS)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum Source {
    /// CPU, GPU and memory numbers from this PC.
    SystemStats {
        /// Poll interval in ms, at least 250. Defaults to 1000.
        #[serde(
            rename = "intervalMs",
            default,
            skip_serializing_if = "Option::is_none"
        )]
        #[schemars(range(min = 250))]
        #[ts(optional, rename = "intervalMs", type = "number")]
        interval_ms: Option<u64>,
        /// Metrics to read, each at most once. Allowed: `cpu.load`, `cpu.temp`, `gpu.load`,
        /// `gpu.temp`, `ram.used`, `ram.total`.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        #[schemars(extend(
            "uniqueItems" = true,
            "items" = {"enum": ["cpu.load", "cpu.temp", "gpu.load", "gpu.temp", "ram.used", "ram.total"]}
        ))]
        #[ts(optional)]
        metrics: Option<Vec<String>>,
    },
    /// Polls a web URL. The call runs only after the user approves the URL.
    HttpPoll {
        /// An http or https URL.
        #[schemars(regex(pattern = "^https?://"), extend("format" = "uri"))]
        url: String,
        /// Seconds between polls, at least 5. Defaults to 60.
        #[serde(
            rename = "intervalSeconds",
            default,
            skip_serializing_if = "Option::is_none"
        )]
        #[schemars(range(min = 5))]
        #[ts(optional, rename = "intervalSeconds", type = "number")]
        interval_seconds: Option<u64>,
        /// Seconds before a poll times out, 1 to 60. Defaults to 10.
        #[serde(
            rename = "timeoutSeconds",
            default,
            skip_serializing_if = "Option::is_none"
        )]
        #[schemars(range(min = 1, max = 60))]
        #[ts(optional, rename = "timeoutSeconds", type = "number")]
        timeout_seconds: Option<u64>,
    },
    /// Watches for running processes. A page `trigger` refers to this kind.
    GameWatch {
        /// Process names to watch, at least one, each at most once.
        #[schemars(length(min = 1), extend("uniqueItems" = true))]
        processes: Vec<String>,
        /// Check interval in ms, at least 500. Defaults to 2000.
        #[serde(
            rename = "intervalMs",
            default,
            skip_serializing_if = "Option::is_none"
        )]
        #[schemars(range(min = 500))]
        #[ts(optional, rename = "intervalMs", type = "number")]
        interval_ms: Option<u64>,
    },
}
