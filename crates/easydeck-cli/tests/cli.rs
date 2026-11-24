//! Runs the built `easydeck-cli` binary against the shared fixtures.

use std::path::PathBuf;
use std::process::Command;

fn cli(args: &[&str]) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_easydeck-cli"))
        .args(args)
        .output()
        .expect("run easydeck-cli")
}

fn repo(path: &str) -> String {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join(path)
        .to_string_lossy()
        .into_owned()
}

#[test]
fn validate_accepts_the_examples() {
    let out = cli(&["validate", &repo("examples/starter.deck.json")]);
    assert!(out.status.success(), "{out:?}");
    assert!(out.stdout.is_empty());
}

#[test]
fn validate_prints_path_rule_message_and_fix() {
    let file = repo("tests/fixtures/invalid_dup_widget_id.deck.json");
    let out = cli(&["validate", &file]);
    assert!(!out.status.success());
    let text = String::from_utf8(out.stdout).unwrap();
    assert_eq!(
        text.trim_end(),
        format!(
            "{file}: /pages/0/widgets/1/id: duplicate-id: duplicate widget id \"a\" on page \"home\" (fix: give each widget on a page its own id)"
        )
    );
}

#[test]
fn gen_check_passes_on_committed_files() {
    let out = cli(&["gen", "--check"]);
    assert!(out.status.success(), "{out:?}");
}

#[test]
fn bad_arguments_exit_with_usage() {
    assert_eq!(cli(&[]).status.code(), Some(2));
    assert_eq!(cli(&["validate"]).status.code(), Some(2));
}
