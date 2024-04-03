use crate::{
    core::{JsonIndex, JsonOutput, JsonValue},
    lexer::Tokenizer,
    parser::JsonParser,
};

pub struct JsonIndexer<'tk> {
    parser: &'tk JsonParser<Tokenizer<'tk>>,
    ast: JsonValue,
}

impl<'tk> JsonIndexer<'tk> {
    pub fn new(parser: &'tk JsonParser<Tokenizer<'tk>>, ast: JsonValue) -> Self {
        Self { parser, ast }
    }
}

impl<'tk> JsonIndexer<'tk> {
    pub fn index(&self, key: impl Into<Key<'tk>>) -> Option<JsonOutput<'_>> {
        match key.into() {
            Key::Str(key_slice) => self.ast.get_object_key(key_slice, self.parser),
            Key::Int(key_number) => self.ast.get_array_item(key_number, self.parser),
        }
    }
}

pub enum Key<'k> {
    Str(&'k str),
    Int(usize),
}

impl<'k> From<&'k str> for Key<'k> {
    fn from(value: &'k str) -> Self {
        return Key::Str(value);
    }
}

impl<'k> From<usize> for Key<'k> {
    fn from(value: usize) -> Self {
        return Key::Int(value);
    }
}

#[rustfmt::skip]
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn index_json_item() {
        let mut obj = JsonParser::new("{'a':1,\"b\":2, c: 3,d : [1,2,3]}");
        let ast = obj.parse().unwrap();

        let indexer = obj.indexer_from(ast);

        assert_eq!(Some(JsonOutput::new("1")), indexer.index("a"));
        assert_eq!(Some(JsonOutput::new("2")), indexer.index("b"));
        assert_eq!(Some(JsonOutput::new("3")), indexer.index("c"));
        assert_eq!(None                            , indexer.index("__not_exist__"));
        assert_eq!(Some(JsonOutput::new("[1,2,3]")), indexer.index("d"));
    }

    #[test]
    fn index_complex_json() {
        let mut obj = JsonParser::new("{a: [ { b: 1 }, { c : 2 } ] }");
        let     ast = obj.parse().unwrap();
        let indexer = obj.indexer_from(ast);

        assert_eq!(Some(JsonOutput::new("[ { b: 1 }, { c : 2 } ]")), indexer.index("a"));
    }

    #[test]
    fn index_array_item() {
        let mut array = JsonParser::new("[1,2,3]");
        let       ast = array.parse().unwrap();
        let   indexer = array.indexer_from(ast);

        assert_eq!(Some(JsonOutput::new("1")), indexer.index(0));
        assert_eq!(Some(JsonOutput::new("2")), indexer.index(1));
        assert_eq!(Some(JsonOutput::new("3")), indexer.index(2));
        assert_eq!(None                      , indexer.index(3));
    }

    #[test]
    fn index_array_complex_item() {
        let mut array = JsonParser::new("[\n\n\n\n{ a: 1 }, { 'b': 2 }, { \"c\": 3 }\n]");
        let       ast = array.parse().unwrap();
        let   indexer = array.indexer_from(ast);

        assert_eq!(Some(JsonOutput::new("{ a: 1 }"))    , indexer.index(0));
        assert_eq!(Some(JsonOutput::new("{ 'b': 2 }"))  , indexer.index(1));
        assert_eq!(Some(JsonOutput::new("{ \"c\": 3 }")), indexer.index(2));
        assert_eq!(None                                 , indexer.index(3));
    }
}
