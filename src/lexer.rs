use std::{marker::PhantomData, ptr};
use crate::{constant, core::{Decimal, Integer, JsonToken, Sign, Span, StrType}, error::JsonError};

#[derive(PartialEq, Debug)]
pub struct Tokenizer<'a> {
    ptr: *const u8,
    pos: usize,
    size: usize,
    _phantom: PhantomData<&'a [u8]>,
}

impl <'a> From<&'a str> for Tokenizer<'a> {
    fn from(slice: &'a str) -> Self {
        Self {
            ptr: slice.as_ptr(),
            pos: 0,
            size: slice.len(),
            _phantom: PhantomData,
        }
    }
}

impl <'a> Iterator for Tokenizer<'a> {
    type Item = JsonToken;

    fn next(&mut self) -> Option<Self::Item> {
        self.parse_token()
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
    fn peek_prev_item(&self) -> Option<u8> {
        if self.pos > 0 {
            let prev_item = unsafe { ptr::read(self.ptr.add(self.pos - 2)) };
            Some(prev_item)
        } else {
            None
        }
    }

    #[inline]
    fn peek_next_item(&self) -> Option<u8> {
        if self.pos >= self.size {
            None
        } else {
            let next_item = unsafe { ptr::read(self.ptr.add(self.pos)) };
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

    fn next_until(&mut self, predicate: impl Fn(u8) -> bool) -> Span {
        let start: usize = self.pos - 1;
        loop {
            let Some(next_item) = self.next_item() else {
                break;
            };

            if predicate(next_item) {
                self.step_back();
                break;
            }
        }
        Span::new(start, self.pos)
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
            _           => JsonToken::ident(start, self.pos),
        }).unwrap_or_else(|err| JsonToken::error(err.to_string(), start, gap))
        
    }
}

impl <'a> Tokenizer<'a> {
    #[inline(always)]
    pub fn parse_token(&mut self) -> Option<JsonToken> {
        let at = self.pos;
        let next_item = self.next_item()?;

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
                    break Some(JsonToken::whitespace(at, self.pos))
                }
            },
            b'{' => JsonToken::open_curly(at).into(),
            b'}' => JsonToken::close_curly(at).into(),
            b'[' => JsonToken::open_square(at).into(),
            b']' => JsonToken::close_square(at).into(),
            b':' => JsonToken::colon(at).into(),
            b',' => JsonToken::comma(at).into(),
            // string and literal
            b'\'' => { let mut str_tokens = Vec::<StrType>::new(); loop {
                // parse all character wrapped inside single quote
                let Some(next_item) = self.next_item() else {
                    break JsonToken::error(constant::msg::MISSING_SINGLE_COLON, at, self.pos).into();
                };
                if next_item.eq(&constant::ascii::ESCAPE)  {
                    let Some(next_it) = self.next_item() else {
                        break JsonToken::error(constant::msg::MISSING_SINGLE_COLON, at, self.pos).into();
                    };

                    // handle '\xXX'
                    if next_it.eq(&b'x') {
                        match self.next_exact_until(2, |item| item.is_ascii_hexdigit()) {
                            Ok(_) => {
                                str_tokens.push(StrType::Ascii(Span::new(self.pos - 3, self.pos)));
                                continue;
                            },
                            Err(err) => break JsonToken::from(err).into(),
                        }
                    }

                    // handle '\uXXXX'
                    if next_it.eq(&b'u') {
                        match self.next_exact_until(4, |item| item.is_ascii_hexdigit()) {
                            Ok(_) => {
                                str_tokens.push(StrType::Unicode(Span::new(self.pos - 5, self.pos)));
                                continue;
                            },
                            Err(err) => break JsonToken::from(err).into(),
                        }
                    }

                    if matches!(next_it, b'\'' | b'\"' | b'\\' | b'b' | b'f' | b'n' | b'r' | b't' | b'v' | b'0') {
                        str_tokens.push(StrType::Special(Span::new(self.pos - 1, self.pos)));
                        continue;
                    }

                    if next_it.is_ascii_digit() {
                        return JsonToken::error(format!("{}: {}", constant::msg::INVALID_ESCAPE, char::from_u32(next_it as u32).unwrap()), at, self.pos).into();
                    }

                    str_tokens.push(StrType::Escape(Span::new(self.pos - 1, self.pos)));
                }

                if next_item.eq(&constant::ascii::SINGLE_QUOTE) {
                    break JsonToken::str(str_tokens, at, self.pos).into()
                }

                let Span { start, end, .. } = self.next_until(|it| matches!(it, b'\\' | b'\''));
                str_tokens.push(StrType::Str(Span::new(start, end)));
            }},
            b'"' => { let mut str_tokens = Vec::<StrType>::new();  loop {
                // parse all character wrapped inside double quote
                let Some(next_item) = self.next_item() else {
                    break JsonToken::error(constant::msg::MISSING_DOUBLE_COLON, at, self.pos).into();
                };
                if next_item.eq(&constant::ascii::ESCAPE)  {
                    let Some(next_it) = self.next_item() else {
                        break JsonToken::error(constant::msg::MISSING_DOUBLE_COLON, at, self.pos).into();
                    };

                    // handle '\xXX'
                    if next_it.eq(&b'x') {
                        match self.next_exact_until(2, |item| item.is_ascii_hexdigit()) {
                            Ok(_) => {
                                str_tokens.push(StrType::Ascii(Span::new(self.pos - 3, self.pos)));
                                continue;
                            },
                            Err(err) => break JsonToken::from(err).into(),
                        }
                    }

                    // handle '\uXXXX'
                    if next_it.eq(&b'u') {
                        match self.next_exact_until(4, |item| item.is_ascii_hexdigit()) {
                            Ok(_) => {
                                str_tokens.push(StrType::Unicode(Span::new(self.pos - 5, self.pos)));
                                continue;
                            },
                            Err(err) => break JsonToken::from(err).into(),
                        }
                    }

                    if matches!(next_it, b'\'' | b'\"' | b'\\' | b'b' | b'f' | b'n' | b'r' | b't' | b'v' | b'0') {
                        str_tokens.push(StrType::Special(Span::new(self.pos - 1, self.pos)));
                        continue;
                    }

                    if next_it.is_ascii_digit() {
                        return JsonToken::error(format!("{}: {}", constant::msg::INVALID_ESCAPE, char::from_u32(next_it as u32).unwrap()), at, self.pos).into();
                    }

                    str_tokens.push(StrType::Escape(Span::new(self.pos - 1, self.pos)));
                }

                if next_item.eq(&constant::ascii::DOUBLE_QUOTE) {
                    break JsonToken::str(str_tokens, at, self.pos).into()
                }

                let Span { start, end, .. } = self.next_until(|it| matches!(it, b'\\' | b'"'));
                str_tokens.push(StrType::Str(Span::new(start, end)));
            }},
            // negative number
            // b'-' => {
            //     let start: usize = self.pos;
            //     let Some(next_item) = self.next_item() else {
            //         return JsonToken::error("Invalid negative number, should following by at least a number", start, self.pos).into();
            //     };

