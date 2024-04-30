use std::{collections::HashMap, fmt::Display, hash::Hash};

use crate::{common::{self, Holder}, error::JsonError, lexer::Tokenizer, parser::JsonParser};

#[derive(PartialEq, PartialOrd, Debug)]
pub enum JsonType {
    Ident,
    // a string will start and end with symbol \' or \"
    Str(Vec<StrType>),
    // continuously numbers, end when reaching symbol space or comma
    Num(NumType),
    // true | false literal, end with comma
    Bool(bool),
    // null
    Null,
}

impl JsonType {
    pub const fn get_type_name(&self) -> &str {
        match self {
            Self::Bool(_) => "boolean",
            Self::Ident => "ident",
            Self::Num(num) => num.get_type_name(),
            Self::Str(_) => "string",
            Self::Null => "null",
        }
    }
}

#[derive(PartialEq, PartialOrd, Debug, Clone)]
pub enum StrType {
    Str(Span),
    // \xXX (U+0000 through U+00FF)
    Ascii(Span),
    // \uXXXX (U+0000 through U+FFFF)
    Unicode(Span),
    // \X
    Escape(Span),
    // \'	Apostrophe	    U+0027
    // \"	Quotation mark	U+0022
    // \\	Reverse solidus	U+005C
    // \b	Backspace	    U+0008
    // \f	Form feed	    U+000C
    // \n	Line feed	    U+000A
    // \r	Carriage return	U+000D
    // \t	Horizontal tab	U+0009
    // \v	Vertical tab	U+000B
    // \0	Null	        U+0000
    Special(Span),
}

impl StrType {
    pub const fn get_type_name(&self) -> &str {
        match self {
            Self::Ascii(_) => "ascii",
            Self::Escape(_) => "escape",
            Self::Special(_) => "special",
            Self::Unicode(_) => "unicode",
            Self::Str(_) => "normal str",
        }
    }
}

#[derive(PartialEq, PartialOrd, Debug, Clone)]
pub enum NumType {
    // 123
    // -123
    Integer(Integer),
    // 123.456
    // -123.456
    // .456 = 0.456
    Decimal(Decimal),
    // 123e-456
    // -123e-456
    Exponential(Span),
    // 0xdecaf
    // -0xC0FFEE
    Hex(Heximal),
    // Infinity
    Infinity(Span),
    // NaN
    NaN(Span),
}

impl NumType {
    pub const fn get_type_name(&self) -> &str {
        match self {
            Self::Integer(_) => "integer",
            Self::Decimal(_) => "decimal",
            Self::Exponential(_) => "exponential",
            Self::Hex(_) => "hexadecimal",
            Self::Infinity(_) => "infinity",
            Self::NaN(_) => "NaN",
        }
    }
}

#[derive(PartialEq, PartialOrd, Debug, Clone)]
pub enum Integer {
    Positive(Span),
    Negative(Span),
}

#[derive(PartialEq, PartialOrd, Debug, Clone)]
pub enum Heximal {
    Positive(Span),
    Negative(Span),
}

#[derive(PartialEq, PartialOrd, Debug, Clone)]
pub enum Decimal {
    Positive(Option<Span>, Option<Span>),
    Negative(Option<Span>, Option<Span>),
}

impl From<Decimal> for NumType {
    fn from(value: Decimal) -> Self {
        NumType::Decimal(value)
    }
}

impl From<Integer> for NumType {
    fn from(value: Integer) -> Self {
        NumType::Integer(value)
    }
}

impl From<Heximal> for NumType {
    fn from(value: Heximal) -> Self {
        NumType::Hex(value)
    }
}

#[derive(PartialEq, PartialOrd, Debug, Clone)]
pub enum Sign {
    Plus,
    Minus,
    None,
}

impl Sign {
    pub const fn detect(sign: u8) -> Self {
        match sign {
            43 => Self::Plus,
            45 => Self::Minus,
            _ => Self::None,
        }
    }

