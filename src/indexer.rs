use crate::{common, core::{JsonBlock, JsonOutput, JsonValue}};

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

pub trait JsonIdx {
    type Out<'out> where Self: 'out;
    fn index<'a>(&self, key: impl Into<Key<'a>>) -> Self::Out<'_>; 
}


impl <'out> JsonIdx for JsonOutput<'out> {
    type Out<'o> = Option<JsonOutput<'o>> where Self: 'o;

    fn index<'a>(&self, key: impl Into<Key<'a>>) -> Self::Out<'_> {
        let block = self.ast.as_slice().first();
        match (key.into(), block) {
            // the `pos` is relative position of value with parent object
            // now the value is the first block of AST
            (Key::Str(key_str), Some(JsonBlock { value: JsonValue::Object(obj, _), .. })) => obj
                .get(&(common::hash_str(key_str) as usize))
                .map(|pos| JsonOutput::new(self.parser, &self.ast.as_slice()[*pos..])),
            (Key::Int(key_int), Some(JsonBlock { value: JsonValue::Array(positions, _), .. })) if key_int < positions.len() => {
                let ast_slice = self.ast.as_slice();
                let ast_len = ast_slice.len();
                let start = positions[key_int];
                let end = positions.get(key_int + 1);
                let range = end.map(|e| start..*e).unwrap_or(start..ast_len);
                Some(JsonOutput::new(self.parser, &ast_slice[range]))
            },
            _ => None,
        }
    }
}

#[rustfmt::skip]
#[cfg(test)]
mod tests {
    use crate::{indexer::JsonIdx, parser::JsonParser};

    #[test]
    fn index_json_item() {
        let source = "{'a':1,\"b\":2, c: 3,d : [1,2,3]}";

        println!("{source}");

        let mut obj = JsonParser::new(source);
        let ast = obj.parse().unwrap();


        assert_eq!(Ok("1"), ast.index("a").unwrap().to_slice());
        assert_eq!(Ok("2"), ast.index("b").unwrap().to_slice());
        assert_eq!(Ok("3"), ast.index("c").unwrap().to_slice());
        assert_eq!(None         , ast.index("__not_exist__"));
        assert_eq!(Ok("[1,2,3]"), ast.index("d").unwrap().to_slice());
    }

    #[test]
    fn index_complex_json() {
        let mut obj = JsonParser::new("{a: [ { b: 1 }, { c : 2 } ] }");
        let     ast = obj.parse().unwrap();

        assert_eq!(Ok("[ { b: 1 }, { c : 2 } ]"), ast.index("a").unwrap().to_slice());
    }

    #[test]
    fn index_array_item() {
        let mut array = JsonParser::new("[1,2,3]");
        let       ast = array.parse().unwrap();

        assert_eq!(Ok("1"), ast.index(0).unwrap().to_slice());
        assert_eq!(Ok("2"), ast.index(1).unwrap().to_slice());
        assert_eq!(Ok("3"), ast.index(2).unwrap().to_slice());
        assert_eq!(None   , ast.index(3));
    }

    #[test]
    fn index_array_complex_item() {
        let mut array = JsonParser::new("[\n\n\n\n{ a: 1 }, { 'b': 2 }, { \"c\": 3 }\n]");
        let       ast = array.parse().unwrap();

        assert_eq!(Ok("{ a: 1 }"), ast.index(0).unwrap().to_slice());
        assert_eq!(Ok("{ 'b': 2 }"), ast.index(1).unwrap().to_slice());
        assert_eq!(Ok("{ \"c\": 3 }"), ast.index(2).unwrap().to_slice());
        assert_eq!(None, ast.index(3));
    }

    #[test]
    fn index_none_exist_key() {
        let mut object = JsonParser::new("{ a: 1 }");
        let        ast = object.parse().unwrap();

        assert_eq!(None, ast.index("b"))
    }
}
