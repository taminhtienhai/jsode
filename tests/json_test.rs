use jsode::prelude::*;

#[test]
fn sample1() -> Result<()> {
    let mut json = JsonParser::new(include_str!("../resources/valid/sample1.json"));
    let out = json.parse()?;

    assert_eq!(Ok("Apple"), out.index("fruit").unwrap().parse_type::<String>().as_deref());
    assert_eq!(Ok("Large"), out.index("size").unwrap().parse_type::<String>().as_deref());
    assert_eq!(Ok("Red"), out.index("color").unwrap().parse_type::<String>().as_deref());
    Ok(())
}


#[test]
fn sample2() -> Result<()> {
    #[derive(Deserialize, PartialEq, Debug)]
    struct Address {
        streetAddress: String,
        city: String,
        state: String,
    }
    #[derive(Deserialize, PartialEq, Debug)]
    struct Phone {
        _type: String,
        number: String,
    }
    #[derive(Deserialize)]
    struct Sample2 {
        firstName: String,
        lastName: String,
        gender: String,
        age: u8,
        address: Address,
        phoneNumbers: Vec<Phone>,
    }
    let mut json = JsonParser::new(include_str!("../resources/valid/sample2.json"));

    let Sample2 { firstName, address, phoneNumbers, ..} = json.parse()?.parse_into::<Sample2>().unwrap();

    assert_eq!("Joe", firstName);
    assert_eq!(Address {
        streetAddress: "101".to_string(),
        city: "San Diego".to_string(),
        state: "CA".to_string(),
    }, address);
    assert_eq!(vec![
        Phone {_type: "home".to_string(), number: "7349282382".to_string()}
    ], phoneNumbers);
    Ok(())
}

#[allow(clippy::get_first)]
#[test]
fn sample3() -> Result<()> {
    #[derive(Deserialize, PartialEq, Debug)]
    struct Color {
        color: String,
        value: String,
    }
    impl Color { pub fn new(c: impl Into<String>, v: impl Into<String>) -> Self {
        Self { color: c.into(), value: v.into() }
    } }
    let mut json = JsonParser::new(include_str!("../resources/valid/sample3.json5"));
    let out = json.parse()?.parse_into::<Vec<Color>>().unwrap();
    assert!(out.len() == 7);
    assert_eq!(Some(&Color::new("red", "#f00")), out.get(0));
    assert_eq!(Some(&Color::new("green", "#0f0")), out.get(1));
    assert_eq!(Some(&Color::new("blue", "#00f")), out.get(2));
    assert_eq!(Some(&Color::new("cyan", "#0ff")), out.get(3));
    assert_eq!(Some(&Color::new("magenta", "#f0f")), out.get(4));
    assert_eq!(Some(&Color::new("yellow", "#ff0")), out.get(5));
    assert_eq!(Some(&Color::new("black", "#000")), out.get(6));
    Ok(())
}