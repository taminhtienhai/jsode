use std::str::FromStr;

use crate::{core::JsonOutput, error::JsonError};

pub trait Deserialize: Sized {
    fn parse(out: &JsonOutput<'_>) -> Result<Self, JsonError>;
}

impl <T: FromStr> Deserialize for T {
    fn parse(out: &JsonOutput<'_>) -> Result<Self, JsonError> {
        out.parse_type::<T>()
    }
}

pub trait JsonPsr {
    type Out<'out,T> where Self: 'out;
    fn parse_into<T: Deserialize>(&self) -> Self::Out<'_,T>;
    // this method is kinda redundant, rather move this to JsonParser
    // fn parse_from<T: Deser>(&self, ast: &JsonValue) -> Self::Out<'_,T>;
}

impl <'out> JsonPsr for JsonOutput<'out> {
    type Out<'o, T> = Result<T, JsonError> where Self: 'o;

    fn parse_into<T: Deserialize>(&self) -> Self::Out<'_, T> {
        T::parse(self)
    }
}

#[cfg(test)]
mod tests {
    use crate::{core::Span, indexer::JsonIdx, parser::JsonParser};

    use super::*;

    #[derive(PartialEq, Debug, Default)]
    struct Color {
        red: u8,
        green: u8,
        blue: u8,
    }

    impl Deserialize for Color {
        fn parse(out: &JsonOutput<'_>) -> Result<Self, JsonError> {
            let Some(red) = out.index("red") else { return Err(JsonError::empty_json(Span::default())) };
            let Some(green) = out.index("green") else { return Err(JsonError::empty_json(Span::default())) };
            let Some(blue) = out.index("blue") else { return Err(JsonError::empty_json(Span::default())) };

            Ok(Color {
                red: red.parse_into::<u8>()?,
                green: green.parse_into::<u8>()?,
                blue: blue.parse_into::<u8>()?,
            })
        }
    }

    #[test]
    fn parse_usize() {
        let mut obj = JsonParser::new("{ a: 1234567 }");
        let     ast = obj.parse().unwrap();
        let    item = ast.index("a").unwrap().parse_type::<usize>();

        assert_eq!(Ok(1234567), item);
    }

    #[test]
    fn parse_struct() {
        let mut obj       = JsonParser::new("{ color: { red: 1, blue: 2, green: 3 } }");
        let mut dark_gray = JsonParser::new("{ red: 96, blue: 96, green: 96 }");

        let     item      = obj.parse().unwrap().index("color").unwrap().parse_into::<Color>();
        let     item2     = dark_gray.parse().unwrap().parse_into::<Color>();

        assert_eq!(Ok(Color { red: 1, blue: 2, green: 3 }), item);
        assert_eq!(Ok(Color { red: 96, blue: 96, green: 96 }), item2);
    }
}