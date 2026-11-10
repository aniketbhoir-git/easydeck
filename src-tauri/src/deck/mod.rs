//! Deck model, load and validate (SPEC §4, §5).

mod load;
mod model;
mod validate;

pub use load::DeckStore;
pub use model::*;
pub use validate::{validate, ValidationError, SUPPORTED_VERSION};
