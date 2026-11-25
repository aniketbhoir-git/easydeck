//! Deck model, load and validate (SPEC §4, §5).

mod approval;
mod diff;
mod load;
mod model;
mod risk;
mod sha256;
mod validate;

pub use approval::{
    is_hash, review, ActionRunner, ApprovalLedger, LedgerFile, LedgerLoad, LedgerStatus,
    NeedsApproval, ReviewItem, RiskReview, RunDenied,
};
pub use diff::{diff, ChangeKind, DeckDiff, DiffEntry, DiffScope};
pub use load::DeckStore;
pub use model::*;
pub use risk::{
    action_hash, action_summary, is_system_shortcut, risk_items, source_hash, RiskItem, RiskKind,
};
pub use validate::{validate, ValidationError, SUPPORTED_VERSION};
