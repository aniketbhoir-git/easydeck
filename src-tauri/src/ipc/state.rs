//! Shared app state held by Tauri.

use crate::error::AppError;
use easydeck_core::deck::{
    is_hash, review, risk_items, source_hash, Action, ApprovalLedger, Deck, DeckStore, LedgerFile,
    LedgerStatus, RiskReview, Source,
};
use std::path::Path;
use std::sync::{Mutex, MutexGuard};

/// Default deck, bundled at build time. SPEC §4 does not name a deck location yet, so the
/// renderer starts from the example deck until the app settings issue adds a user path.
const BUNDLED_DECK: &str = include_str!("../../../examples/starter.deck.json");

pub struct AppState {
    deck: Deck,
    /// The approval ledger (SPEC §4.3): one set of hashes for all decks.
    ledger: Mutex<ApprovalLedger>,
    /// Where the ledger is saved. `None` keeps it in memory (tests).
    file: Option<LedgerFile>,
    /// One message for the user, shown by the next risk review (for example a corrupt ledger).
    notice: Mutex<Option<String>>,
}

fn lock<T>(m: &Mutex<T>) -> MutexGuard<'_, T> {
    m.lock().unwrap_or_else(|e| e.into_inner())
}

fn io_error(e: std::io::Error) -> AppError {
    AppError::Io {
        message: "could not save the approvals".into(),
        detail: Some(e.to_string()),
    }
}

impl AppState {
    /// A deck from JSON text with an empty, in-memory ledger: every risk item is unapproved.
    pub fn from_json(text: &str) -> Result<Self, AppError> {
        let deck = DeckStore::load_str(text).map_err(|errors| AppError::Invalid {
            message: "the deck file is not valid".into(),
            detail: Some(
                errors
                    .iter()
                    .map(|e| format!("{}: {}", e.path, e.message))
                    .collect::<Vec<_>>()
                    .join("; "),
            ),
        })?;
        Ok(Self {
            deck,
            ledger: Mutex::new(ApprovalLedger::new()),
            file: None,
            notice: Mutex::new(None),
        })
    }

    /// Saves the ledger to `file` from now on.
    pub fn with_file(mut self, file: LedgerFile) -> Self {
        self.file = Some(file);
        self
    }

    /// The bundled starter deck with an in-memory ledger that already approves the starter
    /// deck's own items (the first-start state).
    pub fn bundled() -> Result<Self, AppError> {
        Self::open(None)
    }

    /// The bundled starter deck with the ledger of the app data folder `dir` (`None` keeps the
    /// ledger in memory).
    ///
    /// At the first start (no `approvals.json`) the items of the bundled starter deck, which
    /// are in the build, are approved and saved. Nothing else is ever approved here: a deck
    /// from disk, a pack or a proposal gets no approval by being loaded. A corrupt file gives
    /// an empty ledger (not seeded), a `.corrupt` copy and one notice (SPEC §4.3, A-8).
    pub fn open(dir: Option<&Path>) -> Result<Self, AppError> {
        let mut state = Self::from_json(BUNDLED_DECK)?;
        let starter = risk_items(state.deck());
        let Some(dir) = dir else {
            lock(&state.ledger).approve(&starter);
            return Ok(state);
        };
        let file = LedgerFile::in_dir(dir);
        let loaded = file.load();
        let mut ledger = loaded.ledger;
        let mut notice = None;
        match loaded.status {
            LedgerStatus::Loaded => {}
            LedgerStatus::Missing => {
                ledger.approve(&starter);
                if let Err(e) = file.save(&ledger) {
                    notice = Some(format!(
                        "EasyDeck could not save its approvals ({e}). They last until the app closes."
                    ));
                }
            }
            LedgerStatus::Corrupt { renamed, .. } => {
                notice = Some(format!(
                    "The approvals file was damaged, so every action needs approval again. {}",
                    if renamed {
                        "The old file is kept as approvals.json.corrupt."
                    } else {
                        "The old file could not be renamed."
                    }
                ));
            }
            LedgerStatus::Unreadable { reason } => {
                notice = Some(format!(
                    "The approvals file could not be read ({reason}), so every action needs approval again."
                ));
            }
        }
        state.ledger = Mutex::new(ledger);
        state.file = Some(file);
        state.notice = Mutex::new(notice);
        Ok(state)
    }

    pub fn deck(&self) -> &Deck {
        &self.deck
    }

    /// True when any page has a widget with this id.
    pub fn has_widget(&self, widget_id: &str) -> bool {
        self.deck
            .pages
            .iter()
            .flat_map(|p| &p.widgets)
            .any(|w| w.id == widget_id)
    }

    /// True when a widget with this id is a toggle.
    pub fn is_toggle(&self, widget_id: &str) -> bool {
        use easydeck_core::deck::WidgetKind;
        self.deck
            .pages
            .iter()
            .flat_map(|p| &p.widgets)
            .any(|w| w.id == widget_id && matches!(w.kind, WidgetKind::Toggle { .. }))
    }

    /// True when a widget with this id has a long-press `holdAction`.
    pub fn has_hold_action(&self, widget_id: &str) -> bool {
        use easydeck_core::deck::WidgetKind;
        self.deck.pages.iter().flat_map(|p| &p.widgets).any(|w| {
            w.id == widget_id
                && matches!(
                    &w.kind,
                    WidgetKind::Launch {
                        hold_action: Some(_),
                        ..
                    } | WidgetKind::Macro {
                        hold_action: Some(_),
                        ..
                    }
                )
        })
    }

