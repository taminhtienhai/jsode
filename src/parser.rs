use crate::{core::{JsonArray, JsonInt, JsonObject, JsonProp, JsonStr, JsonToken, JsonType, JsonValue, Punct, Span}, error::JsonError, lexer::Tokenizer};

pub struct JsonParser<Iter: Iterator<Item = JsonToken>> {
    iter: Iter,
    state: JsonState,
    // prev_token: Option<JsonToken>,
    // cur_token: Option<JsonToken>,
}

impl <'tk> From<Tokenizer<'tk>> for JsonParser<Tokenizer<'tk>> {
    fn from(value: Tokenizer<'tk>) -> Self {
        Self {
            iter: value,
            state: JsonState::Idle,
            // prev_token: None,
            // cur_token: None,
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

impl <'tk> JsonParser<Tokenizer<'tk>> {
    #[inline]
    pub fn new(src: &'tk str) -> Self {
        Self {
            iter: Tokenizer::from(src),
            state: JsonState::Idle,
        }
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

    pub fn change_state_to(&mut self, new_state: JsonState) {
        self.state = new_state;
    }
}

impl <'tk> JsonParser<Tokenizer<'tk>> {
    // call this when reaching '{'
    fn parse_obj(&mut self) -> Result<JsonValue, JsonError> {
        let mut props = Vec::<JsonProp<JsonStr>>::new();
        loop {
            let prop = self.parse_prop()?;
            if let Some(property) = prop {
                props.push(property);
            } else {
                return Ok(JsonValue::Object(JsonObject::new(props, Span::default())))
            }
        }
    }

    // being call when reaching '['
    fn parse_array(&mut self) -> Result<JsonValue, JsonError> {
        let mut items = Vec::<JsonProp<JsonInt>>::new();
        let mut pos = 0;
        loop {
            let item = self.parse_arr_item(pos)?;
            if let Some(it) = item {
                pos += 1;
                items.push(it);
            } else {
                return Ok(JsonValue::Array(JsonArray::new(items, Span::default())))
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
            None =>  return Err(JsonError::custom("still incomplete parsing this array", Span::default()))
        };
        Ok(Some(JsonProp::new(JsonInt(pos), item_value)))
    }

    fn parse_prop(&mut self) -> Result<Option<JsonProp<JsonStr>>, JsonError> {
        let key = self.next_token_skip(|tk| matches!(tk, JsonToken::Punct(Punct::Comma | Punct::WhiteSpace, _)));
        let colon = self.next_token();
        let value = self.next_token();

        let (key_span, _) = match (key, colon) {
            (
                Some(JsonToken::Data(JsonType::Ident | JsonType::Str, kspan)),
                Some(JsonToken::Punct(Punct::Colon, cspan))
            ) => (kspan, cspan),
            (
                Some(JsonToken::Punct(Punct::CloseCurly, _)),
                _,
            ) => return Ok(None),
            (
                Some(JsonToken::Data(_, key_span)),
                Some(JsonToken::Punct(_, colon_span))
            )                                     => return Err(JsonError::empty_json(key_span.extend(colon_span))),
            (Some(JsonToken::Error(err, span)),_) => return Err(JsonError::custom(err, span)),
            (_,Some(JsonToken::Error(err, span))) => return Err(JsonError::custom(err, span)),
            (_,_)                                 => return Err(JsonError::empty_json(Span::default())),
        };

        let v = match value {
            Some(JsonToken::Punct(Punct::OpenCurly, _)) => self.parse_obj(),
            Some(JsonToken::Punct(Punct::OpenSquare, _)) => self.parse_array(),
            Some(JsonToken::Data(data, data_span)) => Ok(JsonValue::Data(data, data_span)),
            Some(JsonToken::Error(err, span)) => return Err(JsonError::custom(err, span)),
            _ => return Err(JsonError::empty_json(Span::default())),
        }?;

        Ok(Some(JsonProp::new(JsonStr(key_span), v)))
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_json_object() {
        let json = "{'a':1,'b':2}";
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