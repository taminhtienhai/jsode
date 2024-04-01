use crate::{core::{JsonIndex, JsonOutput, JsonValue}, lexer::Tokenizer, parser::{self, JsonParser}};

pub struct JsonIndexer<'tk> {
    parser: &'tk JsonParser<Tokenizer<'tk>>,
    ast: JsonValue,
}

impl <'tk> JsonIndexer<'tk> {
    pub fn new(parser: &'tk JsonParser<Tokenizer<'tk>>, ast: JsonValue) -> Self {
        Self { parser, ast }
    }
}

impl <'tk> JsonIndexer<'tk> {
    pub fn index(&self, key: &'tk str) -> Option<JsonOutput<'_>> {
        self.ast.get(key, self.parser)
    }
}