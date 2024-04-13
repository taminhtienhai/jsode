use std::str::FromStr;

use crate::{core::{JsonArray, JsonInt, JsonOutput, JsonProp, JsonValue, Span}, error::JsonError, lexer::Tokenizer, parser::JsonParser};

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
}

impl <'out> JsonPsr for JsonOutput<'out> {
    type Out<'o, T> = Result<T, JsonError> where Self: 'o;

    fn parse_into<T: Deserialize>(&self) -> Self::Out<'_, T> {
        T::parse(self)
    }
}

pub trait JsonVecPsr {
    type Out<'out, T> where Self: 'out;
    fn parse_into_arr<T: FromStr>(&self) -> Self::Out<'_,T>;
}

impl <'out> JsonVecPsr for JsonOutput<'out> {
    type Out<'o, T> = Result<Vec<T>, JsonError> where Self: 'o;

    fn parse_into_arr<T: FromStr>(&self) -> Self::Out<'_, T> {
        let error_msg = "cannot parse because this is not an array";
        match &self.ast {
            crate::common::Holder::Owned(owned) => match owned {
                JsonValue::Array(JsonArray { properties, .. }) => parse_properties_to_vec(self.parser, properties),
                _ => Err(JsonError::custom(error_msg, Span::default()))
            },
            crate::common::Holder::Borrow(bor) => match bor {
                JsonValue::Array(JsonArray { properties, .. }) => parse_properties_to_vec(self.parser, properties),
                _ => Err(JsonError::custom(error_msg, Span::default()))
            },
        }
    }
}

pub trait JsonSlicePsr {
    type Out<'out, T: 'out> where Self: 'out;
    fn parse_into_slice<T: FromStr>(&self) -> Self::Out<'_,T>;
}

impl <'out> JsonSlicePsr for JsonOutput<'out> {
    type Out<'o, T: 'o> = Result<&'o [T], JsonError> where Self: 'o;

    fn parse_into_slice<T: FromStr>(&self) -> Self::Out<'_, T> {
        let error_msg = "cannot parse because this is not an array";
        match &self.ast {
            crate::common::Holder::Owned(owned) => match owned {
                JsonValue::Array(JsonArray { properties, .. }) => parse_properties_to_slice(self.parser, properties),
                _ => Err(JsonError::custom(error_msg, Span::default()))
            },
            crate::common::Holder::Borrow(bor) => match bor {
                JsonValue::Array(JsonArray { properties, .. }) => parse_properties_to_slice(self.parser, properties),
                _ => Err(JsonError::custom(error_msg, Span::default()))
            },
        }
    }
}

#[inline]
fn parse_properties_to_vec<T: FromStr>(
    parser: &JsonParser<Tokenizer<'_>>,
    props: &[JsonProp<JsonInt>],
) -> Result<Vec<T>, JsonError> {
    props.iter()
        .map(|prop| JsonOutput::new(parser, &prop.value).parse_type::<T>())
        .collect()
}

#[inline]
fn parse_properties_to_slice<'a, T: FromStr>(
    parser: &'a JsonParser<Tokenizer<'a>>,
    props: &'a [JsonProp<JsonInt>],
) -> Result<&'a [T], JsonError> {
    let res: Result<Vec<T>, JsonError> = props.iter()
        .map(|prop| JsonOutput::new(parser, &prop.value).parse_type::<T>())
        .collect();
    res.map(|x| unsafe { std::slice::from_raw_parts(x.as_ptr(), x.len()) })
}

#[cfg(test)]
mod tests {
    use crate::{core::Span, indexer::JsonIdx, parser::JsonParser};

    use super::*;

    #[derive(PartialEq, Debug, Default)]
    struct Nested {
        nested: u8,
    }

    impl Nested {
        pub fn new(inner: u8) -> Self {
            Self { nested: inner }
        }
    }

    impl Deserialize for Nested {
        fn parse(out: &JsonOutput<'_>) -> Result<Self, JsonError> {
            Ok(Self {
                nested: out.index("nested").ok_or_else(|| JsonError::empty_json(Span::default()))?.parse_into::<u8>()?,
            })
        }
    }

    #[derive(PartialEq, Debug, Default)]
    struct Color {
        red: u8,
        green: u8,
        blue: u8,
        alpha: Option<u8>,
        inner: Nested,
        hue: Vec<u8>,
    }

    impl Deserialize for Color {
        fn parse(out: &JsonOutput<'_>) -> Result<Self, JsonError> {
            Ok(Self {
                red: out.index("red").ok_or_else(|| JsonError::empty_json(Span::default()))?.parse_into::<u8>()?,
                green: out.index("green").ok_or_else(|| JsonError::empty_json(Span::default()))?.parse_into::<u8>()?,
                blue: out.index("blue").ok_or_else(|| JsonError::empty_json(Span::default()))?.parse_into::<u8>()?,
                alpha: out.index("alpha")
                    .map(|x| x.parse_into::<u8>())
                    .map_or(Ok(None), |x| x.map(Some))?,
                inner: out.index("inner").ok_or_else(|| JsonError::empty_json(Span::default()))?.parse_into::<Nested>()?,
                hue: out.index("hue").ok_or_else(|| JsonError::empty_json(Span::default()))?.parse_into_arr::<u8>()?,
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
        let mut obj       = JsonParser::new("{ color: { red: 1, blue: 2, green: 3, inner: { nested: 1 }, hue: [1,2,3] } }");
        let mut dark_gray = JsonParser::new("{ red: 96, blue: 96, green: 96, alpha: 1, inner: { nested: 1 }, hue: [1,2,3] }");

        let     item      = obj.parse().unwrap().index("color").unwrap().parse_into::<Color>();
        let     item2     = dark_gray.parse().unwrap().parse_into::<Color>();

        let arr = &[1,2,3];
        assert_eq!(Ok(Color { red: 1, blue: 2, green: 3, alpha: None, inner: Nested::new(1), hue: vec![1,2,3] }), item);
        assert_eq!(Ok(Color { red: 96, blue: 96, green: 96, alpha: Some(1), inner: Nested::new(1), hue: vec![1,2,3] }), item2);
    }
}