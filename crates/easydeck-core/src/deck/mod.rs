//! Deck model, load and validate (SPEC §4, §5).

mod diff;
mod load;
mod model;
mod validate;

pub use diff::{diff, ChangeKind, DeckDiff, DiffEntry, DiffScope};
pub use load::DeckStore;
pub use model::*;
pub use validate::{validate, ValidationError, SUPPORTED_VERSION};