    pub fn to_hexadecimal(&self, start: usize, end: usize) -> NumType {
        match self {
            Self::Plus => Heximal::Positive(Span::new(start - 1, end)).into(),
            Self::Minus => Heximal::Negative(Span::new(start - 1, end)).into(),
            _ => Heximal::Positive(Span::new(start, end)).into(),
        }
    }

    pub fn to_integer(&self, start: usize, end: usize) -> NumType {
        match self {
            Self::Plus => Integer::Positive(Span::new(start - 1, end)).into(),
            Self::Minus => Integer::Negative(Span::new(start - 1, end)).into(),
            _ => Integer::Positive(Span::new(start, end)).into(),
        }
    }

    pub fn to_decimal(&self, int_span: Option<Span>, frac_span: Option<Span>) -> NumType {
        match self {
            Self::Plus => Decimal::Positive(int_span.map(|mut sp| {
                sp.start -= 1;
                sp
            }), frac_span).into(),
            Self::Minus => Decimal::Negative(int_span.map(|mut sp| {
                sp.start -= 1;
                sp
            }), frac_span).into(),
            _ => Decimal::Positive(int_span, frac_span).into(),
        }
    }
}

impl StrType {
    pub(crate) fn parse_str<'a>(&'a self, parser: &'a JsonParser<Tokenizer<'a>>) -> Result<&'a str> {
        match self {
            Self::Str(span) => Ok(parser.take_slice(Span::new(span.start, span.end))?),
            Self::Ascii(span) => Ok(parser.take_slice(Span::new(span.start + 2, span.end))?),
            Self::Unicode(span) => Ok(parser.take_slice(Span::new(span.start + 2, span.end))?),
            Self::Escape(span) => Ok(parser.take_slice(Span::new(span.start + 1, span.end))?),
            Self::Special(span) => {
                let raw = parser.take_raw(span.clone());
                let item = map_special_char(raw[0]);
                Ok(item)
            },
        }
    }
}

const fn map_special_char<'a>(c: u8) -> &'a str {
    match c {
        b'\'' => "\'",
        b'\"' => "\"",
        b'\\' => "\\",
        b'b'  => "\u{08}",
        b'f'  => "\u{0C}",
        b'n'  => "\u{0A}",
        b'r'  => "\u{0D}",
        b't'  => "\u{09}",
        b'v'  => "\u{0B}",
        b'0'  => "\u{00}",
        _ => "\u{00}",
    }
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
    Plus       ,
    Minus      ,
}

#[derive(Default, PartialEq, Eq, PartialOrd, Hash, Clone, Debug)]
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
    pub fn extend(&self, other: Span) -> Self {
        Span::new(self.start, other.end)
    }

    #[inline(always)]
    pub fn collapse(mut self, size: usize) -> Self {
        self.start += size;
        self.end -= size;
        self
    }

    #[inline(always)]
    pub fn shrink_left(mut self, size: usize) -> Self {
        self.start += size;
        self
    }

    #[inline(always)]
    pub fn shrink_right(mut self, size: usize) -> Self {
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
    pub fn str(str_tokens: Vec<StrType>, start: usize, end: usize) -> Self { Self::Data(JsonType::Str(str_tokens), Span::new(start, end)) }
    #[inline(always)]
    pub fn number(ty: NumType, start: usize, end: usize) -> Self { Self::Data(JsonType::Num(ty), Span::new(start, end)) }
    #[inline(always)]
    pub fn boolean(value: bool, start: usize) -> Self { Self::Data(JsonType::Bool(value), Span::new(start, start + if value { 4 } else { 5 })) }
    #[inline(always)]
    pub fn null(at: usize) -> Self { Self::Data(JsonType::Null, Span::new(at, at + 1)) }
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
    pub fn plus(at: usize, end: usize) -> Self { Self::Punct(Punct::Plus, Span::new(at, end)) }
    #[inline(always)]
    pub fn minus(at: usize, end: usize) -> Self { Self::Punct(Punct::Minus, Span::new(at, end)) }

    #[inline(always)]
    pub fn error(msg: impl Into<String>, start: usize, end: usize) -> Self { Self::Error(msg.into(), Span::new(start, end)) }
}

