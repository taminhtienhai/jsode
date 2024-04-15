use jsode::prelude::*;

#[derive(Deserialize, PartialEq, Debug)]
struct Color {
    #[prop = "r"]
    red: u8,
    #[prop = "b"]
    blue: u8,
    green: u8,
}

fn main() -> jsode::Result<()> {
    let mut src = JsonParser::new(r#"{
        'r': 255,
        'b': 96,
        'green': 0,
    }"#);
    let ast = src.parse()?;

    let expected = Color {
        red: 255,
        blue: 96,
        green: 0,
    };
    assert_eq!(expected, ast.parse_into::<Color>()?);

    Ok(())
}