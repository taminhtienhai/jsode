pub(crate) mod core;
pub(crate) mod error;
pub(crate) mod deserialize;
pub(crate) mod indexer;

pub mod common;
pub(crate) mod constant;
pub(crate) mod lexer;
pub(crate) mod parser;

pub mod prelude {
    pub use crate::core::{JsonOutput, Span, Result,};
    pub use crate::error::JsonError;
    pub use crate::parser::JsonParser;
    pub use crate::deserialize::{Deserialize, JsonPsr,};
    pub use crate::indexer::JsonIdx;
    #[cfg(feature = "macro")]
    pub use jsode_macro::Deserialize;
}

pub use crate::core::{JsonOutput, Span, Result,};
pub use crate::error::JsonError;
pub use crate::parser::JsonParser;
pub use crate::deserialize::{Deserialize, JsonPsr,};
pub use crate::indexer::JsonIdx;