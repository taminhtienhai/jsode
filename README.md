<h1 align="center">JSON :heart: Oxide = Jsode</h1>

## Overview

Simple, zero-copy & zero-dependency JSON Parser

### Install

```bash
cargo add jsode
```

### Getting Started

#### 1. Index JSON key

```rust
use jsode::prelude::*;

fn main() -> jsode::Result<()> {
    let mut src = JsonParser::new("{ 'hello': 'world' }");
    let ast = src.parse()?;

    assert!(ast.index("hello").is_some());
    assert!(ast.index("none_exist_key").is_none());

    Ok(())
}
```

#### 2. Getting/Deserialize single JSON's property

```rust
use jsode::prelude::*;

fn main() -> jsode::Result<()> {
    let mut src = JsonParser::new("{ 'hello': 'world' }");
    let ast = src.parse()?;

    assert_eq!("world", ast.index("hello").unwrap().parse_into::<String>()?);

    Ok(())
}
```

#### 3. Deserialize into struct

```rust
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
```