impl From<JsonError> for JsonToken {
    fn from(value: JsonError) -> Self {
        JsonToken::Error(value.to_string(), value.span)
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
            _           => JsonToken::ident(start, end),
        }
    }

    pub fn get_span(&self) -> Span {
        match self {
            Self::Data(_, span) => span.clone(),
            Self::Punct(_, span) => span.clone(),
            Self::Error(_, span) => span.clone(),
        }
    }
}


pub trait JsonQuery {
    type Out<'out> where Self: 'out;
    fn query(&self, command: &str) -> Self::Out<'_>;
}

#[derive(PartialEq, Debug)]
pub struct JsonOutput<'p> {
    pub(crate) parser: &'p JsonParser<Tokenizer<'p>>,
    pub(crate) ast: Holder<'p, JsonValue>,
}

impl <'p> JsonOutput<'p> {
    pub fn new(parser: &'p JsonParser<Tokenizer<'p>>, ast: impl Into<Holder<'p, JsonValue>>) -> Self {
        Self { parser, ast: ast.into(), }
    }

    #[inline]
    pub(crate) fn parse_type<T: std::str::FromStr>(&self) -> Result<T>
        where T::Err: Display {
        let span = self.ast.as_ref().get_span();
        let slice = self.parser.take_slice(span.clone())?;
        slice.parse::<T>().map_err(|err| JsonError::custom(format!("{err}"), span))
    }

    #[inline]
    pub(crate) fn parse_type_span<T: std::str::FromStr>(&self, span: Span) -> Result<T>
        where T::Err: Display {
        let slice = self.parser.take_slice(span.clone())?;
        slice.parse::<T>().map_err(|err| JsonError::custom(format!("{err}"), span))
    }

    #[inline]
    pub fn to_slice(&self) -> Result<&str> {
        self.parser.take_slice(self.ast.as_ref().get_span())
    }

    #[inline]
    pub fn to_slice_span(&self, span: Span) -> Result<&str> {
        self.parser.take_slice(span)
    }

    pub fn to_bytes(&self) -> &[u8] {
        match &self.ast {
            common::Holder::Owned(t) => self.parser.take_raw(t.get_span()), 
            common::Holder::Borrow(t) => self.parser.take_raw(t.get_span()),
        }
    }
}

pub trait JsonKey: Hash {}

#[derive(PartialEq, Eq, Hash, Default, Debug)]
pub struct JsonStr(pub Span);
#[derive(PartialEq, Hash, Default, Debug)]
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
#[derive(PartialEq, Debug)]
pub struct JsonProp<K: JsonKey> {
    pub(crate) key: K,
    pub(crate) value: JsonValue,
}

impl <K: JsonKey> JsonProp<K> {
    pub fn new(k: K, v: JsonValue) -> Self {
        Self { key: k, value: v }
    }
}

#[derive(PartialEq, Debug)]
pub struct JsonObject {
    pub(crate) properties: HashMap<u64, JsonProp<JsonStr>>,
    span: Span,
}

impl JsonObject {
    pub fn new(properties: HashMap<u64, JsonProp<JsonStr>>, span: Span) -> Self {
        Self {
            properties,
            span,
        }
    }
}

#[derive(PartialEq, Debug)]
pub struct JsonArray {
    pub(crate) properties: Vec<JsonProp<JsonInt>>,
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

#[derive(PartialEq, Debug)]
pub enum JsonValue {
    Object(JsonObject),
    Array(JsonArray),
    Data(JsonType, Span),
}

impl JsonValue {
    #[inline]
    pub fn get_span(&self) -> Span {
        match self {
            Self::Object(JsonObject { span, .. }) => span.clone(),
            Self::Array(JsonArray { span, .. }) => span.clone(),
            Self::Data(JsonType::Str(_), span) => span.clone().collapse(1),
            Self::Data(_, span) => span.clone(),
        }
    }

    #[inline(always)]
    pub const fn get_type_name(&self) -> &str {
        match self {
            Self::Object(_) => "object",
            Self::Array(_) => "array",
            Self::Data(data_ty, __not_exist__) => data_ty.get_type_name()
        }
    }
}

pub type Result<T> = core::result::Result<T, JsonError>;