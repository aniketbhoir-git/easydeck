//! The approval ledger (SPEC §4.3, §6.1, §6.5): which risk items the user has approved.
//!
//! One set of hashes for all decks. There is no deck key, so a renamed or imported deck keeps
//! its approvals and never gains any by its name, and the same item in two decks needs one
//! approval. An item is approved if and only if its hash is in the set; an item whose content
//! changed has a new hash and is unapproved again. The ledger lives in `approvals.json` in the
//! app data folder (`LedgerFile`), never in a deck or a pack, and holds hashes only.

use super::risk::{action_hash, action_summary, risk_items, RiskItem};
use super::{Action, Deck};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
use std::io::Write;
use std::path::{Path, PathBuf};
use ts_rs::TS;

/// Format version of `approvals.json`.
const LEDGER_VERSION: u32 = 1;

/// True for a SHA-256 hash as the ledger stores it: 64 lowercase hex characters.
pub fn is_hash(s: &str) -> bool {
    s.len() == 64 && s.bytes().all(|b| matches!(b, b'0'..=b'9' | b'a'..=b'f'))
}

/// The set of approved risk item hashes.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ApprovalLedger {
    approved: BTreeSet<String>,
}

/// The ledger file on disk: `{ "version": 1, "approved": ["<hex sha256>"] }`.
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct LedgerDoc {
    version: u32,
    approved: Vec<String>,
}

impl ApprovalLedger {
    pub fn new() -> Self {
        Self::default()
    }

    /// True when the ledger holds this hash.
    pub fn is_approved(&self, hash: &str) -> bool {
        self.approved.contains(hash)
    }

    /// True when the item's hash is in the ledger (SPEC §5 `is_approved(item)`).
    pub fn is_item_approved(&self, item: &RiskItem) -> bool {
        self.is_approved(&item.hash)
    }

    /// Approves these items (SPEC §5 `approve(items)`). Returns how many were new.
    pub fn approve(&mut self, items: &[RiskItem]) -> usize {
        items
            .iter()
            .filter(|i| self.approved.insert(i.hash.clone()))
            .count()
    }

    /// Removes an approval. Returns true when the hash was there.
    pub fn revoke(&mut self, hash: &str) -> bool {
        self.approved.remove(hash)
    }

    /// The approved hashes, sorted.
    pub fn list(&self) -> impl Iterator<Item = &str> {
        self.approved.iter().map(String::as_str)
    }

    pub fn len(&self) -> usize {
        self.approved.len()
    }

    pub fn is_empty(&self) -> bool {
        self.approved.is_empty()
    }

    /// The gate for one action: `Ok` when it runs nothing outside the window (a page switch)
    /// or when its hash is approved, else `NeedsApproval`. The caller runs the action only on
    /// `Ok`.
    pub fn authorize(&self, action: &Action) -> Result<(), NeedsApproval> {
        match action_hash(action) {
            None => Ok(()),
            Some(hash) if self.is_approved(&hash) => Ok(()),
            Some(hash) => Err(NeedsApproval {
                hash,
                summary: action_summary(action),
            }),
        }
    }

    /// Runs `action` on `runner` only when `authorize` passes. This is the only way core code
    /// reaches an `ActionRunner`: an unapproved action never gets there.
    pub fn run_approved<R: ActionRunner>(
        &self,
        runner: &mut R,
        action: &Action,
    ) -> Result<(), RunDenied<R::Error>> {
        self.authorize(action).map_err(RunDenied::NeedsApproval)?;
        runner.run(action).map_err(RunDenied::Runner)
    }

    /// The ledger as the JSON text of `approvals.json`.
    pub fn to_json(&self) -> String {
        let doc = LedgerDoc {
            version: LEDGER_VERSION,
            approved: self.approved.iter().cloned().collect(),
        };
        let mut text = serde_json::to_string_pretty(&doc).expect("ledger serializes");
        text.push('\n');
        text
    }

    /// Parses `approvals.json`. Any other shape, an unknown version or an entry that is not a
    /// lowercase SHA-256 hex string is an error: the file is then treated as corrupt.
    pub fn from_json(text: &str) -> Result<Self, String> {
        let doc: LedgerDoc = serde_json::from_str(text).map_err(|e| e.to_string())?;
        if doc.version != LEDGER_VERSION {
            return Err(format!("unsupported ledger version {}", doc.version));
        }
        if let Some(bad) = doc.approved.iter().find(|h| !is_hash(h)) {
            return Err(format!("entry is not a SHA-256 hash: {bad:.70}"));
        }
        Ok(Self {
            approved: doc.approved.into_iter().collect(),
        })
    }
}

