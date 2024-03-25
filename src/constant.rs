pub const MISSING_SINGLE_COLON: &str = "missing string's close character \'";
pub const MISSING_DOUBLE_COLON: &str = "missing string's close character \"";
pub const UNEXPECTED_EOS: &str = "current token haven't parsed completely but EOS";


pub mod ascii {
    pub const HORIZONTAL_TAB: u8 = 0x09;
    pub const SPACE: u8 = 0x20;
    pub const LINE_FEED: u8 = 0x0A;
    pub const CARRIAGE_RETURN: u8 = 0x0D;
}