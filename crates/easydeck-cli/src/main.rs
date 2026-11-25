//! `easydeck-cli`: `gen`, `gen --check` and `validate <files>`.
//!
//! Arguments are parsed with `std::env::args`; the crate adds no dependency (A-3).

use easydeck_core::contract;
use easydeck_core::deck::{ChangeKind, DeckStore};
use std::path::{Path, PathBuf};
use std::process::ExitCode;

const USAGE: &str = "usage:\n  easydeck-cli gen [--check]\n  easydeck-cli validate <file>...
  easydeck-cli diff <old> <new>";

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    match args.first().map(String::as_str) {
        Some("gen") => match &args[1..] {
            [] => finish(generate(false)),
            [flag] if flag == "--check" => finish(generate(true)),
            _ => usage(),
        },
        Some("validate") if args.len() > 1 => validate(&args[1..]),
        Some("diff") if args.len() == 3 => diff(&args[1], &args[2]),
        _ => usage(),
    }
}

fn usage() -> ExitCode {
    eprintln!("{USAGE}");
    ExitCode::from(2)
}

fn finish(result: Result<bool, String>) -> ExitCode {
    match result {
        Ok(true) => ExitCode::SUCCESS,
        Ok(false) => ExitCode::FAILURE,
        Err(message) => {
            eprintln!("error: {message}");
            ExitCode::FAILURE
        }
    }
}

/// The repository root: two levels above this crate.
fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

/// Write the generated files, or with `check` compare them with the committed ones.
/// Returns `Ok(true)` when nothing differs (or after a write).
fn generate(check: bool) -> Result<bool, String> {
    let root = repo_root();
    let schema_path = root.join("schema/deck.schema.json");
    let ts_dir = root.join("ui/src/lib/generated");

    let schema = contract::schema_json();
    let ts = contract::typescript_files().map_err(|e| format!("typescript export: {e}"))?;

    if check {
        let mut drift = Vec::new();
        let committed = std::fs::read_to_string(&schema_path)
            .map(|s| s.replace("\r\n", "\n"))
            .unwrap_or_default();
        if committed != schema {
            drift.push("schema/deck.schema.json".to_string());
        }
        let on_disk = contract::read_ts_files(&ts_dir).map_err(|e| e.to_string())?;
        for (name, text) in &ts {
            if on_disk.get(name) != Some(text) {
                drift.push(format!("ui/src/lib/generated/{name}"));
            }
        }
        for name in on_disk.keys().filter(|n| !ts.contains_key(*n)) {
            drift.push(format!("ui/src/lib/generated/{name} (stale)"));
        }
        if drift.is_empty() {
            println!("generated files are up to date");
            return Ok(true);
        }
        for file in &drift {
            eprintln!("drift: {file}");
        }
        eprintln!("run `cargo run -p easydeck-cli -- gen` and commit the result");
        return Ok(false);
    }

    std::fs::write(&schema_path, schema).map_err(|e| format!("{}: {e}", schema_path.display()))?;
    std::fs::create_dir_all(&ts_dir).map_err(|e| e.to_string())?;
    for name in contract::read_ts_files(&ts_dir)
        .map_err(|e| e.to_string())?
        .keys()
    {
        std::fs::remove_file(ts_dir.join(name)).map_err(|e| e.to_string())?;
    }
    for (name, text) in &ts {
        std::fs::write(ts_dir.join(name), text).map_err(|e| format!("{name}: {e}"))?;
    }
    println!(
        "wrote schema/deck.schema.json and {} files in ui/src/lib/generated/",
        ts.len()
    );
    Ok(true)
}

/// Print one line per error: `<file>: <path>: <rule>: <message> (fix: <fix>)`.
fn validate(files: &[String]) -> ExitCode {
    let mut failed = false;
    for file in files {
        if let Err(errors) = DeckStore::load(file) {
            failed = true;
            for e in errors {
                println!(
                    "{file}: {}: {}: {} (fix: {})",
                    e.path, e.rule, e.message, e.fix
                );
            }
        }
    }
    if failed {
        ExitCode::FAILURE
    } else {
        ExitCode::SUCCESS
    }
}

/// Print one line per difference: `<sign> <path>: <summary>`. Signs: `+` added, `-` removed,
/// `~` changed, `>` moved; a `!` after the sign marks an entry that runs something.
fn diff(old: &str, new: &str) -> ExitCode {
    let load = |file: &str| match DeckStore::load(file) {
        Ok(deck) => Some(deck),
        Err(errors) => {
            for e in errors {
                eprintln!("{file}: {}: {}: {}", e.path, e.rule, e.message);
            }
            None
        }
    };
    let (Some(a), Some(b)) = (load(old), load(new)) else {
        return ExitCode::FAILURE;
    };
    let d = DeckStore::diff(&a, &b);
    if d.is_empty() {
        println!("no differences");
    }
    for e in &d.entries {
        let sign = match e.kind {
            ChangeKind::Added => '+',
            ChangeKind::Removed => '-',
            ChangeKind::Changed => '~',
            ChangeKind::Moved => '>',
        };
        let run = if e.runs { "!" } else { "" };
        println!("{sign}{run} {}: {}", e.path, e.summary);
    }
    ExitCode::SUCCESS
}
