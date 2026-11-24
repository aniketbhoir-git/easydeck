//! Validation: version gate, JSON Schema, then semantic checks the schema cannot express.

use super::model::{Action, Deck, LaunchKind, Source, Widget, WidgetKind};
use serde::Serialize;
use serde_json::Value;
use std::collections::HashSet;
use std::sync::OnceLock;

/// The deck format version this build understands.
pub const SUPPORTED_VERSION: u64 = 1;

static SCHEMA_SRC: &str = include_str!("../../../../schema/deck.schema.json");

/// One problem in a deck. `path` is a JSON pointer ("" for the whole file). `rule` names the
/// rule that failed and `fix` says what to change. An LLM reads these, so each one stands
/// alone.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ValidationError {
    pub path: String,
    pub rule: String,
    pub message: String,
    pub fix: String,
}

impl ValidationError {
    pub fn new(
        path: impl Into<String>,
        rule: impl Into<String>,
        message: impl Into<String>,
        fix: impl Into<String>,
    ) -> Self {
        Self {
            path: path.into(),
            rule: rule.into(),
            message: message.into(),
            fix: fix.into(),
        }
    }
}

/// The fix text for a JSON Schema keyword.
fn schema_fix(keyword: &str) -> &'static str {
    match keyword {
        "required" => "add the missing field",
        "additionalProperties" | "unevaluatedProperties" => {
            "remove the field that the schema does not define"
        }
        "type" => "use a value of the type that the schema names",
        "enum" | "const" => "use one of the values that the schema lists",
        "pattern" | "format" => "change the value to match the pattern in the schema",
        "minimum" | "maximum" | "exclusiveMinimum" | "exclusiveMaximum" => {
            "change the number to fit the range in the schema"
        }
        "minLength" | "maxLength" => "change the text length to fit the limits in the schema",
        "minItems" | "maxItems" => "change the number of list items to fit the limits in the schema",
        "uniqueItems" => "remove the repeated list item",
        "oneOf" | "anyOf" => {
            "make the object match one variant of the schema; check its `type` or `kind` field and the fields that variant needs"
        }
        _ => "change the value to satisfy the schema",
    }
}

fn validator() -> &'static jsonschema::Validator {
    static V: OnceLock<jsonschema::Validator> = OnceLock::new();
    V.get_or_init(|| {
        let schema: Value =
            serde_json::from_str(SCHEMA_SRC).expect("embedded deck schema is valid JSON");
        jsonschema::validator_for(&schema).expect("embedded deck schema compiles")
    })
}

/// Validate a deck value. Empty result means the deck is valid.
pub fn validate(value: &Value) -> Vec<ValidationError> {
    match check(value) {
        Ok(_) => Vec::new(),
        Err(errors) => errors,
    }
}

/// Full pipeline: version, schema, typed deserialize, semantic checks.
pub(super) fn check(value: &Value) -> Result<Deck, Vec<ValidationError>> {
    if let Some(version) = value.get("version") {
        if version.as_u64() != Some(SUPPORTED_VERSION) {
            return Err(vec![ValidationError::new(
                "/version",
                "version",
                format!(
                    "unsupported deck version {version}; this build supports {SUPPORTED_VERSION}"
                ),
                format!("set \"version\" to {SUPPORTED_VERSION}"),
            )]);
        }
    }

    let mut errors: Vec<ValidationError> = validator()
        .iter_errors(value)
        .map(|e| {
            let keyword = e.kind().keyword();
            ValidationError::new(
                e.instance_path().to_string(),
                format!("schema/{keyword}"),
                e.to_string(),
                schema_fix(keyword),
            )
        })
        .collect();
    if !errors.is_empty() {
        errors.sort_by(|a, b| a.path.cmp(&b.path));
        return Err(errors);
    }

    let deck: Deck = serde_json::from_value(value.clone()).map_err(|e| {
        vec![ValidationError::new(
            "",
            "model",
            format!("deck does not match the model: {e}"),
            "make the deck match the schema",
        )]
    })?;

    let errors = semantic_errors(&deck, value);
    if errors.is_empty() {
        Ok(deck)
    } else {
        Err(errors)
    }
}

