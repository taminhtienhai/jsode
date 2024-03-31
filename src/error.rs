use core::fmt::Display;

use crate::core::Span;


#[derive(Debug)]
pub enum ErrorMsg {
    MISSING_SINGLE_COLON,
    MISSING_DOUBLE_COLON,
    EMPTY_JSON,
    INVALID_ARRAY,
    CUSTOM(String),
}

#[derive(Debug)]
pub struct JsonError {
    span: Span,
    msg: ErrorMsg,
}

impl Display for JsonError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.msg {
            ErrorMsg::MISSING_DOUBLE_COLON => write!(f, "missing string's close character \""),
            ErrorMsg::MISSING_SINGLE_COLON => write!(f, "missing string's close character \'"),
            ErrorMsg::EMPTY_JSON => write!(f, "json input is empty"),
            ErrorMsg::INVALID_ARRAY => write!(f, "not allow this token when parsing array"),
            ErrorMsg::CUSTOM(ref msg) => write!(f, "{:?}", msg),
        }
    }
}

impl JsonError {
    pub fn custom(msg: impl Into<String>, span: Span) -> Self {
        Self { span, msg: ErrorMsg::CUSTOM(msg.into()) }
    }

    pub fn missing_single_colon(span: Span) -> Self {
        Self { span, msg: ErrorMsg::MISSING_SINGLE_COLON, }
    }

    pub fn missing_double_colon(span: Span) -> Self {
        Self { span, msg: ErrorMsg::MISSING_DOUBLE_COLON, }
    }

    pub fn invalid_array(span: Span) -> Self {
        Self { span, msg: ErrorMsg::INVALID_ARRAY, }
    }

    pub fn empty_json(span: Span) -> Self {
        Self { span, msg: ErrorMsg::EMPTY_JSON, }
    }
}