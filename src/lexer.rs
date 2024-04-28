use std::{marker::PhantomData, ptr};
use crate::{constant, core::{JsonToken, Span}, error::JsonError};

#[derive(PartialEq, Debug)]
pub struct Tokenizer<'a> {
    ptr: *const u8,
    pos: usize,
    size: usize,
    _next_item: Option<u8>,
    _phantom: PhantomData<&'a [u8]>,
}

impl <'a> From<&'a str> for Tokenizer<'a> {
    fn from(slice: &'a str) -> Self {
        Self {
            ptr: slice.as_ptr(),
            pos: 0,
            size: slice.len(),
            _next_item: None,
            _phantom: PhantomData,
        }
    }
}

impl <'a> Iterator for Tokenizer<'a> {
    type Item = JsonToken;

    fn next(&mut self) -> Option<Self::Item> {
        self.parse_token().map(|(tk, next_item)| {
            if let Some(next) = next_item {
                self._next_item.replace(next);
            }
            tk
        })
    }
}

impl <'a> Tokenizer<'a> {
    #[inline(always)]
    pub fn prev_pos(&self) -> usize {
        self.pos - 1
    }

    #[inline(always)]
    pub fn cur_pos(&self) -> usize {
        self.pos
    }


    #[inline]
    pub fn take_raw(&self, span: Span) -> &[u8] {
        unsafe { std::slice::from_raw_parts(self.ptr.add(span.start), span.gap()) }
    }

    #[inline]
    pub fn take_slice(&self, span: Span) -> Result<&str, JsonError> {
        unsafe {
            let slice = std::slice::from_raw_parts(self.ptr.add(span.start), span.gap());
            std::str::from_utf8(slice)
                .map_err(|err| JsonError::custom(err.to_string(), span))
        }
    }

    #[inline]
    fn next_item(&mut self) -> Option<u8> {
        if self.pos >= self.size {
            None
        } else {
            let next_item = unsafe { ptr::read(self.ptr.add(self.pos)) };
            self.pos += 1;
            Some(next_item)
        }
    }

    #[inline]
    fn next_exact_until(&mut self, size: usize, predicate: impl Fn(u8) -> bool) -> Result<(), JsonError> {
        for n in 0..size {
            let Some(next_item) = self.next_item() else {
                return Err(JsonError::custom(
                    format!("[next_exact_until] soon EOF, expect {n} token more"),
                    Span::new(self.pos.saturating_sub(self.pos - n + 1), self.pos)));
            };

            if !predicate(next_item) {
                return Err(JsonError::custom(
                    "[next_exact_until] cannot satisfy condition",
                    Span::new(self.pos.saturating_sub(self.pos - n + 1), self.pos)))
            }
        }
        Ok(())
    }

    #[inline(always)]
    fn step_back(&mut self) {
        self.pos -= 1;
    }

    fn parse_keyword(&self, start: usize) -> JsonToken {
        let gap = self.pos - start;
        let buf = unsafe { std::slice::from_raw_parts(self.ptr.add(start), gap) };
        std::str::from_utf8(buf).map(|res| match res {
            "true"      => JsonToken::boolean(true, start),
            "false"     => JsonToken::boolean(false, start),
            "undefined" => JsonToken::undefined(start),
            x if x.eq_ignore_ascii_case("nan") => JsonToken::NaN(start),
            _           => JsonToken::ident(start, self.pos),
        }).unwrap_or_else(|err| JsonToken::error(err.to_string(), start, gap))
        
    }
}

