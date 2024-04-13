pub(crate) mod core;
pub(crate) mod error;
pub(crate) mod deserialize;
pub(crate) mod indexer;

pub(crate) mod common;
pub(crate) mod constant;
pub(crate) mod lexer;
pub(crate) mod parser;

pub mod prelude {
    pub use crate::core::{JsonOutput, Span};
    pub use crate::error::JsonError;
    pub use crate::parser::JsonParser;
    pub use crate::deserialize::{Deserialize, JsonPsr, JsonVecPsr,};
    pub use crate::indexer::JsonIdx;
    pub use json_parser_macro::Deserialize;
}


#[macro_use]
pub mod _macro;