            //     if next_item.eq(&b'0') {
            //         if matches!(next_item, b'x' | b'X') {
            //             let hex_span = self.next_until(|item| !item.is_ascii_hexdigit());
            //             return Some((JsonToken::number(NumType::Hex(hex_span), at, self.pos), Some(next_item)));
            //         }
            //         return JsonToken::error("Invalid positive hexadecimal", start, self.pos).into();
            //     }

            //     if next_item.is_ascii_digit() {

            //     }

            //     return JsonToken::error("Invalid positive hexadecimal", start, self.pos).into();
            // },
            b'-' | b'+' => {
                let Some(next_it) = self.peek_next_item() else {
                    return JsonToken::error("Invalid number, '+' and '-' must follow by at least a digit", at, self.pos).into();
                };

                if !next_it.is_ascii_digit() {
                    return JsonToken::error("Invalid number, '+' and '-' must follow by at least a digit", at, self.pos).into();
                }

                if next_item.eq(&b'+') {
                    JsonToken::plus(at, self.pos).into()
                } else {
                    JsonToken::minus(at, self.pos).into()
                }
            },
            // positive hexadecimal
            b'0' => {
                let Some(sign) = self.peek_prev_item() else {
                    return JsonToken::error("None sense when parsing number, must have at least one token behind", at, self.pos).into();
                };
                let Some(next_it) = self.next_item() else {
                    return JsonToken::number(Sign::detect(sign).to_integer(at, self.pos), at, self.pos).into()
                };
                if matches!(next_it, b'x' | b'X') {
                    let _ = self.next_until(|item| !item.is_ascii_hexdigit());
                    if (self.pos - at) < 4 {
                        JsonToken::error("Invalid hexadecimal, at least two hexdigit hehind 0x..", at, self.pos).into()
                    } else {
                        JsonToken::number(Sign::detect(sign).to_hexadecimal(at, self.pos), at, self.pos).into()
                    }
                } else if next_it.eq(&b'.') {
                    let frac_span = self.next_until(|item| !item.is_ascii_alphanumeric());
                    JsonToken::number(Sign::detect(sign).to_decimal(None, Some(frac_span)), at, self.pos).into()
                } else if next_it.is_ascii_digit() {
                    let _ = self.next_until(|item| !item.is_ascii_digit());
                    JsonToken::error("Invalid integer, not allow number that start with '0'", at, self.pos).into()
                } else {
                    self.step_back();
                    JsonToken::number(Sign::detect(sign).to_integer(at, self.pos), at, self.pos).into()
                }
            },
            b'.' => {
                let frac_span = self.next_until(|item| !item.is_ascii_alphanumeric());
                if frac_span.end != at {
                    JsonToken::number(Decimal::Positive(None, Some(frac_span)).into(), at, self.pos).into()
                } else {
                    JsonToken::error("Invalid decimal fracment", at, self.pos).into()
                }
            },
            // positive integer or decimal
            b'1'..=b'9' => {
                let Some(sign) = self.peek_prev_item() else {
                    return JsonToken::error("None sense when parsing number, must have at least one token behind", at, self.pos).into();
                };
                let int_span = self.next_until(|item| !item.is_ascii_alphanumeric());

                let Some(next_item) = self.next_item() else {
                    return JsonToken::number(Sign::detect(sign).to_integer(at, self.pos), at, self.pos).into();
                };

                // handle fractional
                if next_item.eq(&b'.') {
                    let dot_pos = self.pos;
                    let frac_span = self.next_until(|item| !item.is_ascii_alphanumeric());

                    if frac_span.end != dot_pos {
                        return JsonToken::number(Sign::detect(sign).to_decimal(Some(int_span), Some(frac_span)), at, self.pos).into();
                    } else {
                        return JsonToken::number(Sign::detect(sign).to_decimal(Some(int_span), None), at, self.pos).into();
                    }
                }

                self.step_back();

                JsonToken::number(Sign::detect(sign).to_integer(at, self.pos), at, self.pos).into()
            },
            // identity or keyword
            b'a'..=b'z' | b'A'..=b'Z' | b'_' => loop {
                let Some(next_item) = self.next_item() else {
                    break self.parse_keyword(at).into()
                };
                // accept ident has number in their name such as 'u8', 'u16'
                if !matches!(next_item, b'a'..=b'z' | b'A'..=b'Z' | b'0'..=b'9' | b'_') {
                    self.step_back();
                    break Some(self.parse_keyword(at))
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
        assert_eq!(Some(JsonToken::number(Integer::Positive(Span::new(6,7)).into(), 6, 7)), tokens.next());
        assert_eq!(Some(JsonToken::whitespace(7,9)), tokens.next());
        assert_eq!(Some(JsonToken::close_curly(9)) , tokens.next());
        assert_eq!(None                            , tokens.next());
    }

    #[test]
    pub fn parse_keyword() {
        let mut kws = Tokenizer::from("true false undefined");
        
        assert_eq!(Some(JsonToken::boolean(true, 0)) , kws.next());
        assert_eq!(Some(JsonToken::whitespace(4,5))  , kws.next());
        assert_eq!(Some(JsonToken::boolean(false, 5)), kws.next());
        assert_eq!(Some(JsonToken::whitespace(10,11)), kws.next());
        assert_eq!(Some(JsonToken::ident(11,20))     , kws.next());
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