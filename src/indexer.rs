use crate::{common, core::{JsonOutput, JsonProp, JsonValue}};

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
        match key.into() {
            Key::Str(key_str) => match self.ast.as_ref() {
                JsonValue::Object(obj) => obj.properties
                    .get(&common::hash_str(key_str))
                    .map(|JsonProp { value, .. }| JsonOutput::new(self.parser, value)),
                _ => None,
            },
            Key::Int(key_int) => match self.ast.as_ref() {
                JsonValue::Array(arr) => {
                    if key_int >= arr.properties.len() {
                        return None;
                    }
                    return arr.properties
                        .get(key_int)
                        .map(|prop| JsonOutput::new(self.parser, &prop.value))
                },
                _ => None,
            }
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
