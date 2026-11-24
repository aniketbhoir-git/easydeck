//! `cargo xtask check [--core]` and `cargo xtask gen [--check]`.
//!
//! CI calls these commands, so a local run and CI cannot drift. This crate holds no generator
//! code: `gen` runs the `gen` command of `easydeck-cli`.

use std::path::{Path, PathBuf};
use std::process::{Command, ExitCode};

const USAGE: &str = "usage: cargo xtask check [--core]\n       cargo xtask gen [--check]";

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let args: Vec<&str> = args.iter().map(String::as_str).collect();
    let root = root_dir();
    let result = match args.as_slice() {
        ["check"] => check(&root, false),
        ["check", "--core"] => check(&root, true),
        ["gen"] => gen(&root, false),
        ["gen", "--check"] => gen(&root, true),
        _ => Err(USAGE.to_string()),
    };
    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(msg) => {
            eprintln!("error: {msg}");
            ExitCode::FAILURE
        }
    }
}

fn root_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("xtask lives one level below the workspace root")
        .to_path_buf()
}

/// Run one named step; fail fast with the step name.
fn step(name: &str, dir: &Path, program: &str, args: &[&str]) -> Result<(), String> {
    eprintln!("\n==> {name}: {program} {}", args.join(" "));
    let status = Command::new(program)
        .args(args)
        .current_dir(dir)
        .status()
        .map_err(|e| format!("step `{name}` could not start `{program}`: {e}"))?;
    if status.success() {
        Ok(())
    } else {
        Err(format!("step `{name}` failed ({status})"))
    }
}

fn cargo() -> String {
    std::env::var("CARGO").unwrap_or_else(|_| "cargo".to_string())
}

fn npm() -> &'static str {
    if cfg!(windows) {
        "npm.cmd"
    } else {
        "npm"
    }
}

fn gen(root: &Path, check_only: bool) -> Result<(), String> {
    let mut args = vec![
        "run",
        "--quiet",
        "--locked",
        "-p",
        "easydeck-cli",
        "--",
        "gen",
    ];
    if check_only {
        args.push("--check");
    }
    step(
        if check_only { "gen --check" } else { "gen" },
        root,
        &cargo(),
        &args,
    )
}

/// Expand `examples/*.deck.json` (std only, no glob crate).
fn example_decks(root: &Path) -> Result<Vec<String>, String> {
    let dir = root.join("examples");
    let mut found: Vec<String> = std::fs::read_dir(&dir)
        .map_err(|e| format!("cannot read {}: {e}", dir.display()))?
        .filter_map(Result::ok)
        .filter_map(|entry| entry.file_name().into_string().ok())
        .filter(|name| name.ends_with(".deck.json"))
        .map(|name| format!("examples/{name}"))
        .collect();
    found.sort();
    if found.is_empty() {
        return Err("no examples/*.deck.json found".to_string());
    }
    Ok(found)
}

fn validate(root: &Path) -> Result<(), String> {
    let decks = example_decks(root)?;
    let mut args = vec![
        "run",
        "--quiet",
        "--locked",
        "-p",
        "easydeck-cli",
        "--",
        "validate",
    ];
    args.extend(decks.iter().map(String::as_str));
    step("validate examples", root, &cargo(), &args)
}

fn check(root: &Path, core_only: bool) -> Result<(), String> {
    let cargo = cargo();
    // The UI builds first: `tauri::generate_context!` embeds `ui/dist`, so clippy and the
    // tests need it.
    if !core_only {
        let ui = root.join("ui");
        step("ui install", &ui, npm(), &["ci"])?;
        step("ui check", &ui, npm(), &["run", "check"])?;
        step("ui test", &ui, npm(), &["test"])?;
        step("ui build", &ui, npm(), &["run", "build"])?;
    }
    step("fmt", root, &cargo, &["fmt", "--all", "--", "--check"])?;
    if core_only {
        let pkgs = ["-p", "easydeck-core", "-p", "easydeck-cli"];
        let mut clippy = vec!["clippy", "--locked"];
        clippy.extend(pkgs);
        clippy.extend(["--all-targets", "--all-features", "--", "-D", "warnings"]);
        step("clippy", root, &cargo, &clippy)?;
        let mut test = vec!["test", "--locked"];
        test.extend(pkgs);
        step("test", root, &cargo, &test)?;
    } else {
        step(
            "clippy",
            root,
            &cargo,
            &[
                "clippy",
                "--locked",
                "--workspace",
                "--all-targets",
                "--all-features",
                "--",
                "-D",
                "warnings",
            ],
        )?;
        step("test", root, &cargo, &["test", "--workspace", "--locked"])?;
    }
    gen(root, true)?;
    validate(root)?;
    eprintln!("\nall checks passed");
    Ok(())
}