/// An action that is not in the ledger. The hash and summary say what was refused.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NeedsApproval {
    pub hash: String,
    pub summary: String,
}

/// SPEC §5 `ActionRunner`: runs one action. R1-5 and R3-1 add the implementations; core code
/// reaches a runner only through `ApprovalLedger::run_approved`.
pub trait ActionRunner {
    type Error;
    fn run(&mut self, action: &Action) -> Result<(), Self::Error>;
}

/// Why `run_approved` did not finish.
#[derive(Debug, PartialEq, Eq)]
pub enum RunDenied<E> {
    /// The action is not approved; the runner was not called.
    NeedsApproval(NeedsApproval),
    /// The runner ran and failed.
    Runner(E),
}

// ---- the file ----

/// How `LedgerFile::load` found the file.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LedgerStatus {
    /// The file was read.
    Loaded,
    /// There is no file: the first start. The ledger is empty.
    Missing,
    /// The file was corrupt. The ledger is empty. The file was renamed to
    /// `approvals.json.corrupt` (`renamed` is false when the rename failed). `reason` says
    /// what was wrong.
    Corrupt { reason: String, renamed: bool },
    /// The file could not be read. The ledger is empty and the file is untouched.
    Unreadable { reason: String },
}

/// What `LedgerFile::load` returns: always a usable ledger.
#[derive(Debug, Clone)]
pub struct LedgerLoad {
    pub ledger: ApprovalLedger,
    pub status: LedgerStatus,
}

/// `approvals.json` at a path (SPEC §4.3: the app data folder). Writes are atomic.
#[derive(Debug, Clone)]
pub struct LedgerFile {
    path: PathBuf,
}

impl LedgerFile {
    /// The ledger file inside the app data folder `dir`.
    pub fn in_dir(dir: impl AsRef<Path>) -> Self {
        Self {
            path: dir.as_ref().join("approvals.json"),
        }
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    fn sibling(&self, suffix: &str) -> PathBuf {
        let mut name = self.path.file_name().unwrap_or_default().to_os_string();
        name.push(suffix);
        self.path.with_file_name(name)
    }

    /// Reads the ledger. A corrupt file gives an empty ledger and is renamed to
    /// `approvals.json.corrupt` (an older copy is replaced); a missing file gives an empty
    /// ledger. This never fails: a broken ledger can only make items unapproved.
    pub fn load(&self) -> LedgerLoad {
        let empty = |status| LedgerLoad {
            ledger: ApprovalLedger::new(),
            status,
        };
        let bytes = match std::fs::read(&self.path) {
            Ok(b) => b,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                return empty(LedgerStatus::Missing)
            }
            Err(e) => {
                return empty(LedgerStatus::Unreadable {
                    reason: e.to_string(),
                })
            }
        };
        let parsed = String::from_utf8(bytes)
            .map_err(|e| e.to_string())
            .and_then(|t| ApprovalLedger::from_json(&t));
        match parsed {
            Ok(ledger) => LedgerLoad {
                ledger,
                status: LedgerStatus::Loaded,
            },
            Err(reason) => {
                let renamed = std::fs::rename(&self.path, self.sibling(".corrupt")).is_ok();
                empty(LedgerStatus::Corrupt { reason, renamed })
            }
        }
    }

    /// Writes the ledger atomically: a temp file in the same folder is written, flushed and
    /// renamed over `approvals.json`, so a crash leaves the old file or the new one.
    pub fn save(&self, ledger: &ApprovalLedger) -> std::io::Result<()> {
        if let Some(dir) = self.path.parent() {
            std::fs::create_dir_all(dir)?;
        }
        let tmp = self.sibling(".tmp");
        let result = (|| {
            let mut f = std::fs::File::create(&tmp)?;
            f.write_all(ledger.to_json().as_bytes())?;
            f.sync_all()?;
            drop(f);
            std::fs::rename(&tmp, &self.path)
        })();
        if result.is_err() {
            let _ = std::fs::remove_file(&tmp);
        }
        result
    }
}

// ---- the review ----

/// One row of the review screen.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
pub struct ReviewItem {
    pub item: RiskItem,
    pub approved: bool,
}

/// What the review screen shows: the risk items of the deck, each marked approved or not, and
/// at most one message for the user (for example that the ledger file was corrupt).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
pub struct RiskReview {
    pub items: Vec<ReviewItem>,
    pub notice: Option<String>,
}

