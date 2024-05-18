use jsode::prelude::*;

#[test]
fn invidiual_u8() {
    let mut parser = JsonParser::new("1");
    let out = parser.parse().unwrap().parse_into::<u8>();
    assert_eq!(Ok(1), out);
}

#[test]
fn invidiual_i8() {
    let mut parser = JsonParser::new("-1");
    let out = parser.parse().unwrap().parse_into::<i8>();
    assert_eq!(Ok(-1), out);
}

#[test]
fn invidiual_true() {
    let mut parser = JsonParser::new("true");
    let out = parser.parse().unwrap().parse_into::<bool>();
    assert_eq!(Ok(true), out);
}

#[test]
fn invidiual_false() {
    let mut parser = JsonParser::new("false");
    let out = parser.parse().unwrap().parse_into::<bool>();
    assert_eq!(Ok(false), out);
}

#[test]
fn invidiual_str() {
    let mut parser = JsonParser::new("\"string\"");
    let out = parser.parse().unwrap().parse_into::<String>();
    assert_eq!(Ok("string".into()), out);
}

#[test]
fn invidiual_vec() {
    let mut parser = JsonParser::new("[1,2,3,4]");
    let out = parser.parse().unwrap().parse_into::<Vec<u8>>();
    assert_eq!(Ok(vec![1,2,3,4]), out);
}