    /// The actions that `run_action` would run for a tap, or for a hold when `hold` is true.
    /// This is the approval gate's input: it names the action that really runs (`action`, or
    /// `holdAction` for a hold). It covers every widget with this id, because an id is unique
    /// only on its page: if any of them is unapproved, the call is refused. A widget kind
    /// that runs nothing through `run_action` gives none.
    pub fn tap_actions(&self, widget_id: &str, hold: bool) -> Vec<&Action> {
        use easydeck_core::deck::WidgetKind;
        self.deck
            .pages
            .iter()
            .flat_map(|p| &p.widgets)
            .filter(|w| w.id == widget_id)
            .filter_map(|w| match &w.kind {
                WidgetKind::Launch {
                    action,
                    hold_action,
                    ..
                }
                | WidgetKind::Macro {
                    action,
                    hold_action,
                } => Some(if hold {
                    hold_action.as_ref()
                } else {
                    Some(action)
                }),
                _ => None,
            })
            .flatten()
            .collect()
    }

    /// Refuses with `needs_approval` unless every action is approved (a page switch needs
    /// none). The caller runs nothing when this fails.
    pub fn authorize<'a>(
        &self,
        actions: impl IntoIterator<Item = &'a Action>,
    ) -> Result<(), AppError> {
        let ledger = lock(&self.ledger);
        for action in actions {
            ledger
                .authorize(action)
                .map_err(|n| AppError::NeedsApproval {
                    message: "this action needs your approval first".into(),
                    detail: Some(n.summary),
                })?;
        }
        Ok(())
    }

    /// True when a source may start: it reaches no network, or its endpoint is approved.
    pub fn source_allowed(&self, source: &Source) -> bool {
        source_hash(source).is_none_or(|h| lock(&self.ledger).is_approved(&h))
    }

    /// The risk items of the deck against the ledger, with the pending notice (shown once).
    pub fn risk_review(&self) -> RiskReview {
        RiskReview {
            items: review(&self.deck, &lock(&self.ledger)),
            notice: lock(&self.notice).take(),
        }
    }

    /// Approves the items with these hashes. Each hash must belong to an item of the loaded
    /// deck: the UI cannot approve what it was not shown. The ledger is saved before the
    /// change is kept, so a failed write approves nothing.
    pub fn approve(&self, hashes: &[String]) -> Result<(), AppError> {
        let items = risk_items(&self.deck);
        let mut chosen = Vec::new();
        for h in hashes {
            match items.iter().find(|i| &i.hash == h) {
                Some(item) => chosen.push(item.clone()),
                None => {
                    return Err(AppError::Invalid {
                        message: "that item is not in the deck".into(),
                        detail: Some(h.chars().take(70).collect()),
                    })
                }
            }
        }
        let mut ledger = lock(&self.ledger);
        let mut next = ledger.clone();
        next.approve(&chosen);
        self.save(&next)?;
        *ledger = next;
        Ok(())
    }

    /// Removes the approvals with these hashes.
    pub fn revoke(&self, hashes: &[String]) -> Result<(), AppError> {
        if let Some(bad) = hashes.iter().find(|h| !is_hash(h)) {
            return Err(AppError::Invalid {
                message: "not an item hash".into(),
                detail: Some(bad.chars().take(70).collect()),
            });
        }
        let mut ledger = lock(&self.ledger);
        let mut next = ledger.clone();
        for h in hashes {
            next.revoke(h);
        }
        self.save(&next)?;
        *ledger = next;
        Ok(())
    }

    fn save(&self, ledger: &ApprovalLedger) -> Result<(), AppError> {
        match &self.file {
            Some(file) => file.save(ledger).map_err(io_error),
            None => Ok(()),
        }
    }

    /// The action to run when a toggle switches to `to_on`: its `onAction` or `offAction`.
    /// `Ok(None)` is a switch to off on a toggle without an `offAction`: nothing to run.
    /// `Err` when the widget does not exist or is not a toggle.
    pub fn toggle_action(
        &self,
        widget_id: &str,
        to_on: bool,
    ) -> Result<Option<&easydeck_core::deck::Action>, AppError> {
        use easydeck_core::deck::WidgetKind;
        let widget = self
            .deck
            .pages
            .iter()
            .flat_map(|p| &p.widgets)
            .find(|w| w.id == widget_id);
        match widget.map(|w| &w.kind) {
            Some(WidgetKind::Toggle {
                on_action,
                off_action,
                ..
            }) => Ok(if to_on {
                Some(on_action)
            } else {
                off_action.as_ref()
            }),
            Some(_) => Err(AppError::Invalid {
                message: "the widget is not a toggle".into(),
                detail: Some(widget_id.to_string()),
            }),
            None => Err(AppError::NotFound {
                message: "no such widget".into(),
                detail: Some(widget_id.to_string()),
            }),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bundled_deck_loads() {
        let state = AppState::bundled().unwrap();
        assert!(!state.deck().pages.is_empty());
        assert!(state.has_widget("notes"));
        assert!(!state.has_widget("nope"));
    }

    #[test]
    fn bad_json_is_invalid_error() {
        let err = AppState::from_json("{").err().unwrap();
        assert_eq!(err.code(), "invalid");
    }
}
