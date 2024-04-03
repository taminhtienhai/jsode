use crate::{core::{JsonType, JsonValue, Span}, error::JsonError, lexer::Tokenizer, parser::JsonParser};

pub trait Deser where Self: Sized {
    fn parse(parser: &JsonParser<Tokenizer<'_>>, value: &JsonValue) -> Result<Self, JsonError>;
}

pub trait Deserialize {
    fn deser<Out: Deser>(&self) -> Result<Out, JsonError>;
}

pub struct Deserializer<'de> {
    parser: &'de JsonParser<Tokenizer<'de>>,
    value: &'de JsonValue,
}

impl <'de> Deserialize for Deserializer<'de> {
    fn deser<Out: Deser>(&self) -> Result<Out, JsonError> {
        Out::parse(self.parser, self.value)
    }
}

impl Deser for usize {
    fn parse(parser: &JsonParser<Tokenizer<'_>>, ast: &JsonValue) -> Result<Self, JsonError> {
        #[allow(clippy::collapsible_match)]
        match ast {
            JsonValue::Data(ty, span) => match ty {
                JsonType::Num => {
                    let slice = parser.take_slice(span.clone())?;
                    Ok(slice
                        .parse()
                        .map_err(|_| JsonError::custom("not a valid usize", Span::default()))?)
                },
                _ => Err(JsonError::custom("not type usize", Span::default())),
            },
            _ => Err(JsonError::custom("not type usize", Span::default())),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_usize() {
        // let mut obj = JsonParser::new("12345678");
        // let     ast = obj.parse().unwrap();
        // let indexer = obj.indexer_from(&ast);
        
        // let us = indexer.index("usize").map(|value| value.);
    }
}