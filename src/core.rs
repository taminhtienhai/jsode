use std::{collections::HashMap, fmt::Display, hash::Hash};

use jsode_macro::reflection;

use crate::{common::Arrice, error::JsonError, parser::JsonParser};

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
    // 123e10
    // 123e+10
    // 123e-10
    Integer(Integer),
    // 123.456
    // -123.456
    // 12.3e456
    // -12.3e-456
    // .456
    // 0.456e2
    Decimal(Decimal),
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
            Self::Hex(_) => "hexadecimal",
            Self::Infinity(_) => "infinity",
            Self::NaN(_) => "NaN",
        }
    }
}

#[derive(PartialEq, PartialOrd, Debug, Clone)]
pub enum Integer {
    // given integer -123e-456
    // 1: -123
    // 2: -456 (exponent)
    Positive(Span, Option<Span>),
    Negative(Span, Option<Span>),
}

#[derive(PartialEq, PartialOrd, Debug, Clone)]
pub enum Heximal {
    // given hexa 0x2E
    // 1: 0x
    // 2: 2E
    Positive(Span, Span),
    Negative(Span, Span),
}

#[derive(PartialEq, PartialOrd, Debug, Clone)]
pub enum Decimal {
    // given decimal -123.456e-789
    // 1: -123
    // 2: 456 (fracment)
    // 3: -789 (exponent)
    Positive(Option<Span>, Option<Span>, Option<Span>),
    Negative(Option<Span>, Option<Span>, Option<Span>),
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

    #[rustfmt::skip]
    pub const fn compute_start_pos(&self, start: usize) -> usize {
        // given a number ?123 (? is '+' or '-')
        // if number has a prefix, move the position backward
        match self {
            // +123
            Self::Plus  => start - 1,
            // -123
            Self::Minus => start - 1,
            // 123
            _           => start,
        }
    }

    pub const fn to_hexadecimal(&self, prefix: Span, suffix: Span) -> NumType {
        match self {
            Self::Plus => NumType::Hex(Heximal::Positive(prefix.expand_left(1), suffix)),
            Self::Minus => NumType::Hex(Heximal::Negative(prefix.expand_left(1), suffix)),
            _ => NumType::Hex(Heximal::Positive(prefix, suffix)),
        }
    }

    pub const fn to_integer(&self, start: usize, end: usize, expo_span: Option<Span>) -> NumType {
        match self {
            Self::Plus => NumType::Integer(Integer::Positive(Span::new(start - 1, end), expo_span)),
            Self::Minus => NumType::Integer(Integer::Negative(Span::new(start - 1, end), expo_span)),
            _ => NumType::Integer(Integer::Positive(Span::new(start, end), expo_span)),
        }
    }

    pub const fn to_decimal(&self, int_span: Option<Span>, frac_span: Option<Span>, expo_span: Option<Span>) -> NumType {
        let new_int_span = if let Some(integer_span) = int_span { Some(integer_span.expand_left(1)) } else { None };
        match self {
            Self::Plus => NumType::Decimal(Decimal::Positive(new_int_span, frac_span, expo_span)),
            Self::Minus => NumType::Decimal(Decimal::Negative(new_int_span, frac_span, expo_span)),
            _ => NumType::Decimal(Decimal::Positive(new_int_span, frac_span, expo_span)),
        }
    }
}

impl StrType {
    pub(crate) fn parse_str<'a>(&'a self, parser: &'a JsonParser<'a>) -> Result<&'a str> {
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
    pub const fn new(start: usize, end: usize) -> Self {
        Self { start, end, col: 0, row: 0 }
    }

    #[inline(always)]
    pub const fn with_counter(start: usize, counter: usize) -> Self {
        Self { start, end: start + counter, col: 0, row: 0 }
    }

    #[inline(always)]
    pub const fn gap(&self) -> usize {
        self.end - self.start
    }

    #[inline]
    pub const fn extend(&self, other: Span) -> Self {
        Span::new(self.start, other.end)
    }

    #[inline(always)]
    pub const fn collapse(mut self, size: usize) -> Self {
        self.start += size;
        self.end -= size;
        self
    }

    #[inline(always)]
    pub const fn shrink_left(mut self, size: usize) -> Self {
        self.start += size;
        self
    }

    #[inline(always)]
    pub const fn shrink_right(mut self, size: usize) -> Self {
        self.end -= size;
        self
    }

    #[inline(always)]
    pub const fn expand_left(mut self, size: usize) -> Self {
        self.start -= size;
        self
    }

    #[inline(always)]
    pub const fn expand_right(mut self, size: usize) -> Self {
        self.end += size;
        self
    }
}

