use crate::{common, core::{Decimal, Heximal, Integer, JsonInt, JsonOutput, JsonProp, JsonType, JsonValue, NumType, StrType}, error::JsonError, lexer::Tokenizer, parser::JsonParser};

pub trait Deserialize: Sized {
    fn parse(out: &JsonOutput<'_>) -> Result<Self, JsonError>;
}

macro_rules! impl_unsigned_deserialization {
    ($type:ty) => {
        impl Deserialize for $type {
            fn parse(out: &JsonOutput<'_>) -> Result<Self, JsonError> {
                match &out.ast {
                    // positive number
                    common::Holder::Owned(JsonValue::Data(JsonType::Num(NumType::Integer(Integer::Positive(int_span))), _)) => out.parse_type_span::<$type>(int_span.clone()),
                    common::Holder::Borrow(JsonValue::Data(JsonType::Num(NumType::Integer(Integer::Positive(int_span))), _)) => out.parse_type_span::<$type>(int_span.clone()),
                    // hexadecimal
                    common::Holder::Owned(JsonValue::Data(JsonType::Num(NumType::Hex(Heximal::Positive(hex_span))), _)) => out.to_slice_span(hex_span.clone().shrink_left(2)).and_then(|slice| <$type>::from_str_radix(slice,16).map_err(|err| JsonError::custom(err.to_string(), hex_span.clone()))),
                    common::Holder::Borrow(JsonValue::Data(JsonType::Num(NumType::Hex(Heximal::Positive(hex_span))), _)) =>  out.to_slice_span(hex_span.clone().shrink_left(2)).and_then(|slice| <$type>::from_str_radix(slice,16).map_err(|err| JsonError::custom(err.to_string(), hex_span.clone()))),
                    // [error] negative number
                    common::Holder::Owned(JsonValue::Data(JsonType::Num(NumType::Integer(Integer::Negative(int_span))), _)) => Err(JsonError::custom(format!("cannot convert negative number to {}", stringify!($type)), int_span.clone())),
                    common::Holder::Borrow(JsonValue::Data(JsonType::Num(NumType::Integer(Integer::Negative(int_span))), _)) => Err(JsonError::custom(format!("cannot convert negative number to {}", stringify!($type)), int_span.clone())),
                    // [error] other
                    common::Holder::Owned(other_type) => Err(JsonError::custom(format!("cannot convert type {} to type {}", other_type.get_type_name(), stringify!($type)), other_type.get_span())),
                    common::Holder::Borrow(other_type) => Err(JsonError::custom(format!("cannot convert type {} to type {}", other_type.get_type_name(), stringify!($type)), other_type.get_span())),
                }
            }
        }
    };
    ($type:ty, $($types:ty),+) => {
        impl_unsigned_deserialization!($type);
        impl_unsigned_deserialization!($($types),+);
    };
}

impl_unsigned_deserialization!(u8, u16, u32, u64, usize);

impl Deserialize for f32 {
    fn parse(out: &JsonOutput<'_>) -> Result<Self, JsonError> {
        match &out.ast {
            common::Holder::Owned(JsonValue::Data(JsonType::Num(NumType::Decimal(_)), _)) => out.parse_type::<f32>(), 
            common::Holder::Borrow(JsonValue::Data(JsonType::Num(NumType::Decimal(_)), _)) => out.parse_type::<f32>(),
            common::Holder::Owned(other_type) => Err(JsonError::custom(format!("cannot convert type {} to type f32", other_type.get_type_name()), other_type.get_span())),
            common::Holder::Borrow(other_type) => Err(JsonError::custom(format!("cannot convert type {} to type f32", other_type.get_type_name()), other_type.get_span())),
        }
    }
}

impl Deserialize for f64 {
    fn parse(out: &JsonOutput<'_>) -> Result<Self, JsonError> {
        match &out.ast {
            common::Holder::Owned(JsonValue::Data(JsonType::Num(NumType::Decimal(_)), _)) => out.parse_type::<f64>(), 
            common::Holder::Borrow(JsonValue::Data(JsonType::Num(NumType::Decimal(_)), _)) => out.parse_type::<f64>(),
            common::Holder::Owned(other_type) => Err(JsonError::custom(format!("cannot convert type {} to type f64", other_type.get_type_name()), other_type.get_span())),
            common::Holder::Borrow(other_type) => Err(JsonError::custom(format!("cannot convert type {} to type f64", other_type.get_type_name()), other_type.get_span())),
        }
    }
}

impl Deserialize for String {
    fn parse(out: &JsonOutput<'_>) -> Result<Self, JsonError> {
        match &out.ast {
            common::Holder::Owned(JsonValue::Data(JsonType::Str(str_tokens), _)) => Ok(parse_str(out.parser, str_tokens)?),
            common::Holder::Borrow(JsonValue::Data(JsonType::Str(str_tokens), _)) => Ok(parse_str(out.parser, str_tokens)?),
            common::Holder::Owned(other_type) => Err(JsonError::invalid_type(other_type.get_span(), "String")),
            common::Holder::Borrow(other_type) => Err(JsonError::invalid_type(other_type.get_span(), "String")),
        }
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

// impl Deserialize for &[u8] {
//     fn parse(out: &JsonOutput<'_>) -> Result<Self, JsonError> {
//         match &out.ast {
//             common::Holder::Owned(t) => Ok(out.parser.take_raw(t.get_span())), 
//             common::Holder::Borrow(t) => Err(JsonError::empty_json(t.get_span())),
//         }
//     }
// }

#[inline(always)]
fn parse_properties_to_vec<T: Deserialize>(
    parser: &JsonParser<Tokenizer<'_>>,
    props: &[JsonProp<JsonInt>],
) -> Result<Vec<T>, JsonError> {
    props.iter()
        .map(|prop| JsonOutput::new(parser, &prop.value).parse_into::<T>())
        .collect()
}

// todo: look later for optimization
// currently heavy copy on source
fn parse_str(
    parser: &JsonParser<Tokenizer<'_>>,
    tokens: &Vec<StrType>,
) -> Result<String, JsonError> {
    let mut result = Vec::<&str>::new();

    for item in tokens {
        let token = item.parse_str(parser)?;
        result.push(token);
    }

    Ok(result.join(""))
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
        let source = r#"{ a: 1234567, b: " \"b\" " }"#;
        println!("{source}");
        let mut obj = JsonParser::new(source);
        let     ast = obj.parse().unwrap();
        let    item = ast.index("a").unwrap().parse_type::<usize>();
        assert_eq!(Ok(1234567), item);
        assert_eq!(Ok(" \"b\" ".to_string()), ast.index("b").unwrap().parse_into::<String>())
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

    #[test]
    fn parse_negative_number() -> crate::Result<()> {
        // let mut obj = JsonParser::new(r"{ negative: -1 }");
        // let out = obj.parse()?;

        // assert!(out.index("negative").unwrap().parse_into::<u8>().inspect_err(|err| println!("{err:?}")).is_err());

        Ok(())
    }
}