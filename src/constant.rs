
pub(crate) mod msg {
    pub const MISSING_SINGLE_COLON: &str = "missing string's close character \'";
    pub const MISSING_DOUBLE_COLON: &str = "missing string's close character \"";
    pub const NOT_SUPPORT_TOKEN:    &str = "not support this token";
    pub const EMPTY_JSON:           &str = "json input is empty";
    pub const INVALID_ARRAY:        &str = "not allow this token when parsing array";
    pub const DUPLICATE_KEY:        &str = "already exist key";
}

pub(crate) mod ascii {
    pub const HORIZONTAL_TAB: u8 = 0x09;
    pub const SPACE: u8 = 0x20;
    pub const LINE_FEED: u8 = 0x0A;
    pub const CARRIAGE_RETURN: u8 = 0x0D;
}