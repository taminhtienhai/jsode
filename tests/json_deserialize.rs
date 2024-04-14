use std::marker::PhantomData;

use jsode::prelude::*;

#[derive(Deserialize, PartialEq, Debug)]
struct Color<'c, T> {
    red: u8,
    green: u8,
    blue: u8,
    alpha: Option<u8>,
    hue: Vec<Hue>,
    _phantom: PhantomData<&'c T>,
}

#[derive(Deserialize, PartialEq, Debug)]
struct Hue {
    h: u8,
}

impl Hue {
    pub fn new(h: u8) -> Self {
        Self { h }
    }
}

#[test]
fn parse_color() -> Result<()> {
    let mut color = JsonParser::new("{ red: 9, green: 10, blue: 11, hue: [{h:1}] }");
    let mut color_alpha = JsonParser::new("{ red: 9, green: 10, blue: 11, alpha: 1, hue: [{h:1},{h:2}], }");

    let res = color.parse()?.parse_into::<Color<'static, String>>();
    let res2 = color_alpha.parse()?.parse_into::<Color<'static, String>>();

    assert_eq!(
        Ok(Color::<'static, String> {
            red: 9,
            green: 10,
            blue: 11,
            alpha: None,
            hue: vec![Hue::new(1)],
            _phantom: PhantomData
        }),
        res
    );
    assert_eq!(
        Ok(Color::<'static, String> {
            red: 9,
            green: 10,
            blue: 11,
            alpha: Some(1),
            hue: vec![Hue::new(1), Hue::new(2)],
            _phantom: PhantomData
        }),
        res2
    );

    Ok(())
}
