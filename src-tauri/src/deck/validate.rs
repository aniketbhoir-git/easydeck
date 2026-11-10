//! Validation: version gate, JSON Schema, then semantic checks the schema cannot express.

use super::model::{Deck, Source, WidgetKind};
use serde::Serialize;
use serde_json::Value;
use std::collections::HashSet;
use std::sync::OnceLock;

/// The deck format version this build understands.
pub const SUPPORTED_VERSION: u64 = 1;

static SCHEMA_SRC: &str = include_str!("../../../schema/deck.schema.json");

/// One problem in a deck. `path` is a JSON pointer ("" for the whole file).
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ValidationError {
    pub path: String,
    pub message: String,
}

impl ValidationError {
    pub fn new(path: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            path: path.into(),
            message: message.into(),
        }
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
                format!(
                    "unsupported deck version {version}; this build supports {SUPPORTED_VERSION}"
                ),
            )]);
        }
    }

    let mut errors: Vec<ValidationError> = validator()
        .iter_errors(value)
        .map(|e| ValidationError::new(e.instance_path().to_string(), e.to_string()))
        .collect();
    if !errors.is_empty() {
        errors.sort_by(|a, b| a.path.cmp(&b.path));
        return Err(errors);
    }

    let deck: Deck = serde_json::from_value(value.clone()).map_err(|e| {
        vec![ValidationError::new(
            "",
            format!("deck does not match the model: {e}"),
        )]
    })?;

    let errors = semantic_errors(&deck);
    if errors.is_empty() {
        Ok(deck)
    } else {
        Err(errors)
    }
}

fn semantic_errors(deck: &Deck) -> Vec<ValidationError> {
    let mut errors = Vec::new();
    let page_ids: HashSet<&str> = deck.pages.iter().map(|p| p.id.as_str()).collect();

    if let Some(home) = deck.settings.as_ref().and_then(|s| s.home_page.as_deref()) {
        if !page_ids.contains(home) {
            errors.push(ValidationError::new(
                "/settings/homePage",
                format!("homePage \"{home}\" is not a page id"),
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
                    format!(
                        "trigger source \"{}\" must name a game_watch source",
                        trigger.source
                    ),
                ));
            }
        }

        let mut seen = HashSet::new();
        for (wi, widget) in page.widgets.iter().enumerate() {
            let base = format!("/pages/{pi}/widgets/{wi}");

            if !seen.insert(widget.id.as_str()) {
                errors.push(ValidationError::new(
                    format!("{base}/id"),
                    format!(
                        "duplicate widget id \"{}\" on page \"{}\"",
                        widget.id, page.id
                    ),
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
                    format!(
                        "widget \"{}\" spans to column {right}, row {bottom} but the grid is {}x{}",
                        widget.id, page.grid.columns, page.grid.rows
                    ),
                ));
            }

            if let WidgetKind::Gauge { source, .. } | WidgetKind::Status { source, .. } =
                &widget.kind
            {
                if !deck.sources.contains_key(source) {
                    errors.push(ValidationError::new(
                        format!("{base}/source"),
                        format!("widget source \"{source}\" is not defined in sources"),
                    ));
                }
            }
        }
    }
    errors
}