#[derive(PartialEq, PartialOrd, Debug)]
pub enum JsonToken {
    Punct(Punct, Span),
    Data(JsonType, Span),
    Error(String, Span),
    Comment(Span),
}

impl JsonToken {
    #[inline(always)]
    pub const fn ident(start: usize, end: usize) -> Self { Self::Data(JsonType::Ident, Span::new(start, end)) }
    #[inline(always)]
    pub const fn str(str_tokens: Vec<StrType>, start: usize, end: usize) -> Self { Self::Data(JsonType::Str(str_tokens), Span::new(start, end)) }
    #[inline(always)]
    pub const fn number(ty: NumType, start: usize, end: usize) -> Self { Self::Data(JsonType::Num(ty), Span::new(start, end)) }
    #[inline(always)]
    pub const fn boolean(value: bool, start: usize) -> Self { Self::Data(JsonType::Bool(value), Span::new(start, start + if value { 4 } else { 5 })) }
    #[inline(always)]
    pub const fn null(at: usize) -> Self { Self::Data(JsonType::Null, Span::new(at, at + 1)) }
    #[inline(always)]
    pub const fn open_curly(at: usize) -> Self { Self::Punct(Punct::OpenCurly, Span::new(at, at + 1)) }
    #[inline(always)]
    pub const fn close_curly(at: usize) -> Self { Self::Punct(Punct::CloseCurly, Span::new(at, at + 1)) }
    #[inline(always)]
    pub const fn open_square(at: usize) -> Self { Self::Punct(Punct::OpenSquare, Span::new(at, at + 1)) }
    #[inline(always)]
    pub const fn close_square(at: usize) -> Self { Self::Punct(Punct::CloseSquare, Span::new(at, at + 1)) }
    #[inline(always)]
    pub const fn single_quote(at: usize) -> Self { Self::Punct(Punct::SingleQuote, Span::new(at, at + 1)) }
    #[inline(always)]
    pub const fn double_quote(at: usize) -> Self { Self::Punct(Punct::DoubleQuote, Span::new(at, at + 1)) }
    #[inline(always)]
    pub const fn colon(at: usize) -> Self { Self::Punct(Punct::Colon, Span::new(at, at + 1)) }
    #[inline(always)]
    pub const fn comma(at: usize) -> Self { Self::Punct(Punct::Comma, Span::new(at, at + 1)) }
    #[inline(always)]
    pub const fn whitespace(at: usize, end: usize) -> Self { Self::Punct(Punct::WhiteSpace, Span::new(at, end)) }
    #[inline(always)]
    pub const fn plus(at: usize, end: usize) -> Self { Self::Punct(Punct::Plus, Span::new(at, end)) }
    #[inline(always)]
    pub const fn minus(at: usize, end: usize) -> Self { Self::Punct(Punct::Minus, Span::new(at, end)) }
    #[inline(always)]
    pub const fn comment(at: usize, end: usize) -> Self { Self::Comment(Span::new(at, end)) }

    #[inline(always)]
    pub fn error(msg: impl Into<String>, start: usize, end: usize) -> Self { Self::Error(msg.into(), Span::new(start, end)) }
}

impl From<JsonError> for JsonToken {
    fn from(value: JsonError) -> Self {
        JsonToken::Error(value.to_string(), value.span)
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

    pub const fn get_span(&self) -> Span {
        match self {
            Self::Data(_, span) => Span::new(span.start, span.end),
            Self::Punct(_, span) => Span::new(span.start, span.end),
            Self::Error(_, span) => Span::new(span.start, span.end),
            Self::Comment(span) => Span::new(span.start, span.end),
        }
    }
}

#[derive(PartialEq, Debug)]
pub struct JsonBlock {
    pub(crate) level: usize,
    pub(crate) value: JsonValue,
}

impl Default for JsonBlock {
    fn default() -> Self {
        Self { level: 0, value: JsonValue::Array(Vec::with_capacity(10), Span::default()) }
    }
}

impl JsonBlock {
    pub fn new(level: usize, value: JsonValue) -> Self {
        Self { level, value }
    }