/// Compares every risk item of `deck` with the ledger. SPEC §6.5: this runs at each load.
pub fn review(deck: &Deck, ledger: &ApprovalLedger) -> Vec<ReviewItem> {
    risk_items(deck)
        .into_iter()
        .map(|item| ReviewItem {
            approved: ledger.is_item_approved(&item),
            item,
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::deck::DeckStore;

    fn deck(exe: &str, x: u32, name: &str) -> Deck {
        DeckStore::load_str(&format!(
            r#"{{"version":1,"name":"{name}","pages":[{{"id":"home","title":"H",
            "grid":{{"columns":4,"rows":2}},"widgets":[
              {{"id":"a","type":"launch","position":{{"x":{x},"y":0}},
                "action":{{"type":"launch","kind":"exe","path":"{exe}"}}}},
              {{"id":"b","type":"macro","position":{{"x":3,"y":1}},
                "action":{{"type":"key_macro","keys":["1"]}}}}]}}]}}"#
        ))
        .unwrap()
    }

    fn approved_ledger(d: &Deck) -> ApprovalLedger {
        let mut l = ApprovalLedger::new();
        l.approve(&risk_items(d));
        l
    }

    fn locked(d: &Deck, l: &ApprovalLedger) -> Vec<String> {
        review(d, l)
            .into_iter()
            .filter(|r| !r.approved)
            .flat_map(|r| r.item.locations)
            .collect()
    }

    #[test]
    fn acceptance_edited_exe_locks_one_tile_and_approval_unlocks_it() {
        let ledger = approved_ledger(&deck("C:/a.exe", 0, "d"));
        let edited = deck("C:/evil.exe", 0, "d");
        assert_eq!(locked(&edited, &ledger), ["pages/home/widgets/a/action"]);
        // The other tile still works.
        assert!(review(&edited, &ledger).iter().any(|r| r.approved));
        let mut ledger = ledger;
        ledger.approve(&risk_items(&edited));
        assert!(locked(&edited, &ledger).is_empty());
    }

    #[test]
    fn a_moved_or_renamed_tile_needs_no_new_approval() {
        let ledger = approved_ledger(&deck("C:/a.exe", 0, "one"));
        assert!(locked(&deck("C:/a.exe", 2, "another name"), &ledger).is_empty());
    }

    #[test]
    fn the_same_item_in_two_decks_needs_one_approval() {
        let one = deck("C:/a.exe", 0, "one");
        let two = deck("C:/a.exe", 1, "two");
        let mut ledger = ApprovalLedger::new();
        ledger.approve(&risk_items(&one));
        assert!(locked(&two, &ledger).is_empty());
        assert_eq!(ledger.len(), 2);
    }

    #[test]
    fn revoke_makes_the_item_unapproved_again() {
        let d = deck("C:/a.exe", 0, "d");
        let mut ledger = approved_ledger(&d);
        let h = risk_items(&d)[0].hash.clone();
        assert!(ledger.revoke(&h));
        assert!(!ledger.revoke(&h));
        assert_eq!(locked(&d, &ledger), ["pages/home/widgets/a/action"]);
        assert_eq!(ledger.list().count(), 1);
    }

    #[test]
    fn json_round_trip_and_shape() {
        let l = approved_ledger(&deck("C:/a.exe", 0, "d"));
        let text = l.to_json();
        let v: serde_json::Value = serde_json::from_str(&text).unwrap();
        assert_eq!(v["version"], 1);
        assert_eq!(v["approved"].as_array().unwrap().len(), 2);
        assert_eq!(ApprovalLedger::from_json(&text).unwrap(), l);
    }

    #[test]
    fn bad_ledger_text_is_rejected() {
        let h = "a".repeat(64);
        for bad in [
            "",
            "[]",
            "{}",
            r#"{"version":2,"approved":[]}"#,
            r#"{"version":1}"#,
            r#"{"version":1,"approved":["xyz"]}"#,
            &format!(r#"{{"version":1,"approved":["{}"]}}"#, h.to_uppercase()),
            &format!(r#"{{"version":1,"approved":["{h}"],"deck":"x"}}"#),
        ] {
            assert!(ApprovalLedger::from_json(bad).is_err(), "{bad}");
        }
        assert!(
            ApprovalLedger::from_json(&format!(r#"{{"version":1,"approved":["{h}"]}}"#)).is_ok()
        );
    }

    // ---- the gate ----

    #[derive(Default)]
    struct Fake {
        ran: Vec<Action>,
    }
    impl ActionRunner for Fake {
        type Error = ();
        fn run(&mut self, action: &Action) -> Result<(), ()> {
            self.ran.push(action.clone());
            Ok(())
        }
    }

    fn actions(d: &Deck) -> Vec<Action> {
        use crate::deck::WidgetKind;
        d.pages
            .iter()
            .flat_map(|p| &p.widgets)
            .filter_map(|w| match &w.kind {
                WidgetKind::Launch { action, .. } | WidgetKind::Macro { action, .. } => {
                    Some(action.clone())
                }
                _ => None,
            })
            .collect()
    }

    #[test]
    fn an_unapproved_action_never_reaches_the_runner() {
        let approved = deck("C:/a.exe", 0, "d");
        let edited = deck("C:/evil.exe", 0, "d");
        let ledger = approved_ledger(&approved);
        let mut runner = Fake::default();
        for a in actions(&edited) {
            let _ = ledger.run_approved(&mut runner, &a);
        }
        // Only the unchanged key macro ran.
        assert_eq!(runner.ran.len(), 1);
        assert!(matches!(runner.ran[0], Action::KeyMacro { .. }));
        let denied = ledger
            .run_approved(&mut runner, &actions(&edited)[0])
            .unwrap_err();
        assert!(matches!(denied, RunDenied::NeedsApproval(_)));
        assert_eq!(runner.ran.len(), 1);
        // An empty ledger runs nothing at all.
        let mut runner = Fake::default();
        for a in actions(&approved) {
            assert!(ApprovalLedger::new().run_approved(&mut runner, &a).is_err());
        }
        assert!(runner.ran.is_empty());
    }

    #[test]
    fn a_page_switch_needs_no_approval() {
        let a = Action::Page {
            page: "home".into(),
        };
        assert!(ApprovalLedger::new().authorize(&a).is_ok());
    }

    // ---- the file ----

    fn temp_dir(tag: &str) -> PathBuf {
        use std::sync::atomic::{AtomicUsize, Ordering};
        static N: AtomicUsize = AtomicUsize::new(0);
        let dir = std::env::temp_dir().join(format!(
            "easydeck-ledger-{tag}-{}-{}",
            std::process::id(),
            N.fetch_add(1, Ordering::Relaxed)
        ));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn missing_file_is_first_start() {
        let dir = temp_dir("missing");
        let r = LedgerFile::in_dir(&dir).load();
        assert_eq!(r.status, LedgerStatus::Missing);
        assert!(r.ledger.is_empty());
        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn save_then_load_round_trips_and_leaves_no_temp_file() {
        let dir = temp_dir("save");
        let file = LedgerFile::in_dir(dir.join("nested"));
        let l = approved_ledger(&deck("C:/a.exe", 0, "d"));
        file.save(&l).unwrap();
        file.save(&l).unwrap();
        let r = file.load();
        assert_eq!(r.status, LedgerStatus::Loaded);
        assert_eq!(r.ledger, l);
        let names: Vec<_> = std::fs::read_dir(dir.join("nested"))
            .unwrap()
            .map(|e| e.unwrap().file_name().into_string().unwrap())
            .collect();
        assert_eq!(names, ["approvals.json"]);
        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn corrupt_file_gives_empty_ledger_and_a_corrupt_copy() {
        let dir = temp_dir("corrupt");
        let file = LedgerFile::in_dir(&dir);
        std::fs::write(file.path(), "{ not json").unwrap();
        let r = file.load();
        assert!(r.ledger.is_empty());
        assert!(matches!(
            r.status,
            LedgerStatus::Corrupt { renamed: true, .. }
        ));
        assert!(!file.path().exists());
        assert_eq!(
            std::fs::read_to_string(dir.join("approvals.json.corrupt")).unwrap(),
            "{ not json"
        );
        // The next load is a first start, and a save works again.
        assert_eq!(file.load().status, LedgerStatus::Missing);
        file.save(&ApprovalLedger::new()).unwrap();
        assert_eq!(file.load().status, LedgerStatus::Loaded);
        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn a_file_with_a_bad_entry_or_invalid_utf8_is_corrupt() {
        let dir = temp_dir("badentry");
        let file = LedgerFile::in_dir(&dir);
        std::fs::write(file.path(), r#"{"version":1,"approved":["nope"]}"#).unwrap();
        assert!(matches!(file.load().status, LedgerStatus::Corrupt { .. }));
        std::fs::write(file.path(), [0xff, 0xfe, 0x00]).unwrap();
        assert!(matches!(file.load().status, LedgerStatus::Corrupt { .. }));
        let _ = std::fs::remove_dir_all(dir);
    }
}
