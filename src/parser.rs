use crate::{core::{JsonArray, JsonInt, JsonObject, JsonProp, JsonStr, JsonToken, JsonType, JsonValue, Punct, Span,}, error::JsonError, indexer::JsonIndexer, lexer::Tokenizer};

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

pub enum JsonState {
    Idle,
    Object,
    Array,
    ObjProp,
    ArrItem,
    EOS, // End Of Stream
    Error,
}

impl <'par> JsonParser<Tokenizer<'par>> {
    #[inline]
    pub fn new(src: &'par str) -> Self {
        Self {
            iter: Tokenizer::from(src),
        }
    }

    pub fn indexer_from(&'par self, ast: &'par JsonValue) -> JsonIndexer<'par> {
        JsonIndexer::new(self, ast)
    }
}

impl <'tk> JsonParser<Tokenizer<'tk>> {
    pub fn parse(&mut self) -> core::result::Result<JsonValue, JsonError> {
        while let Some(next_token) = self.iter.next() {
            match next_token {
                JsonToken::Punct(Punct::OpenCurly, _) => return self.parse_obj(),
                JsonToken::Punct(Punct::OpenSquare, _) => return self.parse_array(),
                _ => return Err(JsonError::missing_double_colon(Span::default())),
            };
        };
        Err(JsonError::missing_double_colon(Span::default()))
    }
}


impl <'tk> JsonParser<Tokenizer<'tk>> {
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
    fn parse_obj(&mut self) -> Result<JsonValue, JsonError> {
        let start = self.iter.prev_pos();
        let mut props = Vec::<JsonProp<JsonStr>>::new();
        loop {
            let prop = self.parse_prop()?;
            if let Some(property) = prop {
                props.push(property);
            } else {
                return Ok(JsonValue::Object(JsonObject::new(props, Span::new(start, self.iter.cur_pos()))))
            }
        }
    }

    // being call when reaching '['
    fn parse_array(&mut self) -> Result<JsonValue, JsonError> {
        let start = self.iter.prev_pos();
        let mut items = Vec::<JsonProp<JsonInt>>::new();
        let mut pos = 0;
        loop {
            let item = self.parse_arr_item(pos)?;
            if let Some(it) = item {
                pos += 1;
                items.push(it);
            } else {
                return Ok(JsonValue::Array(JsonArray::new(items, Span::new(start, self.iter.cur_pos()))));
            }
        }
    }

    fn parse_arr_item(&mut self, pos: usize) -> Result<Option<JsonProp<JsonInt>>, JsonError> {
        let next_item = self.next_token_skip(|tk| matches!(tk, JsonToken::Punct(Punct::Comma | Punct::WhiteSpace, _)));
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
            Some(JsonToken::Data(JsonType::Str, span)) => span.collapse(1),
            Some(JsonToken::Data(JsonType::Ident, span)) => span,
            Some(JsonToken::Punct(Punct::CloseCurly, _)) => return Ok(None),
            Some(JsonToken::Error(err, span)) => return Err(JsonError::custom(err, span)),
            None => return Err(JsonError::custom("[parse_prop] `key` should not be None", Span::default())),
            _ => return Err(JsonError::custom("[parse_prop] unexpected token when parsing key", Span::default())),
        };

        let _colon = match self.next_token() {
            Some(JsonToken::Punct(Punct::Colon, cspan)) => cspan,
            Some(JsonToken::Error(err, span)) => return Err(JsonError::custom(err, span)),
            None => return Err(JsonError::custom("[parse_prop] `colon` should not be None", Span::default())),
            _ => return Err(JsonError::custom("[parse_prop] unexpected token when parsing `colon`", Span::default()))
        };

        let value = match self.next_token() {
            Some(JsonToken::Punct(Punct::OpenCurly, _)) => self.parse_obj()?,
            Some(JsonToken::Punct(Punct::OpenSquare, _)) => self.parse_array()?,
            Some(JsonToken::Data(data, data_span)) => JsonValue::Data(data, data_span),
            Some(JsonToken::Error(err, span)) => return Err(JsonError::custom(err, span)),
            Some(_) => return Err(JsonError::custom("[parse_prop] not able to parse this token", Span::default())), 
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
}