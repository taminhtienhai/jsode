use jsode::JsonError;


#[test]
fn escape_character() {

    println!("{:?}", "\\ U+00FF".as_bytes());
    println!("{}", b'\"');
    println!("{}",  std::str::from_utf8(&[b'\\']).unwrap());

    println!("\u{62}");


    println!("{}", b'+');
    println!("{}", b'-');
}

#[test]
fn hex_to_utf8() {
    let hex = [92];
    let str = std::str::from_utf8(&hex);
    println!("{str:?}");
}

#[test]
fn special_char() {
    // let hex = [b'\\', b'\t'];
    // let tab = std::str::from_utf8(&hex);
    // println!("{tab:?}");

    println!("{}", b'\t');
    println!("{}", b't');
}

#[test]
fn parse_decimal() {
    let dec = "0x1f";
    let int = "123";
    let decimal = ".3";
    let minus_int = "-1";

    assert_eq!(Ok(31), i64::from_str_radix(&dec[2..], 16));
    assert_eq!(Ok(0.3), decimal.parse::<f32>());
    assert_eq!(Ok(-1), minus_int.parse::<i8>());
}