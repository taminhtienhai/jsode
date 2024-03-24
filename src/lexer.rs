#![allow(clippy::never_loop)]

use std::str::Chars;

use crate::constant;

pub struct JsonProp<'a> {
    pub key: &'a str,
    pub value: JsonType,
}

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

#[derive(Default, PartialEq, PartialOrd, Debug)]
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
}

#[derive(PartialEq, PartialOrd, Debug)]
pub enum JsonToken {
    Punct(Punct, Span),
    Data(JsonType, Span),
    Error(String, Span),
}

impl JsonToken {
    #[inline(always)]
    pub fn ident(start: usize, len: usize) -> Self { Self::Data(JsonType::Ident, Span::new(start, start + len)) }
    #[inline(always)]
    pub fn str(start: usize, len: usize) -> Self { Self::Data(JsonType::Str, Span::new(start, start + len)) }
    #[inline(always)]
    pub fn number(start: usize, len: usize) -> Self { Self::Data(JsonType::Num, Span::new(start, start + len)) }
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
    pub fn whitespace(at: usize, end: usize) -> Self { Self::Punct(Punct::WhiteSpace, Span::new(at, at + end)) }

    #[inline(always)]
    pub fn error(msg: impl Into<String>, start: usize, len: usize) -> Self { Self::Error(msg.into(), Span::new(start, start + len)) }
}

impl From<JsonToken> for Option<(JsonToken, Option<char>,)> {
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
    pub fn parse_token(iter: &mut impl Iterator<Item = char>, at: usize, cached_next: &mut Option<char>, src: &str) -> Option<(JsonToken, Option<char>)> {
        let mut end: usize = 0;
        let next_item = cached_next.take().or_else(|| iter.next())?;
        match next_item {
            // ws = *(
            //     %x20 /              ; Space
            //     %x09 /              ; Horizontal tab
            //     %x0A /              ; Line feed or New line
            //     %x0D )              ; Carriage return
            '\u{0020}' | '\u{0009}' | '\u{000A}' | '\u{000D}'  => loop {
                end += 1;
                // parse all character wrapped inside single quote
                let Some(next_item) = iter.next() else {
                    break JsonToken::whitespace(at, end).into()
                };
                if !matches!(next_item, '\u{0020}' | '\u{0009}' | '\u{000A}' | '\u{000D}') {
                    break Some((JsonToken::whitespace(at, end), Some(next_item)))
                }
            },
            '{' => JsonToken::open_curly(at).into(),
            '}' => JsonToken::close_curly(at).into(),
            '[' => JsonToken::open_square(at).into(),
            ']' => JsonToken::close_square(at).into(),
            ':' => JsonToken::colon(at).into(),
            ',' => JsonToken::comma(at).into(),
            // string and literal
            '\'' => loop {
                end += 1;
                // parse all character wrapped inside single quote
                let Some(next_item) = iter.next() else {
                    break JsonToken::error(constant::MISSING_SINGLE_COLON, at, end).into();
                };
                if next_item.eq(&'\'') {
                    break JsonToken::str(at, end).into()
                }

            },
            '"' => loop {
                end += 1;
                // parse all character wrapped inside double quote
                let Some(next_item) = iter.next() else {
                    break JsonToken::error(constant::MISSING_DOUBLE_COLON, at, end).into();
                };
                if next_item.eq(&'"') {
                    break JsonToken::str(at, end).into()
                }
            },
            // identity or keyword
            'a'..='z' | 'A'..='Z' | '_' => loop {
                end += 1;
                // parse all character wrapped inside single quote
                // todo: parse once more time because this ident potential to be a keyword (true, false, NaN, ...)
                let Some(next_item) = iter.next() else {
                    break JsonToken::parse_keyword(src, at, end).into()
                };
                if !matches!(next_item, 'a'..='z' | 'A'..='Z' | '_') {
                    break Some((JsonToken::parse_keyword(src, at, end), Some(next_item)))
                }
            },
            // number
            '0'..='9' => loop {
                end += 1;
                // parse all number include dot(.), exhauted when reaching none digit character
                let Some(next_item) = iter.next() else {
                    break JsonToken::number(at, end).into();
                };
                if !next_item.is_ascii_digit() {
                    break Some((JsonToken::number(at, end), Some(next_item)));
                }
            },
            _ => None
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

///////////////

pub struct TokenIter<'a, Iter: Iterator<Item = char>> {
    src: &'a str,
    iter: Iter,
    pos: usize,
    _next_item: Option<char>,
}

impl <'a, Iter: Iterator<Item = char>> Iterator for TokenIter<'a, Iter> {
    type Item = JsonToken;

    fn next(&mut self) -> Option<Self::Item> {
        JsonToken::parse_token(&mut self.iter, self.pos, &mut self._next_item, self.src).map(|(tk, next_item)| {
            if let Some(next) = next_item {
                self._next_item.replace(next);
            }
            self.pos += tk.size();
            tk
        })
    }
}

pub struct Tokenizer<'a> {
    source: &'a str,
}

impl <'a> From<&'a str> for Tokenizer<'a> {
    fn from(value: &'a str) -> Self {
        Self { source: value }
    }
}

