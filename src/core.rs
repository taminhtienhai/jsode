use std::{any::TypeId, str::FromStr};

use crate::{deserialize::Deser, error::JsonError, lexer::Tokenizer, parser::JsonParser};

#[derive(PartialEq, PartialOrd, Debug)]
pub enum JsonType {
    Ident,
    // a string will start and end with symbol \' or \"
    Str,
    // continuously numbers, end when reaching symbol space or comma
    Num,
    // true | false literal, end with comma
    Bool(bool),
    // null
    Null,
    // undefined
    Undefined,
    // NaN ignorecase
    NaN,
}

#[derive(PartialEq, PartialOrd, Debug)]
pub enum Punct {
    DoubleQuote,
    SingleQuote,
    Comma      ,
    Colon      ,
    OpenSquare ,
    CloseSquare,
    OpenCurly  ,
    CloseCurly ,
    WhiteSpace ,
}

#[derive(Default, PartialEq, PartialOrd, Clone, Debug)]
pub struct Span {
    pub start: usize,
    pub end: usize,
    pub col: usize,
    pub row: usize,
}

impl Span {
    #[inline(always)]
    pub fn new(start: usize, end: usize) -> Self {
        Self { start, end, col: 0, row: 0 }
    }

    #[inline(always)]
    pub fn with_counter(start: usize, counter: usize) -> Self {
        Self { start, end: start + counter, col: 0, row: 0 }
    }

    #[inline(always)]
    pub fn gap(&self) -> usize {
        self.end - self.start
    }

    #[inline]
    pub fn extend(&self, other: Span) -> Span {
        Span::new(self.start, other.end)
    }

    #[inline]
    pub fn collapse(mut self, size: usize) -> Span {
        self.start += size;
        self.end -= size;
        self
    }
}

#[derive(PartialEq, PartialOrd, Debug)]
pub enum JsonToken {
    Punct(Punct, Span),
    Data(JsonType, Span),
    Error(String, Span),
}

impl JsonToken {
    #[inline(always)]
    pub fn ident(start: usize, end: usize) -> Self { Self::Data(JsonType::Ident, Span::new(start, end)) }
    #[inline(always)]
    pub fn str(start: usize, end: usize) -> Self { Self::Data(JsonType::Str, Span::new(start, end)) }
    #[inline(always)]
    pub fn number(start: usize, end: usize) -> Self { Self::Data(JsonType::Num, Span::new(start, end)) }
    #[inline(always)]
    pub fn boolean(value: bool, start: usize) -> Self { Self::Data(JsonType::Bool(value), Span::new(start, start + if value { 4 } else { 5 })) }
    #[inline(always)]
    pub fn null(at: usize) -> Self { Self::Data(JsonType::Null, Span::new(at, at + 1)) }
    #[inline(always)]
    pub fn undefined(at: usize) -> Self { Self::Data(JsonType::Undefined, Span::new(at, at + 9)) }
    #[allow(non_snake_case)]
    #[inline(always)]
    pub fn NaN(at: usize) -> Self { Self::Data(JsonType::NaN, Span::new(at, at + 3)) }

    #[inline(always)]
    pub fn open_curly(at: usize) -> Self { Self::Punct(Punct::OpenCurly, Span::new(at, at + 1)) }
    #[inline(always)]
    pub fn close_curly(at: usize) -> Self { Self::Punct(Punct::CloseCurly, Span::new(at, at + 1)) }
    #[inline(always)]
    pub fn open_square(at: usize) -> Self { Self::Punct(Punct::OpenSquare, Span::new(at, at + 1)) }
    #[inline(always)]
    pub fn close_square(at: usize) -> Self { Self::Punct(Punct::CloseSquare, Span::new(at, at + 1)) }
    #[inline(always)]
    pub fn single_quote(at: usize) -> Self { Self::Punct(Punct::SingleQuote, Span::new(at, at + 1)) }
    #[inline(always)]
    pub fn double_quote(at: usize) -> Self { Self::Punct(Punct::DoubleQuote, Span::new(at, at + 1)) }
    #[inline(always)]
    pub fn colon(at: usize) -> Self { Self::Punct(Punct::Colon, Span::new(at, at + 1)) }
    #[inline(always)]
    pub fn comma(at: usize) -> Self { Self::Punct(Punct::Comma, Span::new(at, at + 1)) }
    #[inline(always)]
    pub fn whitespace(at: usize, end: usize) -> Self { Self::Punct(Punct::WhiteSpace, Span::new(at, end)) }

    #[inline(always)]
    pub fn error(msg: impl Into<String>, start: usize, end: usize) -> Self { Self::Error(msg.into(), Span::new(start, end)) }
}

impl From<JsonToken> for Option<(JsonToken, Option<u8>,)> {
    fn from(value: JsonToken) -> Self {
        Some((value, None,))
    }
}

