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
fn f32() -> Result<()> {
    let mut obj = JsonParser::new("{ zero: 0.0, random: 10.5, only_frac: .5 }");
    let out = obj.parse()?;

    assert_eq!(Ok(0.0), out.index("zero").unwrap().parse_into::<f32>());
    assert_eq!(Ok(10.5), out.index("random").unwrap().parse_into::<f32>());
    assert_eq!(Ok(0.5), out.index("only_frac").unwrap().parse_into::<f32>());
    Ok(())
}

#[test]
fn f64() -> Result<()> {
    let mut obj = JsonParser::new("{ min: 0.0, max: 10.5 }");
    let out = obj.parse()?;

    assert_eq!(Ok(0.0), out.index("min").unwrap().parse_into::<f64>());
    assert_eq!(Ok(10.5), out.index("max").unwrap().parse_into::<f64>());
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