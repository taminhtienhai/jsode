use jsode::prelude::*;

#[test]
fn sample1() -> Result<()> {
    let mut json = JsonParser::new(include_str!("../resources/valid/sample1.json"));
    let out = json.parse()?;

    assert_eq!(Ok("Apple"), out.index("fruit").unwrap().parse_into::<String>().as_deref());
    assert_eq!(Ok("Large"), out.index("size").unwrap().parse_into::<String>().as_deref());
    assert_eq!(Ok("Red"), out.index("color").unwrap().parse_into::<String>().as_deref());
    assert_eq!(Ok("\t"), out.index("escape").unwrap().parse_into::<String>().as_deref());
    Ok(())
}


#[test]
fn sample2() -> Result<()> {
    #[derive(Deserialize, PartialEq, Debug)]
    struct Address {
        #[prop = "streetAddress"]
        street_address: String,
        city: String,
        state: String,
    }
    #[derive(Deserialize, PartialEq, Debug)]
    struct Phone {
        #[prop = "type"]
        ty: String,
        number: String,
    }
    #[derive(Deserialize)]
    struct Sample2 {
        #[prop = "firstName"]
        first_name: String,
        #[prop = "lastName"]
        last_name: String,
        gender: String,
        age: u8,
        address: Address,
        #[prop = "phoneNumbers"]
        phone_numbers: Vec<Phone>,
    }
    let mut json = JsonParser::new(include_str!("../resources/valid/sample2.json"));

    let Sample2 { first_name, address, phone_numbers, ..} = json.parse()?.parse_into::<Sample2>().unwrap();

    assert_eq!("Joe", first_name);
    assert_eq!(Address {
        street_address: "101".to_string(),
        city: "San Diego".to_string(),
        state: "CA".to_string(),
    }, address);
    assert_eq!(vec![
        Phone {ty: "home".to_string(), number: "7349282382".to_string()}
    ], phone_numbers);
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

#[test]
fn sample5() -> Result<()> {
    let mut json = JsonParser::new(include_str!("../resources/valid/sample5.json5"));
    let out = json.parse()?;

    assert_eq!(Ok("and you can quote me on that"), out.index("unquoted").unwrap().parse_into::<String>().as_deref());
    assert_eq!(Ok("I can use \"double quotes\" here"), out.index("singleQuotes").unwrap().parse_into::<String>().as_deref());
    assert_eq!(Ok("Look, Mom! \nNo \\n's!"), out.index("lineBreaks").unwrap().parse_into::<String>().as_deref());
    assert_eq!(Ok(vec!["arrays".to_string()]), out.index("andIn").unwrap().parse_into::<Vec<String>>());
    // assert_eq!(Ok(912559), out.index("hexadecimal").unwrap().parse_into::<usize>());
    // assert_eq!(Ok(8675309), out.index("andTrailing").unwrap().parse_into::<usize>());
    
    Ok(())
}

#[test]
fn sample6() -> Result<()> {
    let mut json = JsonParser::new(include_str!("../resources/valid/sample6.json5"));
    let out = json.parse()?;

    assert_eq!("\\\"".as_bytes(), out.index("quote").unwrap().to_bytes());
    assert_eq!([b'\\', b'\\'], out.index("reserver_solidus").unwrap().to_bytes());
    assert_eq!([b'\\', b'b'] , out.index("backspace").unwrap().to_bytes());
    assert_eq!([b'\\', b'f'] , out.index("formfeed").unwrap().to_bytes());
    assert_eq!([b'\\', b'n'] , out.index("newline").unwrap().to_bytes());
    assert_eq!([b'\\', b'r'] , out.index("carriage_return").unwrap().to_bytes());
    assert_eq!("\\t \\\", { tab: \\\"t\\\" }".as_bytes(), out.index("tab").unwrap().to_bytes());
    assert_eq!("\\\" \\\' \\\"".as_bytes(), out.index("single_quote").unwrap().to_bytes());
    assert_eq!(r"\u032c \\ \/ \b \f \n \r \\\r\\\\\/ \t".as_bytes(), out.index("all").unwrap().to_bytes());
    assert_eq!(r"\x5C \u005C\u005c".as_bytes(), out.index("special").unwrap().to_bytes());

    Ok(())
}

#[test]
fn sample7() -> Result<()> {
    let mut json = JsonParser::new(include_str!("../resources/valid/sample7.json5"));
    let out = json.parse()?;

    let raw_json = out.index("raw_json").unwrap().parse_into::<String>().unwrap();

    let mut nested_json = JsonParser::new(&raw_json);
    let nested_out = nested_json.parse()?;

    assert_eq!("barrrr", nested_out.index("foo").unwrap().parse_into::<String>()?);

    Ok(())
}