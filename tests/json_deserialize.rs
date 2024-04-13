use std::marker::PhantomData;

use json_parser_macro::Deserialize;
use json_parser::prelude::*;

#[derive(Deserialize, PartialEq, Debug)]
struct Color<'c, T> {
    red: u8,
    green: u8,
    blue: u8,
    alpha: Option<u8>,
    _phantom: PhantomData<&'c T>,
}

#[test]
fn parse_color() -> Result<(), JsonError> {
    let mut color = JsonParser::new("{ red: 9, green: 10, blue: 11 }");
    let mut color_alpha = JsonParser::new("{ red: 9, green: 10, blue: 11, alpha: 1 }");

    let res = color.parse()?.parse_into::<Color<'static, String>>();
    let res2 = color_alpha.parse()?.parse_into::<Color<'static, String>>();

    assert_eq!(
        Ok(Color::<'static, String> {
            red: 9,
            green: 10,
            blue: 11,
            alpha: None,
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
            _phantom: PhantomData
        }),
        res2
    );

    Ok(())
}
