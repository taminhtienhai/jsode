<h1 align="center">JSON :heart: Oxide = Jsode</h1>

## Overview

Easy parsing JSON source and transform it into interaction Rust type.

### Getting Started

1. Install

```toml
[dependencies]
jsode = { version = "0.1" }
```

2. Parsing AST

```rust
use jsode::prelude::*;

// `jsonde::Result` already imported via prelude mod, so you only need to write `-> Result<()>`.
// I just wrote down it's full path to make sure you don't value it as `std::result::Result`.
fn main() -> jsode::Result<()> {
    let src = JsonParser::new("{ 'hello': 'world' }")

    assert!(src.parse().is_ok());

    Ok(())
}
```

3. Index json key

```rust
use jsode::prelude::*;

fn main() -> jsode::Result<()> {
    let src = JsonParser::new("{ 'hello': 'world' }")
    let ast = src.parse()?;

    assert!(ast.index("hello").is_some());
    assert!(ast.index("none_exist_key").is_none());

    Ok(())
}
```

4. Getting/Deserialize json's property

```rust
use jsode::prelude::*;

fn main() -> jsode::Result<()> {
    let src = JsonParser::new("{ 'hello': 'world' }")
    let ast = src.parse()?;

    assert_eq!(Some("world"), ast.index("hello").parse_into::<String>()?);

    Ok(())
}
```

5. Deserialize into Rust struct

```rust
use jsode::prelude::*;

#[Deserialize, PartialEq, Debug]
struct Color {
    red: u8,
    green: u8,
    blue: u8,
}

fn main() -> jsode::Result<()> {
    let src = JsonParser::new(r#"{
        'red': 255,
        'blue': 96,
        'green': 0
    }"#);
    let ast = src.parse()?;

    let expected = Color {
        red: 255,
        blue: 96,
        green: 0,
    };
    assert_eq!(Ok(expected), ast.parse_into::<Color>()?);

    Ok(())
}
```