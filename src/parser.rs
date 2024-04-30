use std::collections::HashMap;

use crate::{common, constant::msg, core::{JsonArray, JsonInt, JsonObject, JsonOutput, JsonProp, JsonStr, JsonToken, JsonType, JsonValue, Punct, Span}, error::JsonError, lexer::Tokenizer};

#[derive(PartialEq, Debug)]
pub struct JsonParser<Iter: Iterator<Item = JsonToken>> {
    iter: Iter,
}

impl <'tk> From<Tokenizer<'tk>> for JsonParser<Tokenizer<'tk>> {
    fn from(value: Tokenizer<'tk>) -> Self {
        Self {
            iter: value,
        }
    }
}

impl <'par> JsonParser<Tokenizer<'par>> {
    #[inline]
    pub fn new(src: &'par str) -> Self {
        Self {
            iter: Tokenizer::from(src),
        }
    }
}

impl <'tk> JsonParser<Tokenizer<'tk>> {
    pub fn parse(&'tk mut self) -> core::result::Result<JsonOutput, JsonError> {
        while let Some(next_token) = self.iter.next() {
            match next_token {
                JsonToken::Punct(Punct::OpenCurly, _) => return self.start_parse_obj(),
                JsonToken::Punct(Punct::OpenSquare, _) => return self.start_parse_array(),
                _ => return Err(JsonError::missing_double_colon(next_token.get_span())),
            };
        };
        Err(JsonError::missing_double_colon(Span::default()))
    }
}


impl <'tk> JsonParser<Tokenizer<'tk>> {

    #[inline]
    pub fn take_raw(&self, span: Span) -> &[u8] {
        self.iter.take_raw(span)
    }

    #[inline]
    pub fn take_slice(&self, span: Span) -> Result<&str, JsonError> {
        self.iter.take_slice(span)
    }

    // fetching next token, skip all 'whitespace'
    #[inline]
    pub fn next_token(&mut self) -> Option<JsonToken> {
        loop {
            let token = self.iter.next()?;
            match token {
                JsonToken::Punct(Punct::WhiteSpace, _) => continue,
                _ => break Some(token),
            };
        }
    }

    #[inline]
    pub fn next_token_skip(&mut self, predicate: impl Fn(&JsonToken) -> bool) -> Option<JsonToken> {
        loop {
            let token = self.iter.next()?;
            if (predicate)(&token) {
                continue;
            }
            return Some(token);
        }
    }
}

