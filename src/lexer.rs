use std::{marker::PhantomData, ptr};
use jsode_macro::reflection;

use crate::{constant, core::{Decimal, JsonToken, NumType, Sign, Span, StrType}, error::JsonError};

#[derive(PartialEq, Debug)]
pub struct Tokenizer<'a> {
    ptr: *const u8,
    pub(crate) pos: usize,
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
            // comment
            b'/' => {
                let Some(next_item) = self.next_item() else {
                    return JsonToken::error("Invalid comment, must follow by another '/' (single-line comment) or '*' (multi-line comment)", at, self.pos).into();
                };
                // single-line comment
                if next_item.eq(&b'/') {
                    let _ = self.consume_until(|it| it.eq(&b'\n'));
                    return JsonToken::comment(at, self.pos).into();
                }
                // multi-line comment
                if next_item.eq(&b'*') {
                    let _ = self.consume_pair_until(|f,s| f.eq(&b'*') && s.eq(&b'/'));
                    return JsonToken::comment(at, self.pos).into();
                }

                JsonToken::error("Invalid comment, must follow by another '/' (single-line comment) or '*' (multi-line comment)", at, self.pos).into()
            },
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

                let Span { start, end, .. } = self.move_backward_then_consume_until(1, |it| matches!(it, b'\\' | b'\''));
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

                let Span { start, end, .. } = self.move_backward_then_consume_until(1, |it| matches!(it, b'\\' | b'"'));
                str_tokens.push(StrType::Str(Span::new(start, end)));
            }},
            // negative number
            b'-' | b'+' => {
                let Some(next_it) = self.peek_next_item() else {
                    return JsonToken::error("Invalid number, '+' and '-' must follow by at least a digit", at, self.pos).into();
                };

                if !next_it.is_ascii_digit() {
                    return JsonToken::error("Invalid number, '+' and '-' must follow by at least a digit", at, self.pos).into();
                }

                JsonToken::whitespace(at, self.pos).into()
            },
            b'.' => {
                // consume until reaching a non-digit character
                let frac_span = self.consume_until(|item| !item.is_ascii_digit());
                if frac_span.end != at {
                    JsonToken::number(Decimal::Positive(None, Some(frac_span), None).into(), at, self.pos).into()
                } else {
                    JsonToken::error("Invalid decimal fracment", at, self.pos).into()
                }
            },
            // positive integer or decimal
            b'0'..=b'9' => {
                let Some(sign) = self.peek_prev_item() else {
                    return JsonToken::error("None sense when parsing number, must have at least one token behind", at, self.pos).into();
                };
                let inverse = Sign::detect(sign);
                let start_at = inverse.compute_start_pos(at);

                // handle hexadecimal
                if next_item.eq(&b'0') && self.peek_next_item().is_some_and(|it| matches!(it, b'x' | b'X')) {
                    let _ = self.move_forward_then_consume_until(1, |item| !item.is_ascii_hexdigit());
                    if (self.pos - at) < 4 {
                        return JsonToken::error("Invalid hexadecimal, at least two hexdigit hehind 0x..", start_at, self.pos).into()
                    } else {
                        return JsonToken::number(inverse.to_hexadecimal(Span::new(at, at + 2), Span::new(at + 2, self.pos)), start_at, self.pos).into()
                    }
                }

                let int_span = self.move_backward_then_consume_until(1, |item| !item.is_ascii_digit());
                let Some(next_item) = self.next_item() else {
                    return JsonToken::number(inverse.to_integer(at, self.pos, None), start_at, self.pos).into();
                };

                // handle fractional
                if next_item.eq(&b'.') {
                    // consume until reaching a non-digit character
                    let frac_span = self.consume_until(|item| !item.is_ascii_digit());

                    if frac_span.start != frac_span.end {
                        if self.peek_next_item().is_some_and(|it| matches!(it, b'e' | b'E')) {
                            let expo_span = self.move_forward_then_consume_until(1, |item| !matches!(item, b'+' | b'-' | b'0'..=b'9'));
                            if expo_span.start != expo_span.end {
                                return JsonToken::number(inverse.to_decimal(Some(int_span), Some(frac_span), Some(expo_span)), start_at, self.pos).into();
                            }
                        }
                        return JsonToken::number(inverse.to_decimal(Some(int_span), Some(frac_span), None), start_at, self.pos).into();
                    } else {
                        return JsonToken::number(inverse.to_decimal(Some(int_span), None, None), start_at, self.pos).into();
                    }
                }
                // handle exponential
                if matches!(next_item, b'e' | b'E') {
                    // if exponent has a sign ('+' | '-') and follow by a digit. Move cursor to the next position and return 1.
                    // if exponent has only digits, return 0 instead.
                    // otherwise not a valid exponent
                    let expo_sign: usize = match self.peek_next_item() {
                        Some(b'+' | b'-') if self.peek_next_nth_item(2).is_some_and(|it| it.is_ascii_digit()) => {
                            self.step_front();
                            1
                        },
                        Some(n_n_it) if n_n_it.is_ascii_digit() => 0,
                        _ => return JsonToken::error("Invalid exponential number, exponent must contain at least one number", start_at, self.pos).into(),
                    };
                    let Span { start: expo_start, end: expo_end, .. } = self.consume_until(|item| !item.is_ascii_digit());

                    return JsonToken::number(
                        // minus for the `expo_sign` which was computed in the previous step to include the sign ('+' | '-') in the final span
                        // if it exist
                        inverse.to_integer(int_span.start, int_span.end, Some(Span::new(expo_start - expo_sign, expo_end))),
                        start_at,
                        self.pos
                    ).into();
                }

                self.step_back();
                JsonToken::number(inverse.to_integer(at, self.pos, None), start_at, self.pos).into()
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

impl <'a> Tokenizer<'a> {
    #[inline]
    pub const fn take_raw(&self, span: Span) -> &[u8] {
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

    // SAFETY: as long as `self.pos` being control and not exceeding `self.size`
    // 0 <= self.pos <= self.size
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

    // SAFETY: 0 <= self.pos <= self.size
    #[inline]
    const fn peek_prev_item(&self) -> Option<u8> {
        if self.pos > 0 {
            // subtract is sounded, program will panic if (self.pos - 2) < 0
            let prev_item = unsafe { ptr::read(self.ptr.add(self.pos.saturating_sub(2))) };
            Some(prev_item)
        } else {
            None
        }
    }

    #[inline]
    const fn peek_next_item(&self) -> Option<u8> {
        if self.pos >= self.size {
            None
        } else {
            let next_item = unsafe { ptr::read(self.ptr.add(self.pos)) };
            Some(next_item)
        }
    }

    // SAFETY: 0 <= self.pos <= self.size
    #[inline]
    const fn peek_next_nth_item(&self, n: usize) -> Option<u8> {
        // because `self.pos` is already present for next item position, so we must minus the `n` with `1` to make it correct.
        // if `n` equal 0, return item at `self.pos`
        let nth_item_pos = n - 1;
        if self.pos + nth_item_pos >= self.size {
            None
        } else {
            let next_item = unsafe { ptr::read(self.ptr.add(self.pos + nth_item_pos)) };
            Some(next_item)
        }
    }

    #[inline]
    #[reflection]
    fn next_exact_until(&mut self, size: usize, predicate: impl Fn(u8) -> bool) -> Result<(), JsonError> {
        for n in 0..size {
            let Some(next_item) = self.next_item() else {
                return Err(JsonError::custom(
                    format!("[{__fn_ident}] soon EOF, expect {n} token more"),
                    Span::new(self.pos - (self.pos - n + 1), self.pos)));
            };

            if !predicate(next_item) {
                return Err(JsonError::custom(
                    format!("[{__fn_ident}] cannot satisfy condition"),
                    Span::new(self.pos - (self.pos - n + 1), self.pos)))
            }
        }
        Ok(())
    }

    // iterate over `src` until reaching the **UNEXPECTED** token
    // CAUTION: this method modify `self.pos` to avoid consume the **UNEXPECTED** token
    #[inline]
    fn consume_until(&mut self, predicate: impl Fn(u8) -> bool) -> Span {
        let start: usize = self.pos;
        while let Some(next_item) = self.next_item() {
            if predicate(next_item) {
                // the cursor is pointing into `next_item` (UNEXPECTED token),
                // move the cursor backward to unconsume it.
                self.step_back();
                break;
            }
        }
        Span::new(start, self.pos)
    }

    // iterate over `src` until reaching **EXPECTED** token
    // CAUTION: this method modify `self.pos` to consume all **EXPECTED** tokens
    #[inline]
    fn consume_pair_until(&mut self, predicate: impl Fn(u8,u8) -> bool) -> Span {
        let start: usize = self.pos;
        loop {
            let (Some(first_item), Some(second_item)) = (self.next_item(), self.peek_next_item()) else {
                break;
            };

            if predicate(first_item, second_item) {
                // the cursor is pointing into `second_item`,
                // move the cursor forward to consume it.
                self.step_front();
                break;
            }
        }
        Span::new(start, self.pos)
    }

    #[inline]
    fn move_forward_then_consume_until(&mut self, n: usize, predicate: impl Fn(u8) -> bool) -> Span {
        self.pos = (self.pos + n).min(self.size);
        self.consume_until(predicate)
    }

    #[inline]
    fn move_backward_then_consume_until(&mut self, n: usize, predicate: impl Fn(u8) -> bool) -> Span {
        // sounded
        self.pos -= n;
        self.consume_until(predicate)
    }

    #[inline(always)]
    pub(crate) fn step_back(&mut self) -> usize {
        // sounded
        self.pos -= 1;
        self.pos
    }

    #[inline(always)]
    pub(crate) fn step_back_nth(&mut self, step: usize) -> usize {
        // sounded
        self.pos -= step;
        self.pos
    }
    
    #[inline(always)]
    fn step_front(&mut self) -> usize {
        if self.pos >= self.size { return self.pos; }
        self.pos += 1;
        self.pos
    }

    fn parse_keyword(&self, start: usize) -> JsonToken {
        let gap = self.pos - start;
        let buf = unsafe { std::slice::from_raw_parts(self.ptr.add(start), gap) };
        std::str::from_utf8(buf).map(|res| match res {
            "true"      => JsonToken::boolean(true, start),
            "false"     => JsonToken::boolean(false, start),
            "Infinity"  => JsonToken::number(NumType::Infinity(Span::new(start, self.pos)), start, self.pos),
            "NaN"       => JsonToken::number(NumType::NaN(Span::new(start, self.pos)), start, self.pos),
            _           => JsonToken::ident(start, self.pos),
        }).unwrap_or_else(|err| JsonToken::error(err.to_string(), start, gap))
        
    }
}

#[cfg(test)]
mod tests {
    use crate::core::Integer;
    use super::*;

    #[test]
    pub fn accept_slice_str() {
        let mut tokens = Tokenizer::from("{ one:1  }");

        assert_eq!(Some(JsonToken::open_curly(0))  , tokens.next());
        assert_eq!(Some(JsonToken::whitespace(1,2)), tokens.next());
        assert_eq!(Some(JsonToken::ident(2, 5))    , tokens.next());
        assert_eq!(Some(JsonToken::colon(5))       , tokens.next());
        assert_eq!(Some(JsonToken::number(Integer::Positive(Span::new(6,7), None).into(), 6, 7)), tokens.next());
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