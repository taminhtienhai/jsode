#![allow(clippy::never_loop)]

use std::str::Chars;

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
    Bool,
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
}

impl Span {
    #[inline(always)]
    pub fn new(start: usize, end: usize) -> Self {
        Self { start, end }
    }

    #[inline(always)]
    pub fn with_counter(start: usize, counter: usize) -> Self {
        Self { start, end: start + counter }
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
    pub fn boolean(value: bool, at: usize) -> Self { Self::Data(JsonType::Bool, Span::new(at, at + 1)) }
    #[inline(always)]
    pub fn null(at: usize) -> Self { Self::Data(JsonType::Null, Span::new(at, at + 1)) }
    #[inline(always)]
    pub fn undefined(at: usize) -> Self { Self::Data(JsonType::Undefined, Span::new(at, at + 1)) }
    #[allow(non_snake_case)]
    #[inline(always)]
    pub fn NaN(at: usize) -> Self { Self::Data(JsonType::NaN, Span::new(at, at + 1)) }

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
    pub fn whitespace(at: usize) -> Self { Self::Punct(Punct::Comma, Span::new(at, at + 1)) }

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
            JsonToken::Data(_, Span { start, end, }) => end - start,
            JsonToken::Punct(_, Span { start, end }) => end - start,
            JsonToken::Error(_, _) => 0,
        }
    }
    pub fn parse_token(iter: &mut impl Iterator<Item = char>, at: usize, cached_next: &mut Option<char>) -> Option<(JsonToken, Option<char>)> {
        let mut end: usize = 0;
        let Some(next_item) = cached_next.take().or_else(|| iter.next()) else {
            return None;
        };
        match next_item {
            ' ' => JsonToken::whitespace(at).into(),
            '{' => JsonToken::open_curly(at).into(),
            '}' => JsonToken::close_curly(at).into(),
            '[' => JsonToken::open_square(at).into(),
            ']' => JsonToken::close_square(at).into(),
            ':' => JsonToken::colon(at).into(),
            ',' => JsonToken::comma(at).into(),
            '\'' => loop {
                end += 1;
                // parse all character wrapped inside single quote
                let Some(next_item) = iter.next() else {
                    break JsonToken::error("missing string's close character \'", at, end).into();
                };
                if next_item.eq(&'\'') {
                    break JsonToken::str(at, end).into()
                }

            },
            '"' => loop {
                end += 1;
                // parse all character wrapped inside double quote
                let Some(next_item) = iter.next() else {
                    break JsonToken::error("missing string's close character \"", at, end).into();
                };
                if next_item.eq(&'"') {
                    break JsonToken::str(at, end).into()
                }
            },
            //// true | false
            //// undefined
            //// NaN
            //// => keyword matching
            // 't' => loop {
            //     let r = iter.next();
            //     let u = iter.next();
            //     let e = iter.next();

            //     if !r.is_some() || !u.is_some() || !e.is_some() {
            //         break 
            //     }
            // },
            x if x.is_ascii_alphabetic() => loop {
                end += 1;
                // parse all character wrapped inside single quote
                let Some(next_item) = iter.next() else {
                    break JsonToken::error("missing string's close character \'", at, end).into();
                };
                if !next_item.is_ascii_alphabetic() {
                    break Some((JsonToken::ident(at, end), Some(next_item)))
                }
            },
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

///////////////

pub struct TokenIter<Iter: Iterator<Item = char>> {
    iter: Iter,
    pos: usize,
    _next_item: Option<char>,
}

impl <Iter: Iterator<Item = char>> Iterator for TokenIter<Iter> {
    type Item = JsonToken;

    fn next(&mut self) -> Option<Self::Item> {
        JsonToken::parse_token(&mut self.iter, self.pos, &mut self._next_item).map(|(tk, next_item)| {
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
    type IntoIter = TokenIter<Chars<'a>>;

    fn into_iter(self) -> Self::IntoIter {
        TokenIter {
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
        let mut tokens = Tokenizer::from("{ one:1}").into_iter();

        assert_eq!(Some(JsonToken::open_curly(0)) , tokens.next());
        assert_eq!(Some(JsonToken::whitespace(1)) , tokens.next());
        assert_eq!(Some(JsonToken::str(2, 3))     , tokens.next());
        assert_eq!(Some(JsonToken::colon(5))      , tokens.next());
        assert_eq!(Some(JsonToken::number(6,1))   , tokens.next());
        assert_eq!(Some(JsonToken::close_curly(7)), tokens.next());
        assert_eq!(None                           , tokens.next());
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