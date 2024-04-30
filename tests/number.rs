use jsode::{JsonIdx, JsonParser, JsonPsr, Result};


#[test]
fn zero() -> Result<()> {
    let mut obj = JsonParser::new("{ zero: 0 }");
    let out = obj.parse()?;

    assert_eq!(Ok(0), out.index("zero").unwrap().parse_into::<u8>());
    assert_eq!(Ok(0), out.index("zero").unwrap().parse_into::<u16>());
    assert_eq!(Ok(0), out.index("zero").unwrap().parse_into::<u32>());
    assert_eq!(Ok(0), out.index("zero").unwrap().parse_into::<u64>());
    assert_eq!(Ok(0), out.index("zero").unwrap().parse_into::<usize>());

    Ok(())
}

#[test]
fn u8() -> Result<()> {
    let mut obj = JsonParser::new("{ min: 0, max: 255 }");
    let out = obj.parse()?;

    assert_eq!(Ok(u8::MIN), out.index("min").unwrap().parse_into::<u8>());
    assert_eq!(Ok(u8::MAX), out.index("max").unwrap().parse_into::<u8>());
    Ok(())
}

#[test]
fn u16() -> Result<()> {
    let mut obj = JsonParser::new("{ min: 0, max: 65535 }");
    let out = obj.parse()?;

    assert_eq!(Ok(u16::MIN), out.index("min").unwrap().parse_into::<u16>());
    assert_eq!(Ok(u16::MAX), out.index("max").unwrap().parse_into::<u16>());
    Ok(())
}

#[test]
fn u32() -> Result<()> {
    let mut obj = JsonParser::new("{ min: 0, max: 4294967295 }");
    let out = obj.parse()?;

    assert_eq!(Ok(u32::MIN), out.index("min").unwrap().parse_into::<u32>());
    assert_eq!(Ok(u32::MAX), out.index("max").unwrap().parse_into::<u32>());
    Ok(())
}

#[test]
fn u64() -> Result<()> {
    let mut obj = JsonParser::new("{ min: 0, max: 18446744073709551615 }");
    let out = obj.parse()?;

    assert_eq!(Ok(u64::MIN), out.index("min").unwrap().parse_into::<u64>());
    assert_eq!(Ok(u64::MAX), out.index("max").unwrap().parse_into::<u64>());
    Ok(())
}

#[test]
fn usize() -> Result<()> {
    let mut obj = JsonParser::new("{ min: 0, max: 18446744073709551615 }");
    let out = obj.parse()?;

    assert_eq!(Ok(usize::MIN), out.index("min").unwrap().parse_into::<usize>());
    assert_eq!(Ok(usize::MAX), out.index("max").unwrap().parse_into::<usize>());
    Ok(())
}

#[test]
fn i8() -> Result<()> {
    let mut obj = JsonParser::new("{ min: -128, max: 127 }");
    let out = obj.parse()?;

    assert_eq!(Ok(i8::MIN), out.index("min").unwrap().parse_into::<i8>());
    assert_eq!(Ok(i8::MAX), out.index("max").unwrap().parse_into::<i8>());

    Ok(())
}

#[test]
fn i16() -> Result<()> {
    let mut obj = JsonParser::new("{ min: -32768, max: 32767 }");
    let out = obj.parse()?;

    assert_eq!(Ok(i16::MIN), out.index("min").unwrap().parse_into::<i16>());
    assert_eq!(Ok(i16::MAX), out.index("max").unwrap().parse_into::<i16>());

    Ok(())
}

#[test]
fn i32() -> Result<()> {
    let mut obj = JsonParser::new("{ min: -2147483648, max: 2147483647 }");
    let out = obj.parse()?;

    assert_eq!(Ok(i32::MIN), out.index("min").unwrap().parse_into::<i32>());
    assert_eq!(Ok(i32::MAX), out.index("max").unwrap().parse_into::<i32>());

    Ok(())
}

#[test]
fn i64() -> Result<()> {
    let mut obj = JsonParser::new("{ min: -9223372036854775808, max: 9223372036854775807 }");
    let out = obj.parse()?;

    assert_eq!(Ok(i64::MIN), out.index("min").unwrap().parse_into::<i64>());
    assert_eq!(Ok(i64::MAX), out.index("max").unwrap().parse_into::<i64>());

    Ok(())
}

#[test]
fn isize() -> Result<()> {
    let mut obj = JsonParser::new("{ min: -9223372036854775808, max: 9223372036854775807 }");
    let out = obj.parse()?;

    assert_eq!(Ok(isize::MIN), out.index("min").unwrap().parse_into::<isize>());
    assert_eq!(Ok(isize::MAX), out.index("max").unwrap().parse_into::<isize>());

    Ok(())
}

