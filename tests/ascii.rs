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
    let hex = [b'\\', b'\t'];
    let tab = std::str::from_utf8(&hex);
    println!("{tab:?}");

    println!("{}", b'\t');
    println!("{}", b't');
}

#[test]
fn parse_decimal() {
    let dec = "0x1f";
    let neg_dec = "0x-1f";
    let decimal = ".3";
    let minus_int = "-1";
    let big_num = "1e10";
    let zero_prefix = 123.0e+3;

    println!("{}", zero_prefix);

    assert_eq!(Ok(31), i64::from_str_radix(&dec[2..], 16));
    assert_eq!(Ok(-31), i64::from_str_radix(&neg_dec[2..], 16));
    assert_eq!(Ok(0.3), decimal.parse::<f32>());
    assert_eq!(Ok(-1), minus_int.parse::<i8>());
    assert_eq!(Ok(1e10), big_num.parse::<f64>());
}

#[test]
fn base_number() {
    let uint8: u8 = u8::MAX;
    println!("[u8] Big Edian: {:?}", uint8.to_be_bytes());
    println!("[u8] Little Edian: {:?}", uint8.to_le_bytes());
    println!("[u8] Native Edian: {:?}", uint8.to_ne_bytes());
    println!("---------------------------------------------");

    let uint16: u16 = 256;
    println!("[u16] Big Edian: {:?}", uint16.to_be_bytes());
    println!("[u16] Little Edian: {:?}", uint16.to_le_bytes());
    println!("[u16] Native Edian: {:?}", uint16.to_ne_bytes());
    println!("---------------------------------------------");

    let uint32: u32 = 255;
    println!("[u32] Big Edian: {:?}", uint32.to_be_bytes());
    println!("[u32] Little Edian: {:?}", uint32.to_le_bytes());
    println!("[u32] Native Edian: {:?}", uint32.to_ne_bytes());
    println!("---------------------------------------------");

    let uintu64: u64 = 255;
    println!("[u64] Big Edian: {:?}", uintu64.to_be_bytes());
    println!("[u64] Little Edian: {:?}", uintu64.to_le_bytes());
    println!("[u64] Native Edian: {:?}", uintu64.to_ne_bytes());
    println!("---------------------------------------------");

    let int64: i16 = 0;
    println!("[i64] Big Edian: {:?}", int64.to_be_bytes());
    println!("[i64] Little Edian: {:?}", int64.to_le_bytes());
    println!("[i64] Native Edian: {:?}", int64.to_ne_bytes());
    println!("---------------------------------------------");

    println!("{}", ("29".parse::<u8>().unwrap() as i8));

    let int64: i64 = 255;
    println!("[i64] Big Edian: {:?}", int64.to_be_bytes());
    println!("[i64] Little Edian: {:?}", int64.to_le_bytes());
    println!("[i64] Native Edian: {:?}", int64.to_ne_bytes());
    println!("---------------------------------------------");

    let float32: f32 = 1.27;
    println!("[f32] Big Edian: {:?}", float32.to_be_bytes());
    println!("[f32] Little Edian: {:?}", float32.to_le_bytes());
    println!("[f32] Native Edian: {:?}", float32.to_ne_bytes());
    println!("---------------------------------------------");

    let float64: f64 = 1.27;
    println!("[f64] Big Edian: {:?}", float64.to_be_bytes());
    println!("[f64] Little Edian: {:?}", float64.to_le_bytes());
    println!("[f64] Native Edian: {:?}", float64.to_ne_bytes());
}