impl <'a> IntoIterator for Tokenizer<'a> {
    type Item = JsonToken;
    type IntoIter = TokenIter<'a, Chars<'a>>;

    fn into_iter(self) -> Self::IntoIter {
        TokenIter {
            src: self.source,
            iter: self.source.chars(),
            pos: 0,
            _next_item: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    pub fn accept_slice_str() {
        let mut tokens = Tokenizer::from("{ one:1  }").into_iter();

        assert_eq!(Some(JsonToken::open_curly(0)) , tokens.next());
        assert_eq!(Some(JsonToken::whitespace(1,1)) , tokens.next());
        assert_eq!(Some(JsonToken::ident(2, 3))   , tokens.next());
        assert_eq!(Some(JsonToken::colon(5))      , tokens.next());
        assert_eq!(Some(JsonToken::number(6,1))   , tokens.next());
        assert_eq!(Some(JsonToken::whitespace(7,2)) , tokens.next());
        assert_eq!(Some(JsonToken::close_curly(9)), tokens.next());
        assert_eq!(None                           , tokens.next());
    }

    #[test]
    pub fn parse_keyword() {
        let mut kws = Tokenizer::from("true false NaN undefined").into_iter();
        
        assert_eq!(Some(JsonToken::boolean(true, 0)) , kws.next());
        assert_eq!(Some(JsonToken::whitespace(4,1))    , kws.next());
        assert_eq!(Some(JsonToken::boolean(false, 5)), kws.next());
        assert_eq!(Some(JsonToken::whitespace(10, 1))   , kws.next());
        assert_eq!(Some(JsonToken::NaN(11))          , kws.next());
        assert_eq!(Some(JsonToken::whitespace(14, 1))   , kws.next());
        assert_eq!(Some(JsonToken::undefined(15))    , kws.next());
        assert_eq!(None                              , kws.next());
    }

    #[test]
    pub fn parse_str_missing_() {
        let mut errors = Tokenizer::from("{ totally_error_str: 'abc }").into_iter();

        assert_eq!(Some(JsonToken::open_curly(0)) , errors.next());
        assert_eq!(Some(JsonToken::whitespace(1,1)) , errors.next());
        assert_eq!(Some(JsonToken::ident(2, 17))  , errors.next());
        assert_eq!(Some(JsonToken::colon(19))     , errors.next());
        assert_eq!(Some(JsonToken::whitespace(20,1)), errors.next());
        assert_eq!(Some(JsonToken::error(constant::MISSING_SINGLE_COLON, 21, 6)), errors.next());
        assert_eq!(None                           , errors.next());
    }

    #[test]
    fn peek_iter() {
        let src = "abcde";
        let mut iter = src.chars();

        let mut peek01 = iter.by_ref().peekable();

        assert_eq!(Some(&'a'), peek01.peek());
        assert_eq!(Some(&'a'), peek01.peek());
        assert_eq!(Some(&'a'), peek01.peek());
        assert_eq!(Some(&'a'), peek01.peek());

        let mut peek02 = iter.by_ref().peekable();

        assert_eq!(Some(&'b'), peek02.peek());
    }
}