fn semantic_errors(deck: &Deck, raw: &Value) -> Vec<ValidationError> {
    let mut errors = Vec::new();
    let page_ids: HashSet<&str> = deck.pages.iter().map(|p| p.id.as_str()).collect();

    if let Some(home) = deck.settings.as_ref().and_then(|s| s.home_page.as_deref()) {
        if !page_ids.contains(home) {
            errors.push(ValidationError::new(
                "/settings/homePage",
                "home-page",
                format!("homePage \"{home}\" is not a page id"),
                "set homePage to the id of a page in this deck, or remove it",
            ));
        }
    }

    for (pi, page) in deck.pages.iter().enumerate() {
        if let Some(trigger) = &page.trigger {
            if !matches!(
                deck.sources.get(&trigger.source),
                Some(Source::GameWatch { .. })
            ) {
                errors.push(ValidationError::new(
                    format!("/pages/{pi}/trigger/source"),
                    "trigger-source",
                    format!(
                        "trigger source \"{}\" must name a game_watch source",
                        trigger.source
                    ),
                    "point the trigger at a source whose kind is game_watch",
                ));
            }
        }

        let mut seen = HashSet::new();
        for (wi, widget) in page.widgets.iter().enumerate() {
            let base = format!("/pages/{pi}/widgets/{wi}");

            if !seen.insert(widget.id.as_str()) {
                errors.push(ValidationError::new(
                    format!("{base}/id"),
                    "duplicate-id",
                    format!(
                        "duplicate widget id \"{}\" on page \"{}\"",
                        widget.id, page.id
                    ),
                    "give each widget on a page its own id",
                ));
            }

            let p = widget.position;
            let (right, bottom) = (
                u64::from(p.x) + u64::from(p.w),
                u64::from(p.y) + u64::from(p.h),
            );
            if right > u64::from(page.grid.columns) || bottom > u64::from(page.grid.rows) {
                errors.push(ValidationError::new(
                    format!("{base}/position"),
                    "off-grid",
                    format!(
                        "widget \"{}\" spans to column {right}, row {bottom} but the grid is {}x{}",
                        widget.id, page.grid.columns, page.grid.rows
                    ),
                    "move or shrink the widget, or make the grid larger",
                ));
            }

            if let WidgetKind::Gauge { source, .. } | WidgetKind::Status { source, .. } =
                &widget.kind
            {
                if !deck.sources.contains_key(source) {
                    errors.push(ValidationError::new(
                        format!("{base}/source"),
                        "unknown-source",
                        format!("widget source \"{source}\" is not defined in sources"),
                        "use the id of a source in the sources map, or add the source",
                    ));
                }
            }

            errors.extend(widget_errors(&base, widget, raw_widget(raw, pi, wi)));
        }
    }
    errors
}

fn raw_widget(raw: &Value, page: usize, widget: usize) -> Option<&Value> {
    raw.get("pages")?.get(page)?.get("widgets")?.get(widget)
}

/// Rules that the generated schema cannot express: the action type of a widget, the required
/// fields of each launch kind, and unknown fields on a widget. The widget type is a flattened
/// enum, so the schema cannot close it. All three report at the widget path.
fn widget_errors(base: &str, widget: &Widget, raw: Option<&Value>) -> Vec<ValidationError> {
    let mut errors = Vec::new();

    // `action` and `holdAction` of a launch or macro widget share one action-kind rule.
    let (kind_word, want_launch, action, hold_action) = match &widget.kind {
        WidgetKind::Launch {
            action,
            hold_action,
            ..
        } => ("launch", true, Some(action), hold_action.as_ref()),
        WidgetKind::Macro {
            action,
            hold_action,
        } => ("macro", false, Some(action), hold_action.as_ref()),
        _ => ("", false, None, None),
    };
    let (action_type, wrong_word) = if want_launch {
        ("launch", "a launch action")
    } else {
        ("key_macro", "a key_macro action")
    };
    for (field, a) in [("action", action), ("holdAction", hold_action)] {
        let Some(a) = a else { continue };
        let ok = if want_launch {
            matches!(a, Action::Launch { .. })
        } else {
            matches!(a, Action::KeyMacro { .. })
        };
        if !ok {
            errors.push(ValidationError::new(
                base,
                "widget-action",
                format!(
                    "widget \"{}\" is a {kind_word} widget but its {field} is not {wrong_word}",
                    widget.id
                ),
                format!("set {field}.type to \"{action_type}\""),
            ));
        }
    }

    let actions: Vec<&Action> = match &widget.kind {
        WidgetKind::Launch {
            action,
            hold_action,
            ..
        }
        | WidgetKind::Macro {
            action,
            hold_action,
        } => std::iter::once(action)
            .chain(hold_action.as_ref())
            .collect(),
        WidgetKind::Toggle {
            on_action,
            off_action,
        } => std::iter::once(on_action)
            .chain(off_action.as_ref())
            .collect(),
        _ => Vec::new(),
    };
    for action in actions {
        if let Action::Launch {
            kind,
            app_id,
            path,
            target,
            ..
        } = action
        {
            let (kind_name, field, missing) = match kind {
                LaunchKind::Steam => ("steam", "appId", app_id.is_none()),
                LaunchKind::Exe => ("exe", "path", path.is_none()),
                LaunchKind::Uri => ("uri", "target", target.is_none()),
                LaunchKind::Url => ("url", "target", target.is_none()),
            };
            if missing {
                errors.push(ValidationError::new(
                    base,
                    "launch-required-field",
                    format!(
                        "widget \"{}\": a launch action of kind {} needs the field \"{field}\"",
                        widget.id, kind_name
                    ),
                    format!("add \"{field}\" to the launch action"),
                ));
            }
        }
    }

    // Unknown fields: a key in the file that the typed widget does not write back.
    if let (Some(Value::Object(raw)), Ok(Value::Object(known))) =
        (raw, serde_json::to_value(widget))
    {
        let unknown: Vec<&str> = raw
            .keys()
            .filter(|k| !known.contains_key(*k))
            .map(String::as_str)
            .collect();
        if !unknown.is_empty() {
            errors.push(ValidationError::new(
                base,
                "unknown-field",
                format!(
                    "widget \"{}\" has fields that its type does not define: {}",
                    widget.id,
                    unknown.join(", ")
                ),
                format!("remove {}", unknown.join(", ")),
            ));
        }
    }
    errors
}
