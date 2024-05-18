
pub(crate) mod msg {
    pub const MISSING_SINGLE_COLON: &str = "missing string's close character \'";
    pub const MISSING_DOUBLE_COLON: &str = "missing string's close character \"";
    pub const NOT_SUPPORT_TOKEN:    &str = "not support this token";
    pub const EMPTY_JSON:           &str = "json input is empty";
    pub const INVALID_ARRAY:        &str = "not allow this token when parsing array";
    pub const DUPLICATE_KEY:        &str = "already exist key";
    pub const INVALID_ESCAPE:       &str = "the following escape string is not allow";
    pub const SOON_EOS:             &str = "No more state in stack, soon EOS";
}

pub(crate) mod ascii {
    pub const BACKSPACE: u8 = 0x08;
    pub const HORIZONTAL_TAB: u8 = 0x09;
    pub const LINE_FEED: u8 = 0x0A;
    pub const FORM_FEED: u8 = 0x0C;
    pub const SPACE: u8 = 0x20;
    pub const CARRIAGE_RETURN: u8 = 0x0D;
    pub const NON_BREAKING_SPACE: u8 = 0xA0;
    pub const PARAGRAPH_SEPARATOR: u16 = 0x2029;
    pub const LINE_SEPARATOR: u16 = 0x2028;
    pub const BYTE_ORDER_MARK: u16 = 0xFEFF;

    pub const ESCAPE: u8 = b'\\';
    pub const SINGLE_QUOTE: u8= b'\'';
    pub const DOUBLE_QUOTE: u8= b'\"';
}