impl <'a> Tokenizer<'a> {
    pub fn parse_token(&mut self) -> Option<(JsonToken, Option<u8>,)> {
        let at = self.pos;
        let next_item = self._next_item.take().inspect(|_| {
            self.pos += 1;
        }).or_else(|| self.next_item())?;

        match next_item {
            // ws = *(
            //     %x20 /              ; Space
            //     %x09 /              ; Horizontal tab
            //     %x0A /              ; Line feed or New line
            //     %x0D )              ; Carriage return
            constant::ascii::HORIZONTAL_TAB
            | constant::ascii::SPACE
            | constant::ascii::LINE_FEED
            | constant::ascii::CARRIAGE_RETURN
            | constant::ascii::FORM_FEED
            | constant::ascii::NON_BREAKING_SPACE
            | constant::ascii::BACKSPACE => loop {
                // parse all character wrapped inside single quote
                let Some(next_item) = self.next_item() else {
                    break JsonToken::whitespace(at, self.pos).into()
                };
                if !matches!(next_item, constant::ascii::HORIZONTAL_TAB
                    | constant::ascii::SPACE
                    | constant::ascii::LINE_FEED
                    | constant::ascii::CARRIAGE_RETURN
                    | constant::ascii::FORM_FEED
                    | constant::ascii::NON_BREAKING_SPACE
                    | constant::ascii::BACKSPACE) {
                    self.step_back();
                    break Some((JsonToken::whitespace(at, self.pos), Some(next_item)))
                }
            },
            b'{' => JsonToken::open_curly(at).into(),
            b'}' => JsonToken::close_curly(at).into(),
            b'[' => JsonToken::open_square(at).into(),
            b']' => JsonToken::close_square(at).into(),
            b':' => JsonToken::colon(at).into(),
            b',' => JsonToken::comma(at).into(),
            // string and literal
            b'\'' => loop {
                // parse all character wrapped inside single quote
                let Some(next_item) = self.next_item() else {
                    break JsonToken::error(constant::msg::MISSING_SINGLE_COLON, at, self.pos).into();
                };

                if next_item.eq(&constant::ascii::ESCAPE)  {
                    let Some(next_it) = self.next_item() else {
                        break JsonToken::error(constant::msg::MISSING_DOUBLE_COLON, at, self.pos).into();
                    };

                    // handle '\uXXXX'
                    if next_it.eq(&b'u') {
                        match self.next_exact_until(4, |item| matches!(item, b'0'..=b'9' | b'a'..=b'z' | b'A'..=b'Z')) {
                            Ok(_) => continue,
                            Err(err) => break JsonToken::from(err).into(),
                        }
                    }

                    // handle '\xXX'
                    if next_it.eq(&b'x') {
                        match self.next_exact_until(2, |item| matches!(item, b'0'..=b'9' | b'a'..=b'z' | b'A'..=b'Z')) {
                            Ok(_) => continue,
                            Err(err) => break JsonToken::from(err).into(),
                        }
                    }

                    if next_it.is_ascii_digit() {
                        return JsonToken::error(format!("{}: {}", constant::msg::INVALID_ESCAPE, char::from_u32(next_it as u32).unwrap()), at, self.pos).into();
                    }
                }

                if next_item.eq(&constant::ascii::SINGLE_QUOTE) {
                    break JsonToken::str(at, self.pos).into()
                }
            },
            b'"' => loop {
                // parse all character wrapped inside double quote
                let Some(next_item) = self.next_item() else {
                    break JsonToken::error(constant::msg::MISSING_DOUBLE_COLON, at, self.pos).into();
                };
                if next_item.eq(&constant::ascii::ESCAPE)  {
                    let Some(next_it) = self.next_item() else {
                        break JsonToken::error(constant::msg::MISSING_DOUBLE_COLON, at, self.pos).into();
                    };

                    // handle '\uXXXX'
                    if next_it.eq(&b'u') {
                        match self.next_exact_until(4, |item| matches!(item, b'0'..=b'9' | b'a'..=b'z' | b'A'..=b'Z')) {
                            Ok(_) => continue,
                            Err(err) => break JsonToken::from(err).into(),
                        }
                    }

                    // handle '\xXX'
                    if next_it.eq(&b'x') {
                        match self.next_exact_until(2, |item| matches!(item, b'0'..=b'9' | b'a'..=b'z' | b'A'..=b'Z')) {
                            Ok(_) => continue,
                            Err(err) => break JsonToken::from(err).into(),
                        }
                    }

                    if next_it.is_ascii_digit() {
                        return JsonToken::error(format!("{}: {}", constant::msg::INVALID_ESCAPE, char::from_u32(next_it as u32).unwrap()), at, self.pos).into();
                    }
                }
                if next_item.eq(&constant::ascii::DOUBLE_QUOTE) {
                    break JsonToken::str(at, self.pos).into()
                }
            },
            // number
            b'0'..=b'9' => loop {
                // parse all number include dot(.), exhauted when reaching none digit character
                let Some(next_item) = self.next_item() else {
                    break JsonToken::number(at, self.pos).into();
                };
                if !next_item.is_ascii_digit() {
                    self.step_back();
                    break Some((JsonToken::number(at, self.pos), Some(next_item)));
                }
            },
            // identity or keyword
            b'a'..=b'z' | b'A'..=b'Z' | b'_' => loop {
                let Some(next_item) = self.next_item() else {
                    break self.parse_keyword(at).into()
                };
                // accept ident has number in their name such as 'u8', 'u16'
                if !matches!(next_item, b'a'..=b'z' | b'A'..=b'Z' | b'0'..=b'9' | b'_') {
                    self.step_back();
                    break Some((self.parse_keyword(at), Some(next_item)))
                }
            },
            
            unknown_token => JsonToken::error(format!("{} {}", constant::msg::NOT_SUPPORT_TOKEN, char::from_u32(unknown_token as u32).unwrap()), at, self.pos).into()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    pub fn accept_slice_str() {
        let mut tokens = Tokenizer::from("{ one:1  }");

        assert_eq!(Some(JsonToken::open_curly(0))  , tokens.next());
        assert_eq!(Some(JsonToken::whitespace(1,2)), tokens.next());
        assert_eq!(Some(JsonToken::ident(2, 5))    , tokens.next());
        assert_eq!(Some(JsonToken::colon(5))       , tokens.next());
        assert_eq!(Some(JsonToken::number(6,7))    , tokens.next());
        assert_eq!(Some(JsonToken::whitespace(7,9)), tokens.next());
        assert_eq!(Some(JsonToken::close_curly(9)) , tokens.next());
        assert_eq!(None                            , tokens.next());
    }

    #[test]
    pub fn parse_keyword() {
        let mut kws = Tokenizer::from("true false NaN undefined");
        
        assert_eq!(Some(JsonToken::boolean(true, 0)) , kws.next());
        assert_eq!(Some(JsonToken::whitespace(4,5))  , kws.next());
        assert_eq!(Some(JsonToken::boolean(false, 5)), kws.next());
        assert_eq!(Some(JsonToken::whitespace(10,11)), kws.next());
        assert_eq!(Some(JsonToken::NaN(11))          , kws.next());
        assert_eq!(Some(JsonToken::whitespace(14,15)), kws.next());
        assert_eq!(Some(JsonToken::undefined(15))    , kws.next());
        assert_eq!(None                              , kws.next());
    }

    #[test]
    pub fn parse_str_missing_() {
        let mut errors = Tokenizer::from("{ totally_error_str: 'abc }");

        assert_eq!(Some(JsonToken::open_curly(0))  , errors.next());
        assert_eq!(Some(JsonToken::whitespace(1,2)), errors.next());
        assert_eq!(Some(JsonToken::ident(2, 19))   , errors.next());
        assert_eq!(Some(JsonToken::colon(19))      , errors.next());
        assert_eq!(Some(JsonToken::whitespace(20,21)), errors.next());
        assert_eq!(Some(JsonToken::error(constant::msg::MISSING_SINGLE_COLON, 21, 27)), errors.next());
        assert_eq!(None                            , errors.next());
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