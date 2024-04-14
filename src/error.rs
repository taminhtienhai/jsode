use core::fmt::Display;
use std::fmt::Debug;

use crate::core::Span;
use crate::constant::msg;

#[derive(PartialEq, Debug)]
pub enum ErrorMsg {
    MissingSingleColon,
    MissingDoubleColon,
    EmptyJson,
    InvalidArray,
    Custom(String),
}

#[derive(PartialEq)]
pub struct JsonError {
    span: Span,
    msg: ErrorMsg,
}

#[rustfmt::skip]
fn display_error(err: &ErrorMsg, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match err {
        ErrorMsg::MissingDoubleColon => write!(f, "{}", msg::MISSING_DOUBLE_COLON),
        ErrorMsg::MissingSingleColon => write!(f, "{}", msg::MISSING_SINGLE_COLON),
        ErrorMsg::EmptyJson          => write!(f, "{}", msg::EMPTY_JSON),
        ErrorMsg::InvalidArray       => write!(f, "{}", msg::INVALID_ARRAY),
        ErrorMsg::Custom(msg)        => write!(f, "{:?}", msg),
    }
}

impl Debug for JsonError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "source file panic at {}..{}", self.span.start, self.span.end)?;
        display_error(&self.msg, f)
    }
}

#[rustfmt::skip]
impl Display for JsonError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        display_error(&self.msg, f)
    }
}

#[rustfmt::skip]
impl JsonError {
    pub fn custom(msg: impl Into<String>, span: Span) -> Self {
        Self { span, msg: ErrorMsg::Custom(msg.into()), }
    }

    pub fn missing_single_colon(span: Span) -> Self {
        Self { span, msg: ErrorMsg::MissingSingleColon, }
    }

    pub fn missing_double_colon(span: Span) -> Self {
        Self { span, msg: ErrorMsg::MissingDoubleColon, }
    }

    pub fn invalid_array(span: Span) -> Self {
        Self { span, msg: ErrorMsg::InvalidArray, }
    }

    pub fn empty_json(span: Span) -> Self {
        Self { span, msg: ErrorMsg::EmptyJson, }
    }
}