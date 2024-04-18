use crate::{common, core::{JsonInt, JsonOutput, JsonProp, JsonValue}, error::JsonError, lexer::Tokenizer, parser::JsonParser};

pub trait Deserialize: Sized {
    fn parse(out: &JsonOutput<'_>) -> Result<Self, JsonError>;
}

// impl Deserialize for $ty

impl Deserialize for u8 {
    fn parse(out: &JsonOutput<'_>) -> Result<Self, JsonError> {
        out.parse_type::<u8>()
    }
}

impl Deserialize for u16 {
    fn parse(out: &JsonOutput<'_>) -> Result<Self, JsonError> {
        out.parse_type::<u16>()
    }
}

impl Deserialize for u32 {
    fn parse(out: &JsonOutput<'_>) -> Result<Self, JsonError> {
        out.parse_type::<u32>()
    }
}

impl Deserialize for u64 {
    fn parse(out: &JsonOutput<'_>) -> Result<Self, JsonError> {
        out.parse_type::<u64>()
    }
}

impl Deserialize for usize {
    fn parse(out: &JsonOutput<'_>) -> Result<Self, JsonError> {
        out.parse_type::<usize>()
    }
}

impl Deserialize for String {
    fn parse(out: &JsonOutput<'_>) -> Result<Self, JsonError> {
        out.parse_type::<String>()
    }
}

impl <T: Deserialize> Deserialize for Vec<T> {
    fn parse(out: &JsonOutput<'_>) -> Result<Self, JsonError> {
        match &out.ast {
            common::Holder::Owned(JsonValue::Array(arr)) => parse_properties_to_vec(out.parser, &arr.properties), 
            common::Holder::Borrow(JsonValue::Array(arr)) => parse_properties_to_vec(out.parser, &arr.properties),
            common::Holder::Owned(other_type) => Err(JsonError::invalid_array(other_type.get_span())),
            common::Holder::Borrow(other_type) => Err(JsonError::invalid_array(other_type.get_span())),
        }
    }
}

#[inline(always)]
fn parse_properties_to_vec<T: Deserialize>(
    parser: &JsonParser<Tokenizer<'_>>,
    props: &[JsonProp<JsonInt>],
) -> Result<Vec<T>, JsonError> {
    props.iter()
        .map(|prop| JsonOutput::new(parser, &prop.value).parse_into::<T>())
        .collect()
}

// WARN: introduce memory leak because we haven't dealloc slice value after use
// SOLUTION: impl Drop trait that provide manual memory management
#[cfg(feature = "unstable")]
impl <T: Deserialize> Deserialize for &[T] {
    fn parse(out: &JsonOutput<'_>) -> Result<Self, JsonError> {
        match &out.ast {
            common::Holder::Owned(JsonValue::Array(JsonArray { properties: props, .. })) => {
                let res = props.iter()
                    .map(|prop| JsonOutput::new(out.parser, &prop.value).parse_into::<T>())
                    .collect::<Result<Vec<T>, JsonError>>()?;
                let ptr = res.as_ptr();
                let len = res.len();
                mem::forget(res); // avoid `res` being dropped by rust compiler
                Ok(unsafe { std::slice::from_raw_parts(ptr, len) })
            },
            common::Holder::Borrow(JsonValue::Array(JsonArray { properties: props, .. })) => {
                let res = props.iter()
                    .map(|prop| JsonOutput::new(out.parser, &prop.value).parse_into::<T>())
                    .collect::<Result<Vec<T>, JsonError>>()?;
                let ptr = res.as_ptr();
                let len = res.len();
                mem::forget(res);
                Ok(unsafe { std::slice::from_raw_parts(ptr, len) })
            },
            common::Holder::Owned(other_type) => Err(JsonError::invalid_array(other_type.get_span())),
            common::Holder::Borrow(other_type) => Err(JsonError::invalid_array(other_type.get_span())),
        }
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
        hue: Vec<Nested>,
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
                hue: out.index("hue").ok_or_else(|| JsonError::empty_json(Span::default()))?.parse_into::<Vec<Nested>>()?,
            })
        }
    }

    #[test]
    fn parse_usize() {
        let mut obj = JsonParser::new(r#"{ a: 1234567,b: "\"b\"" }"#);
        let     ast = obj.parse().unwrap();
        let    item = ast.index("a").unwrap().parse_type::<usize>();
        assert_eq!(Ok(1234567), item);
        assert_eq!(Ok("\\\"b\\\""), ast.index("b").unwrap().parse_type::<String>().as_deref())
    }

    #[test]
    fn parse_struct() {
        let mut obj       = JsonParser::new("{ color: { red: 1, blue: 2, green: 3, inner: { nested: 1 }, hue: [{nested: 1}] } }");
        let mut dark_gray = JsonParser::new("{ red: 96, blue: 96, green: 96, alpha: 1, inner: { nested: 1 }, hue: [{nested: 2}] }");

        let     item      = obj.parse().unwrap().index("color").unwrap().parse_into::<Color>();
        let     item2     = dark_gray.parse().unwrap().parse_into::<Color>();

        assert_eq!(Ok(Color { red: 1, blue: 2, green: 3, alpha: None, inner: Nested::new(1), hue: vec![Nested::new(1)] }), item);
        assert_eq!(Ok(Color { red: 96, blue: 96, green: 96, alpha: Some(1), inner: Nested::new(1), hue: vec![Nested::new(2)] }), item2);
    }
}