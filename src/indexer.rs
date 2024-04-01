use crate::{core::{JsonIndex, JsonOutput, JsonValue}, lexer::Tokenizer, parser::JsonParser};

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn index_json_item() {
        let mut obj = JsonParser::new("{'a':1,\"b\":2, c: 3}");
        let ast = obj.parse().unwrap();

        let indexer = obj.indexer_from(ast);

        assert_eq!(Some(JsonOutput::new("1")), indexer.index("a"));
        assert_eq!(Some(JsonOutput::new("2")), indexer.index("b"));
        assert_eq!(Some(JsonOutput::new("3")), indexer.index("c"));
        assert_eq!(None                      , indexer.index("d"));
    }
}