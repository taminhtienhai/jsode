use crate::{
    core::{JsonIndex, JsonOutput, JsonValue},
    lexer::Tokenizer,
    parser::JsonParser,
};

pub struct JsonIndexer<'idx> {
    parser: &'idx JsonParser<Tokenizer<'idx>>,
    ast: &'idx JsonValue,
}

impl<'idx> JsonIndexer<'idx> {
    pub fn new(parser: &'idx JsonParser<Tokenizer<'idx>>, ast: &'idx JsonValue) -> Self {
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

        let indexer = obj.indexer_from(&ast);

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
        let indexer = obj.indexer_from(&ast);

        assert_eq!(Some(JsonOutput::new("[ { b: 1 }, { c : 2 } ]")), indexer.index("a"));
    }

    #[test]
    fn index_array_item() {
        let mut array = JsonParser::new("[1,2,3]");
        let       ast = array.parse().unwrap();
        let   indexer = array.indexer_from(&ast);

        assert_eq!(Some(JsonOutput::new("1")), indexer.index(0));
        assert_eq!(Some(JsonOutput::new("2")), indexer.index(1));
        assert_eq!(Some(JsonOutput::new("3")), indexer.index(2));
        assert_eq!(None                      , indexer.index(3));
    }

    #[test]
    fn index_array_complex_item() {
        let mut array = JsonParser::new("[\n\n\n\n{ a: 1 }, { 'b': 2 }, { \"c\": 3 }\n]");
        let       ast = array.parse().unwrap();
        let   indexer = array.indexer_from(&ast);

        assert_eq!(Some(JsonOutput::new("{ a: 1 }"))    , indexer.index(0));
        assert_eq!(Some(JsonOutput::new("{ 'b': 2 }"))  , indexer.index(1));
        assert_eq!(Some(JsonOutput::new("{ \"c\": 3 }")), indexer.index(2));
        assert_eq!(None                                 , indexer.index(3));
    }

    #[test]
    fn parse_number_output() {
        let mut obj = JsonParser::new("{ u8: 255, u16: 65535, u32: 4294967295, u64: 18446744073709551615 }");
        let       ast = obj.parse().unwrap();
        let   indexer = obj.indexer_from(&ast);

        assert_eq!(255, indexer.index("u8").unwrap().parse::<u8>().unwrap());
        assert_eq!(65_535, indexer.index("u16").unwrap().parse::<u16>().unwrap());
        assert_eq!(4_294_967_295, indexer.index("u32").unwrap().parse::<u32>().unwrap());
        assert_eq!(18_446_744_073_709_551_615, indexer.index("u64").unwrap().parse::<u64>().unwrap());
    }

    #[test]
    fn parse_boolean_output() {
        let mut obj = JsonParser::new("{ t: true, f: false }");
        let       ast = obj.parse().unwrap();
        let   indexer = obj.indexer_from(&ast);

        assert_eq!(true, indexer.index("t").unwrap().parse::<bool>().unwrap());
        assert_eq!(false, indexer.index("f").unwrap().parse::<bool>().unwrap());
    }
}
