use std::collections::{HashMap, VecDeque};

use jsode_macro::reflection;

use crate::{
    common, constant::msg, core::{
        JsonBlock, JsonOutput, JsonToken,
        JsonType, JsonValue, Punct, Span,
    }, error::JsonError, lexer::Tokenizer
};

#[derive(PartialEq, Debug)]
pub struct JsonParser<'tk> {
    iter: Tokenizer<'tk>,
}

impl<'tk> JsonParser<'tk> {
    #[inline]
    pub fn new(src: &'tk str) -> Self {
        Self {
            iter: Tokenizer::from(src),
        }
    }
}

impl<'tk> JsonParser<'tk> {
    pub fn parse(&'_ mut self) -> crate::Result<JsonOutput<'_>> {
        let mut cursor = JsonCursor::init(self)?;

        let init_block = match cursor.roots.back() {
            Some(State::Object(_, _)) => JsonBlock::new(0, JsonValue::Object(HashMap::with_capacity(10), Span::default())),
            Some(State::Array(_, _)) => JsonBlock::new(0, JsonValue::Array(Vec::with_capacity(10), Span::default())),
            Some(State::Value(JsonType::Ident, span)) => return Err(JsonError::custom("Invalid JSON", span.clone())),
            Some(State::Value(_, value_span)) => {
                if self.next_token().is_some() {
                    return Err(JsonError::custom("Invalid JSON", value_span.clone()));
                }
                let State::Value(ty, span) = cursor.roots.pop_back().unwrap() else {
                    return Err(JsonError::custom("Invalid JSON", Span::default())); 
                };

                return Ok(JsonOutput::new(self, Vec::<JsonBlock>::from_iter([
                    JsonBlock::new(0, JsonValue::Value(ty, span))
                ])));
            },
            None => return Err(JsonError::custom("Invalid JSON", Span::default())),
        };
        cursor.level += 1;

        let mut ast = Vec::<JsonBlock>::from_iter([init_block]);

        while let Some(state) = cursor.roots.back() {
            let block = match state {
                State::Object(_,_) => cursor.parse_object_prop(self, ast.as_mut()),
                State::Array(_,_) => cursor.parse_array_item(self, ast.as_mut()),
                State::Value(_,_) => cursor.parse_value(self),
            }?;

            let next_token = match (!cursor.roots.is_empty(), self.next_token()) {
                (true, token @ Some(_)) => token,
                (false, None) => None,
                _ => return Err(JsonError::custom("Invalid JSON", Span::default())),
            };

            let Some(block_value) = block else { match &next_token {
                Some(JsonToken::Punct(Punct::Comma, _)) => 0,
                None => 0,
                Some(JsonToken::Punct(Punct::CloseCurly | Punct::CloseSquare, span)) => self.iter.step_back_nth(span.gap()),
                Some(other) => return Err(JsonError::custom("expect comma or close-square, found other", other.get_span())),
            }; continue; };

            match (&block_value.value, next_token) {
                (JsonValue::Prop(_,_,_) | JsonValue::Value(_,_), Some(JsonToken::Punct(Punct::Comma, _))) => 0,
                (JsonValue::Prop(_,_,_) | JsonValue::Value(_,_), Some(JsonToken::Punct(Punct::CloseCurly | Punct::CloseSquare, span))) => self.iter.step_back_nth(span.gap()),
                (JsonValue::Prop(_,_,_) | JsonValue::Value(_,_), Some(other)) => return Err(JsonError::custom("expect comma or close-square, found other", other.get_span())),
                (JsonValue::Prop(_,_,_) | JsonValue::Value(_,_), None) => return Err(JsonError::custom("parsing prop value but reaching None", Span::default())),
                (_, Some(other)) => self.iter.step_back_nth(other.get_span().gap()),
                (_, None) => return Err(JsonError::custom("expect comma or close-square, found other", Span::default())),
            };

            ast.push(block_value);
        }

        Ok(JsonOutput::new(self, ast))
    }
}

impl<'tk> JsonParser<'tk> {
    #[inline]
    pub const fn take_raw(&self, span: Span) -> &[u8] {
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

// state represent for the parent's type
#[derive(Debug)]
pub(crate) enum State {
    // the `usize` is the position in ast
    // the `HashMap` is indexes of their children
    // everytime a new property parsed successfully, `HashMap` will inserted a new key-value
    Object(usize, HashMap<usize, usize>),
    // the first `usize` is the position in ast,
    // the second `Vec<usize>` are position of each item
    // everytime a new item parsed successfully, second `usize` will increased by one
    Array(usize, Vec<usize>),
    // Prop,
    Value(JsonType, Span),
    // EOF,
}

#[derive(Debug)]
pub(crate) struct JsonCursor {
    level: usize,
    roots: VecDeque<State>,
}

impl JsonCursor {
    pub fn new(state: State) -> Self {
        Self {
            level: 0,
            roots: VecDeque::from_iter([state]),
        }
    }

    /// going up a level
    /// pop back latest state out of stack
    fn pop_state(&mut self) -> Option<State> {
        self.level = self.level.saturating_sub(1);
        self.roots.pop_back()
    }

    /// add new prop's index to PARENT
    /// note: PARENT should be an object
    #[inline]
    #[reflection]
    fn update_prop_index(&mut self, key: Span, parser: &JsonParser<'_>, block_pos: usize) -> crate::Result<()> {
        let Some(State::Object(anchor, ref mut prop_indexes)) = self.roots.back_mut() else {
            return Err(JsonError::custom(format!("[{__fn_ident}] {}", msg::SOON_EOS), Span::default()));
        };
        // insert new item to object indexes
        let key_slice = parser.take_slice(key)?;
        let key_hashed = common::hash_str(key_slice) as usize;
        // we should use relative instead absolute position here
        // because lately when we index value, the origin size of ast is hard to trace
        prop_indexes.insert(key_hashed, block_pos - *anchor);

        Ok(())
    }

    /// increase PARENT length by 1
    /// note: PARENT should be an array
    #[inline]
    #[reflection]
    fn update_array_length(&mut self, pos: usize) -> crate::Result<()> {
        let Some(State::Array(anchor, ref mut item_indexes)) = self.roots.back_mut() else {
            return Err(JsonError::custom(format!("[{__fn_ident}] {}", msg::SOON_EOS), Span::default()));
        };
        // push array's item related position
        item_indexes.push(pos - *anchor);

        Ok(())
    }

    #[inline]
    fn create_object_block(&mut self, position: usize, span: Span) -> JsonBlock {
        let block = JsonBlock {
            level: self.level,
            value: JsonValue::Object(HashMap::with_capacity(10), span),
        };
        self.level += 1;
        self.roots.push_back(State::Object(position, HashMap::with_capacity(10)));
        block
    }

    #[inline]
    fn create_array_block(&mut self, position: usize, span: Span) -> JsonBlock {
        let block = JsonBlock {
            level: self.level,
            value: JsonValue::Array(Vec::with_capacity(10), span),
        };
        self.level += 1;
        self.roots.push_back(State::Array(position, Vec::with_capacity(10)));
        block
    }

    #[reflection]
    fn create_prop_block(&mut self, key: Span, value: JsonType, value_span: Span, parser: &JsonParser<'_>, block_pos: usize) -> crate::Result<JsonBlock> {
        let Some(State::Object(anchor, ref mut prop_indexes)) = self.roots.back_mut() else {
            return Err(JsonError::custom(format!("[{__fn_ident}] {}", msg::SOON_EOS), Span::default()));
        };
        // insert new item to object indexes
        let key_slice = parser.take_slice(key.clone())?;
        let key_hashed = common::hash_str(key_slice) as usize;

        prop_indexes.insert(key_hashed, block_pos - *anchor);

        Ok(JsonBlock {
            level: self.level,
            value: JsonValue::Prop(value, value_span.clone(), key.extend(value_span)),
        })
    }

    #[reflection]
    fn create_item_block(&mut self, pos: usize, value: JsonType, value_span: Span) -> crate::Result<JsonBlock> {
        let Some(State::Array(anchor, ref mut item_indexes)) = self.roots.back_mut() else {
            return Err(JsonError::custom(format!("[{__fn_ident}] {}", msg::SOON_EOS), Span::default()));
        };
        // increase length of parent array by `1`
        item_indexes.push(pos - *anchor);

        Ok(JsonBlock {
            level: self.level,
            value: JsonValue::Value(value, value_span),
        })
    }

    #[inline]
    const fn create_value_block(&self, value: JsonType, value_span: Span) -> JsonBlock {
        JsonBlock {
            level: self.level,
            value: JsonValue::Value(value, value_span),
        }
    }

    // jump to higher level and update it's indexes
    // it also mean jump to the block that represent for the parent of those items,
    // tell him that all your children were born and you need to know their name (index).
    #[reflection]
    fn rollup_indexes(&mut self, ast: &mut [JsonBlock], end: usize) -> crate::Result<()> {
        // the `state` holding the position of the parent object/array
        let Some(state) = self.pop_state() else {
            return Err(JsonError::custom(format!("[{__fn_ident}] {}", msg::SOON_EOS), Span::default()));
        };

        match state {
            // take the block locate at `pos`
            // if block's type is an Array, then process update its indexes
            State::Array(pos, indexes) => match ast.get_mut(pos) {
                Some(block) => if let JsonValue::Array(item_indexes, array_span) = &mut block.value {
                    item_indexes.extend(indexes);
                    array_span.end = end;
                },
                _ => return Err(JsonError::custom(format!("[{__fn_ident}] the JsonBlock at index {pos} is not an Array, cannot update indexes"), Span::default())),
            },
            // take the block locate at `pos`
            // if block's type is an Object, then process update its indexes
            State::Object(pos, indexes) => match ast.get_mut(pos) {
                Some(block) => if let JsonValue::Object(prop_indexes, obj_span) = &mut block.value {
                    prop_indexes.extend(indexes);
                    obj_span.end = end;
                },
                _ => return Err(JsonError::custom(format!("[{__fn_ident}] the JsonBlock at index {pos} is not an Object, cannot update indexes"), Span::default())),
            },
            // only Object and Array is allow to have items and nested children
            // Value should be one of primitive JSON supported's type
            _ => return Err(JsonError::custom(format!("[{__fn_ident}] not allow State::Value when rollup indexes"), Span::default())),
        };

        Ok(())
    }
}

impl JsonCursor {
    pub fn init(parser: &mut JsonParser<'_>) -> crate::Result<Self> {
        let Some(token) = parser.next_token() else {
            return Err(JsonError::custom("Reach the end of token stream, soon EOF", Span::default()));
        };

        match token {
            JsonToken::Punct(Punct::OpenCurly, _) => Ok(Self::new(State::Object(0, HashMap::new()))),
            JsonToken::Punct(Punct::OpenSquare, _) => Ok(Self::new(State::Array(0, Vec::new()))),
            JsonToken::Data(ty, span) => Ok(Self::new(State::Value(ty, span))),
            other_type => Err(JsonError::custom("Invalid JSON, should be comment, value, open-curly, open-square", other_type.get_span())),
        }
    }

    #[reflection]
    pub fn parse_object_prop(&mut self, parser: &mut JsonParser<'_>, ast: &mut [JsonBlock]) -> crate::Result<Option<JsonBlock>> {
        let key_span = match parser.next_token_skip(|tk| matches!(tk, JsonToken::Punct(Punct::WhiteSpace, _) | JsonToken::Comment(_))) {
            Some(JsonToken::Data(JsonType::Str(_), span)) => span.collapse(1),
            Some(JsonToken::Data(JsonType::Ident, span)) => span,
            // hitting the end of this object
            Some(JsonToken::Punct(Punct::CloseCurly, span)) => {
                self.rollup_indexes(ast, span.end)?;
                return Ok(None);
            }
            Some(JsonToken::Error(err, span)) => return Err(JsonError::custom(format!("[{__fn_ident}] {}", err), span)),
            Some(tk) => return Err(JsonError::custom(format!("[{__fn_ident}] expect JSON's key is a str/ident, found other"), tk.get_span())),
            None => return Err(JsonError::custom(format!("[{__fn_ident}] `key` should not be None"), Span::default()))
        };

        let _colon = match parser.next_token() {
            Some(JsonToken::Punct(Punct::Colon, cspan)) => cspan,
            Some(JsonToken::Error(err, span)) => return Err(JsonError::custom(format!("[{__fn_ident}] {}", err), span)),
            Some(tk) => return Err(JsonError::custom(format!("[{__fn_ident}] expect next token is a colon, found other"), tk.get_span())),
            None => return Err(JsonError::custom(format!("[{__fn_ident}] `colon` should not be None"), Span::default()))
        };

        let value = match parser.next_token_skip(|tk| matches!(tk, JsonToken::Punct(Punct::WhiteSpace | Punct::Plus | Punct::Minus, _))) {
            Some(JsonToken::Punct(Punct::OpenCurly, span)) => {
                self.update_prop_index(key_span, parser, ast.len())?;
                self.create_object_block(ast.len(), span)
            },
            Some(JsonToken::Punct(Punct::OpenSquare, span)) => {
                self.update_prop_index(key_span, parser, ast.len())?;
                self.create_array_block(ast.len(), span)
            },
            Some(JsonToken::Data(data @ JsonType::Str(_), data_span)) => self.create_prop_block(key_span, data, data_span.collapse(1), parser, ast.len())?,
            Some(JsonToken::Data(data, data_span)) => self.create_prop_block(key_span, data, data_span, parser, ast.len())?,
            Some(JsonToken::Error(err, span)) => return Err(JsonError::custom(format!("[{__fn_ident}] {}", err), span)),
            Some(tk) => return Err(JsonError::custom(format!("[{__fn_ident}] expect next token is primitive value, open-curly or open-square, found other"), tk.get_span())),
            None => return Err(JsonError::custom(format!("[{__fn_ident}] parsing prop value but reaching None"), Span::default())),
        };

        Ok(Some(value))
    }

    #[reflection]
    pub fn parse_array_item(&mut self, parser: &mut JsonParser<'_>, ast: &mut [JsonBlock]) -> crate::Result<Option<JsonBlock>> {
        let item_value = match parser.next_token_skip(|tk| matches!(tk, JsonToken::Punct(Punct::WhiteSpace | Punct::Plus | Punct::Minus, _) | JsonToken::Comment(_))) {
            Some(JsonToken::Data(data, data_span)) => self.create_item_block(ast.len(), data, data_span)?,
            Some(JsonToken::Punct(Punct::OpenCurly, span)) => {
                self.update_array_length(ast.len())?;
                self.create_object_block(ast.len(), span)
            },
            Some(JsonToken::Punct(Punct::OpenSquare, span)) => {
                self.update_array_length(ast.len())?;
                self.create_array_block(ast.len(), span)
            },
            // hitting the end of this array
            Some(JsonToken::Punct(Punct::CloseSquare, span)) =>  {
                self.rollup_indexes(ast, span.end)?;
                return Ok(None)
            },
            Some(JsonToken::Punct(_, span)) => return Err(JsonError::invalid_array(span)),
            Some(JsonToken::Error(err, span)) => return Err(JsonError::custom(format!("[{__fn_ident}] {}", err), span)),
            Some(JsonToken::Comment(span)) => return Err(JsonError::custom(format!("[{__fn_ident}] should not reaching this state, because all comments must be stripped all"), span)),
            None => return Err(JsonError::custom(format!("[{__fn_ident}] reaching None when parsing"), Span::default()))
        };

        Ok(Some(item_value))
    }

    // the whole source is single-value
    #[reflection]
    pub fn parse_value(&mut self, parser: &mut JsonParser<'_>) -> crate::Result<Option<JsonBlock>> {
        let next_item = parser.next_token_skip(|tk| matches!(tk, JsonToken::Punct(Punct::WhiteSpace | Punct::Plus | Punct::Minus, _) | JsonToken::Comment(_)));
        let item_value = match next_item {
            Some(JsonToken::Data(data, data_span)) => self.create_value_block(data, data_span),
            _ => return Err(JsonError::custom(format!("[{__fn_ident}] invalid json value"), Span::default())),
        };
        Ok(Some(item_value))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_simple_json() {
        let mut parser =
            JsonParser::new("{ a: 1, b: { c: 0x0F }, d: \"\", f: true, g: [1,[2,3],{h:1}]}");
        let out = parser.parse();

        assert!(out.inspect_err(|err| eprintln!("{err}")).is_ok());
    }

    #[test]
    fn parse_complex_json() {
        let mut parser =
            JsonParser::new("{ a: 1, b: [1,2, { d: { e: [{f:1}] } }], g: \"\n\t\", h: 0x9F }");
        let out = parser.parse();

        assert!(out.inspect_err(|err| eprintln!("{err}")).is_ok());
    }
}