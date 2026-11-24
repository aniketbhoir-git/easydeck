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
    let mut text = serde_json::to_string_pretty(&schema).expect("schema serializes");
    text.push('\n');
    text
}

/// The TypeScript files for the deck types, keyed by file name, with LF line ends.
pub fn typescript_files() -> std::io::Result<BTreeMap<String, String>> {
    let dir = std::env::temp_dir().join(format!("easydeck-ts-{}", std::process::id()));
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