impl JsonToken {
    pub fn size(&self) -> usize {
        match self {
            JsonToken::Data(_, Span { start, end, .. }) => end - start,
            JsonToken::Punct(_, Span { start, end, .. }) => end - start,
            JsonToken::Error(_, _) => 0,
        }
    }
}

impl JsonToken {
    pub fn parse_keyword(src: &str, start: usize, end: usize) -> JsonToken {
        match &src[start..start + end] {
            "true"      => JsonToken::boolean(true, start),
            "false"     => JsonToken::boolean(false, start),
            "undefined" => JsonToken::undefined(start),
            x if x.eq_ignore_ascii_case("nan") => JsonToken::NaN(start),
            _           => JsonToken::ident(start, end),
        }
    }
}

/////////////////////
pub trait JsonKey {}

#[derive(Default)]
pub struct JsonStr(pub Span);
#[derive(Default)]
pub struct JsonInt(pub usize);

impl JsonKey for JsonStr {}
impl JsonKey for JsonInt {}

// JsonProp should be applied to both JsonArray and JsonObject
// **JsonObject**:
// For given json "{ name: 'haitmt', age: 25 }", output will be:
// JsonObject { properties: [ JsonProp { key: 'name', value: 'haitmt' }, JsonProp { key: 'age', value: 18 } ] }
//
// **JsonArray**:
// For given json "[{ id: 1 }, { id: 2 }]", output will be:
// JsonArray { properties:  }
pub struct JsonProp<K: JsonKey> {
    pub key: K,
    pub value: JsonValue,
}

impl <K: JsonKey> JsonProp<K> {
    pub fn new(k: K, v: JsonValue) -> Self {
        Self { key: k, value: v }
    }
}

pub struct JsonObject {
    properties: Vec<JsonProp<JsonStr>>,
    span: Span,
}

impl JsonObject {
    pub fn new(properties: Vec<JsonProp<JsonStr>>, span: Span) -> Self {
        Self {
            properties,
            span,
        }
    }
}

pub struct JsonArray {
    properties: Vec<JsonProp<JsonInt>>,
    span: Span,
}

impl JsonArray {
    pub fn new(properties: Vec<JsonProp<JsonInt>>, span: Span) -> Self {
        Self {
            properties,
            span,
        }
    }
}

pub enum JsonValue {
    Object(JsonObject),
    Array(JsonArray),
    Data(JsonType, Span),
}

impl JsonValue {
    pub fn get_span(&self) -> Span {
        match self {
            Self::Object(JsonObject { span, .. }) => span.clone(),
            Self::Array(JsonArray { span, .. }) => span.clone(),
            Self::Data(_, span) => span.clone(),
        }
    }
}

#[derive(PartialEq, Debug)]
pub struct JsonOutput<'a> {
    // span: Span,
    raw: &'a str,
}

impl <'a> JsonOutput<'a> {
    // pub fn new(raw: &'a str, span: Span) -> Self {
    //     Self { raw, span, }
    // }
    pub fn new(raw: &'a str) -> Self {
        Self { raw }
    }
}

impl <'out> JsonOutput<'out> {
    pub fn parse<T: FromStr>(&self) -> Result<T, JsonError> {
        self.raw.parse::<T>().map_err(|_| JsonError::custom("not correct type", Span::default()))
    }
} 

pub trait JsonIndex {
    type Output<'out>;
    fn get_object_key<'obj>(&self, key: &'obj str, indexer: &'obj JsonParser<Tokenizer<'obj>>) -> Self::Output<'obj>;
    fn get_array_item<'arr>(&self, key: usize, indexer: &'arr JsonParser<Tokenizer<'arr>>) -> Self::Output<'arr>;
}

impl JsonIndex for JsonValue {
    type Output<'out> = Option<JsonOutput<'out>>;

    fn get_object_key<'o>(&self, key: &'o str, indexer: &'o JsonParser<Tokenizer<'o>>) -> Self::Output<'o> {
        match self {
            Self::Object(obj) => {
                for prop in obj.properties.iter() {
                    // warn: skip error here
                    let Ok(inner_key) = indexer.take_slice(prop.key.0.clone()) else {
                        return None;
                    };
                    if key.eq(inner_key) {
                        let prop_span = prop.value.get_span();
                        let Ok(raw) = indexer.take_slice(prop_span) else {
                            return None;
                        };
                        return Some(JsonOutput::new(raw));
                    }
                }
                None
            },
            _ => None,
        }
    }
    
    fn get_array_item<'a>(&self, key: usize, indexer: &'a JsonParser<Tokenizer<'a>>) -> Self::Output<'a> {
        match self {
            Self::Array(arr) => {
                if key >= arr.properties.len() {
                    return None;
                }
                let item = arr.properties.get(key)?;
                let item_span = item.value.get_span();
                let Ok(raw) = indexer.take_slice(item_span) else {
                    return None;
                };
                return Some(JsonOutput::new(raw));

            },
            _ => None,
        }
    }
}