impl <'tk> JsonParser<Tokenizer<'tk>> {
    // call this when reaching '{'
    fn start_parse_obj(&'tk mut self) -> Result<JsonOutput<'tk>, JsonError> {
        let start = self.iter.cur_item_pos();
        let mut props = HashMap::<u64, JsonProp<JsonStr>>::new();
        loop {
            if let Some(JsonProp { key, value }) = self.parse_prop()? {
                let key_slice = self.take_slice(key.0.clone())?;
                let hashed_key = common::hash_str(key_slice);
                if props.contains_key(&hashed_key) {
                    return Err(JsonError::custom(format!("{} `{}`", msg::DUPLICATE_KEY, key_slice), key.0))
                }
                props.insert(hashed_key, JsonProp::new(JsonStr(key.0.clone()), value));
            } else {
                let ast = JsonValue::Object(JsonObject::new(props, Span::new(start, self.iter.next_item_pos())));
                return Ok(JsonOutput::new(self, ast))
            }
        }
    }

    fn start_parse_array(&'tk mut self) -> Result<JsonOutput<'tk>, JsonError> {
        let start = self.iter.cur_item_pos();
        let mut items = Vec::<JsonProp<JsonInt>>::new();
        let mut pos = 0;
        loop {
            let item = self.parse_arr_item(pos)?;
            if let Some(it) = item {
                pos += 1;
                items.push(it);
            } else {
                let ast = JsonValue::Array(JsonArray::new(items, Span::new(start, self.iter.next_item_pos())));
                return Ok(JsonOutput::new(self, ast));
            }
        }
    }

    // call this when reaching '{'
    fn parse_obj(&mut self) -> Result<JsonValue, JsonError> {
        let start = self.iter.cur_item_pos();
        let mut props = HashMap::<u64, JsonProp<JsonStr>>::new();
        loop {
            if let Some(JsonProp { key, value }) = self.parse_prop()? {
                let key_slice = self.take_slice(key.0.clone())?;
                let hashed_key = common::hash_str(key_slice);
                if props.contains_key(&hashed_key) {
                    return Err(JsonError::custom(format!("{} `{}`", msg::DUPLICATE_KEY, key_slice), key.0))
                }
                props.insert(hashed_key, JsonProp::new(JsonStr(key.0.clone()), value));
            } else {
                return Ok(JsonValue::Object(JsonObject::new(props, Span::new(start, self.iter.next_item_pos()))))
            }
        }
    }

    // being call when reaching '['
    fn parse_array(&mut self) -> Result<JsonValue, JsonError> {
        let start = self.iter.cur_item_pos();
        let mut items = Vec::<JsonProp<JsonInt>>::new();
        let mut pos = 0;
        loop {
            let item = self.parse_arr_item(pos)?;
            if let Some(it) = item {
                pos += 1;
                items.push(it);
            } else {
                return Ok(JsonValue::Array(JsonArray::new(items, Span::new(start, self.iter.next_item_pos()))));
            }
        }
    }

    fn parse_arr_item(&mut self, pos: usize) -> Result<Option<JsonProp<JsonInt>>, JsonError> {
        let next_item = self.next_token_skip(|tk| matches!(tk, JsonToken::Punct(Punct::Comma | Punct::WhiteSpace | Punct::Plus | Punct::Minus, _)));
        let item_value = match next_item {
            Some(JsonToken::Data(data, span)) => JsonValue::Data(data, span),
            Some(JsonToken::Punct(Punct::OpenCurly, _)) => self.parse_obj()?,
            Some(JsonToken::Punct(Punct::OpenSquare, _)) => self.parse_array()?,
            Some(JsonToken::Punct(Punct::CloseSquare, _)) => return Ok(None),
            Some(JsonToken::Punct(_, span)) => return Err(JsonError::invalid_array(span)),
            Some(JsonToken::Error(err, span)) => return Err(JsonError::custom(err, span)),
            None =>  return Err(JsonError::custom("[parse_arr_item] reaching None when parsing", Span::default()))
        };
        Ok(Some(JsonProp::new(JsonInt(pos), item_value)))
    }

    fn parse_prop(&mut self) -> Result<Option<JsonProp<JsonStr>>, JsonError> {
        let key_span = match self.next_token_skip(|tk| matches!(tk, JsonToken::Punct(Punct::Comma | Punct::WhiteSpace, _))) {
            Some(JsonToken::Data(JsonType::Str(_), span)) => span.collapse(1),
            Some(JsonToken::Data(JsonType::Ident, span)) => span,
            Some(JsonToken::Punct(Punct::CloseCurly, _)) => return Ok(None),
            Some(JsonToken::Error(err, span)) => return Err(JsonError::custom(err, span)),
            Some(tk) => return Err(JsonError::custom("[parse_prop] unexpected token when parsing key", tk.get_span())),
            None => return Err(JsonError::custom("[parse_prop] `key` should not be None", Span::default())),
        };

        let _colon = match self.next_token() {
            Some(JsonToken::Punct(Punct::Colon, cspan)) => cspan,
            Some(JsonToken::Error(err, span)) => return Err(JsonError::custom(err, span)),
            Some(tk) => return Err(JsonError::custom("[parse_prop] unexpected token when parsing `colon`", tk.get_span())),
            None => return Err(JsonError::custom("[parse_prop] `colon` should not be None", Span::default())),
        };

        let value = match self.next_token_skip(|tk| matches!(tk, JsonToken::Punct(Punct::WhiteSpace | Punct::Plus | Punct::Minus, _))) {
            Some(JsonToken::Punct(Punct::OpenCurly, _)) => self.parse_obj()?,
            Some(JsonToken::Punct(Punct::OpenSquare, _)) => self.parse_array()?,
            Some(JsonToken::Data(JsonType::Str(str_value), data_span)) => JsonValue::Data(JsonType::Str(str_value), data_span),
            Some(JsonToken::Data(data, data_span)) => JsonValue::Data(data, data_span),
            Some(JsonToken::Error(err, span)) => return Err(JsonError::custom(err, span)),
            Some(tk) =>  return Err(JsonError::custom("[parse_prop] not able to parse this token", tk.get_span())),
            None => return Err(JsonError::custom("[parse_prop] parsing prop value but reaching None", Span::default())),
        };

        Ok(Some(JsonProp::new(JsonStr(key_span), value)))
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_json_object() {
        let json = "{'a':1 ,'b':2}";
        let mut obj = JsonParser::new(json);

        let _ = obj.parse().inspect_err(|e| eprintln!("{}", e));
    }

    #[test]
    fn parse_json_array() {
        let json = "[1,2, { a: 1, b: [1,2,3] }]";
        let mut arr = JsonParser::new(json);

        let _ = arr.parse().inspect_err(|e| eprintln!("{}", e));
    }

    #[test]
    fn parse_json_object2() {
        let json = "{'a':1 ,'b':2}";
        let mut obj = JsonParser::new(json);

        let _ = obj.parse().inspect_err(|e| eprintln!("{}", e));
    }

    #[test]
    fn parse_json_array2() {
        let json = "[1,2, { a: 1, b: [1,2,3] }]";
        let mut arr = JsonParser::new(json);

        let _ = arr.parse().inspect_err(|e| eprintln!("{}", e));
    }

    // UnHappy Cases

    #[test]
    fn json_key_not_valid() {
        let mut arr = JsonParser::new("{ $dolar: 1 }");

        let err = arr.parse().unwrap_err();

        println!("{err}");
        println!("{err:?}");
    }

    #[test]
    fn duplicate_key() {
        let mut obj = JsonParser::new(
        r#"{
            a: 1,
            a: 2
        }"#
        );
        let err = obj.parse().unwrap_err();

        println!("{err}");
        println!("{err:?}");
    }
}