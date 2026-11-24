//! The generated deck contract: the JSON Schema and the TypeScript types (SPEC §4.1).
//! `easydeck-cli gen` writes them; `gen --check` compares them with the committed files.

use crate::deck::Deck;
use std::collections::BTreeMap;
use std::path::Path;
use ts_rs::TS;

/// The JSON Schema of a deck, as pretty JSON with a final newline.
pub fn schema_json() -> String {
    let mut schema = schemars::generate::SchemaSettings::draft2020_12()
        .into_generator()
        .into_root_schema_for::<Deck>();
    schema.insert(
        "$id".into(),
        "https://easydeck.dev/schema/deck.schema.json".into(),
    );
    // schemars 1.2.3 has no setting that stops `Option<T>` from adding `null`. A deck file
    // omits an optional field; it never writes `null`. Strip `null` from the generated schema.
    let mut value = serde_json::to_value(&schema).expect("schema serializes");
    strip_null(&mut value);
    let mut text = serde_json::to_string_pretty(&value).expect("schema serializes");
    text.push('\n');
    text
}

/// Remove the `null` alternative that schemars adds for `Option<T>`: `"type": [T, "null"]`
/// becomes `"type": T`, and `anyOf: [X, {"type": "null"}]` becomes `X`.
fn strip_null(value: &mut serde_json::Value) {
    use serde_json::Value;
    match value {
        Value::Array(items) => items.iter_mut().for_each(strip_null),
        Value::Object(map) => {
            map.values_mut().for_each(strip_null);
            if let Some(Value::Array(types)) = map.get_mut("type") {
                types.retain(|t| t != "null");
                if types.len() == 1 {
                    let only = types.remove(0);
                    map.insert("type".into(), only);
                }
            }
            if let Some(Value::Array(any_of)) = map.get_mut("anyOf") {
                any_of.retain(|s| s.get("type") != Some(&Value::String("null".into())));
                if any_of.len() == 1 {
                    let only = any_of.remove(0);
                    map.remove("anyOf");
                    if let Value::Object(inner) = only {
                        map.extend(inner);
                    }
                }
            }
        }
        _ => {}
    }
}

/// The TypeScript files for the deck types, keyed by file name, with LF line ends.
pub fn typescript_files() -> std::io::Result<BTreeMap<String, String>> {
    static COUNTER: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
    let n = COUNTER.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    let dir = std::env::temp_dir().join(format!("easydeck-ts-{}-{n}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir)?;
    let result = Deck::export_all(&ts_rs::Config::new().with_out_dir(&dir))
        .map_err(|e| std::io::Error::other(e.to_string()))
        .and_then(|()| read_ts_files(&dir));
    let _ = std::fs::remove_dir_all(&dir);
    result
}

/// Read every `.ts` file of a directory, keyed by file name, with LF line ends. A missing
/// directory gives an empty map.
pub fn read_ts_files(dir: &Path) -> std::io::Result<BTreeMap<String, String>> {
    let mut files = BTreeMap::new();
    if !dir.exists() {
        return Ok(files);
    }
    for entry in std::fs::read_dir(dir)? {
        let path = entry?.path();
        if path.extension().is_some_and(|e| e == "ts") {
            let name = path.file_name().unwrap().to_string_lossy().into_owned();
            files.insert(name, std::fs::read_to_string(&path)?.replace("\r\n", "\n"));
        }
    }
    Ok(files)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The committed schema is the one that the validator embeds. It must match the types.
    #[test]
    fn committed_schema_matches_the_types() {
        let committed = include_str!("../../../schema/deck.schema.json").replace("\r\n", "\n");
        assert_eq!(
            committed,
            schema_json(),
            "run `cargo run -p easydeck-cli -- gen`"
        );
    }

    #[test]
    fn typescript_export_covers_the_deck_types() {
        let files = typescript_files().unwrap();
        for name in ["Deck.ts", "Widget.ts", "Action.ts", "Source.ts"] {
            assert!(files.contains_key(name), "missing {name}");
        }
    }
}