#[test]
fn f32() -> Result<()> {
    let mut obj = JsonParser::new("{ zero: 0.0, random: 10.5, only_frac: .5, expo: 1e10  }");
    let out = obj.parse()?;

    assert_eq!(Ok(0.0), out.index("zero").unwrap().parse_into::<f32>());
    assert_eq!(Ok(10.5), out.index("random").unwrap().parse_into::<f32>());
    assert_eq!(Ok(0.5), out.index("only_frac").unwrap().parse_into::<f32>());
    assert_eq!(Ok(1e10), out.index("expo").unwrap().parse_into::<f32>());
    Ok(())
}

#[test]
fn f64() -> Result<()> {
    let mut obj = JsonParser::new(r#"{
        min: -1.7976931348623157e308,
        max: 1.7976931348623157e+308,
        zero_frac: 0.5,
        random: 19.8,
        only_frac: .5,
        expo: 1e10,
    }"#);
    let out = obj.parse()?;

    assert_eq!(Ok(f64::MIN), out.index("min").unwrap().parse_into::<f64>());
    assert_eq!(Ok(f64::MAX), out.index("max").unwrap().parse_into::<f64>());
    assert_eq!(Ok(0.5), out.index("zero_frac").unwrap().parse_into::<f64>());
    assert_eq!(Ok(19.8), out.index("random").unwrap().parse_into::<f64>());
    assert_eq!(Ok(0.5), out.index("only_frac").unwrap().parse_into::<f64>());
    assert_eq!(Ok(1e10), out.index("expo").unwrap().parse_into::<f64>());
    Ok(())
}

#[test]
fn exponential() -> Result<()> {
    let mut obj = JsonParser::new("{ pos: 1e10, neg: -2e-10 }");
    let out = obj.parse()?;
    assert_eq!(Ok(1e10), out.index("pos").unwrap().parse_into::<f64>());
    assert_eq!(Ok(-2e-10), out.index("neg").unwrap().parse_into::<f64>());
    Ok(())
}


#[test]
fn hex() -> Result<()> {
    let mut obj = JsonParser::new("{ space: 0x20, dollar: 0X24, small_y: 0xFF }");
    let out = obj.parse()?;

    assert_eq!(Ok(32), out.index("space").unwrap().parse_into::<u8>());
    assert_eq!(Ok(36), out.index("dollar").unwrap().parse_into::<u8>());
    assert_eq!(Ok(255), out.index("small_y").unwrap().parse_into::<u8>());
    Ok(())
}

#[test]
fn keyword() -> Result<()> {
    let mut obj = JsonParser::new("{ inf: Infinity, nan: NaN }");
    let out = obj.parse()?;

    assert_eq!(Ok(f32::INFINITY), out.index("inf").unwrap().parse_into::<f32>());
    assert_eq!(Ok(f64::INFINITY), out.index("inf").unwrap().parse_into::<f64>());
    assert!(out.index("nan").unwrap().parse_into::<f32>().is_ok_and(|it| it.is_nan()));
    assert!(out.index("nan").unwrap().parse_into::<f64>().is_ok_and(|it| it.is_nan()));

    Ok(())
}

#[test]
fn hotspot() -> Result<()> {
    let mut obj = JsonParser::new(
    r#"{
        int: 100,
        pos_int: +100,
        neg_int: -100,
        dec: 99,
        decimal: 99.99,
        pos_decimal: +99.99,
        neg_decimal: -99.99,
        hex: 0X20,
        pos_hex: +0x20,
        neg_hex: -0x20,
        expo: 1e10,
        pos_expo: +1e10,
        neg_expo: -1e10,
        neg_expo_neg: -1e-10,
        infinity: Infinity,
        nan: NaN,
    }"#);
    let out = obj.parse()?;

    assert_eq!(Ok(100), out.index("int").unwrap().parse_into::<i8>());
    assert_eq!(Ok(100), out.index("pos_int").unwrap().parse_into::<i8>());
    assert_eq!(Ok(-100), out.index("neg_int").unwrap().parse_into::<i8>());

    assert_eq!(Ok(99.00), out.index("dec").unwrap().parse_into::<f32>());
    assert_eq!(Ok(99.99), out.index("decimal").unwrap().parse_into::<f32>());
    assert_eq!(Ok(99.99), out.index("pos_decimal").unwrap().parse_into::<f32>());
    assert_eq!(Ok(-99.99), out.index("neg_decimal").unwrap().parse_into::<f32>());

    assert_eq!(Ok(1e10), out.index("expo").unwrap().parse_into::<f64>());
    assert_eq!(Ok(1e10), out.index("pos_expo").unwrap().parse_into::<f64>());
    assert_eq!(Ok(-1e10), out.index("neg_expo").unwrap().parse_into::<f64>());
    assert_eq!(Ok(-1e-10), out.index("neg_expo_neg").unwrap().parse_into::<f64>());

    assert_eq!(Ok(0x20), out.index("hex").unwrap().parse_into::<i8>());
    assert_eq!(Ok(0x20), out.index("pos_hex").unwrap().parse_into::<i8>());
    assert_eq!(Ok(-0x20), out.index("neg_hex").unwrap().parse_into::<i8>());

    Ok(())
}