    #[inline]
    pub(crate) fn parse_type<T: std::str::FromStr>(&self, parser: &JsonParser<'_>) -> Result<T>
        where T::Err: Display {
        let span = self.value.get_span();
        let slice = parser.take_slice(span.clone())?;
        slice.parse::<T>().map_err(|err| JsonError::custom(format!("{err}"), span))
    }

    #[inline]
    pub(crate) fn parse_type_span<T: std::str::FromStr>(&self, parser: &JsonParser<'_>, span: Span) -> Result<T>
        where T::Err: Display {
        let slice = parser.take_slice(span.clone())?;
        slice.parse::<T>().map_err(|err| JsonError::custom(format!("{err}"), span))
    }

    #[inline]
    pub fn to_slice<'a>(&'a self, parser: &'a JsonParser<'a>) -> Result<&str> {
        let span = self.value.get_span();
        parser.take_slice(span)
    }

    pub const fn to_bytes<'a>(&'a self, parser: &'a JsonParser<'a>) -> &[u8] {
        let span = self.value.get_span();
        parser.take_raw(span)
    }
}

#[derive(PartialEq, Debug)]
pub struct JsonOutput<'out> {
    pub(crate) parser: &'out JsonParser<'out>,
    pub(crate) ast: Arrice<'out, JsonBlock>,
}

impl <'out> JsonOutput<'out> {
    pub fn new(parser: &'out JsonParser<'out>, ast: impl Into<Arrice<'out, JsonBlock>>) -> Self {
        Self { parser, ast: ast.into(), }
    }

    #[inline]
    #[reflection]
    pub(crate) fn parse_type<T: std::str::FromStr>(&self) -> Result<T>
        where T::Err: Display {
        self.ast.as_slice().first()
            .map(|it| it.parse_type(self.parser))
            .ok_or(JsonError::custom(format!("[{__fn_ident}] Soon EOF"), Span::default()))?
    }

    #[inline]
    #[reflection]
    pub(crate) fn parse_type_span<T: std::str::FromStr>(&self, span: Span) -> Result<T>
        where T::Err: Display {
        self.ast.as_slice().first()
            .map(|it| it.parse_type_span(self.parser, span))
            .ok_or(JsonError::custom(format!("[{__fn_ident}] Soon EOF"), Span::default()))?
    }

    #[inline]
    #[reflection]
    pub fn to_slice(&self) -> Result<&str> {
        self.ast.as_slice().first()
            .map(|it| it.to_slice(self.parser))
            .ok_or(JsonError::custom(format!("[{__fn_ident}] Soon EOF"), Span::default()))?
    }

    #[inline]
    pub fn to_slice_span(&self, span: Span) -> Result<&str> {
        self.parser.take_slice(span)
    }

    pub fn to_bytes(&self) -> Result<&[u8]> {
        self.ast.as_slice().first()
            .map(|it| Ok(it.to_bytes(self.parser)))
            .ok_or(JsonError::custom("msg", Span::default()))?
    }
}

#[derive(PartialEq, Debug)]
pub enum JsonValue {
    Object(HashMap<usize,usize>, Span),
    Array(Vec<usize>, Span),
    // given prop `year: 2024`
    // JsonType - type of value (Number in this example)
    // Span - span of value (span of `2024` in this example)
    // Span - span of whole property
    Prop(JsonType, Span, Span),
    Value(JsonType, Span),
}

impl JsonValue {
    #[inline]
    pub const fn get_span(&self) -> Span {
        match self {
            Self::Object(_, span) => Span::new(span.start, span.end),
            Self::Array(_, span) => Span::new(span.start, span.end),
            Self::Prop(_, span, _) => Span::new(span.start, span.end),
            Self::Value(_, span) => Span::new(span.start, span.end),
        }
    }

    #[inline(always)]
    pub const fn get_type_name(&self) -> &str {
        match self {
            Self::Object(_,_) => "object",
            Self::Array(_,_) => "array",
            Self::Prop(data_ty,_,_) => data_ty.get_type_name(),
            Self::Value(data_ty, _) => data_ty.get_type_name()
        }
    }
}

pub type Result<T> = core::result::Result<T, JsonError>;