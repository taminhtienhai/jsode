# Project Roadmap

### Ideas

1. allow user define their own custom placeholder then inject after

Example:
```rust
fn main() -> jsode::Result<()> {
    // this tell parser to replace both key & value
    let dollar = Label::new("currency", "dollar");

    let parser = JSONParser::new("{ $currency: "100_$currency" }");
    let ast = parser.parse(dollar)?;

    assert_eq!(Ok("100_dollar"), ast.index("dollar").unwrap().parse_into::<String>()?);
    assert_eq!(Ok("100_dollar"), ast.index(dollar).unwrap().parse_into::<String>()?);

    Ok(())
}
```

If the following deserializaton doesn't include related label, it could be fail.

We can tell the parser that only replace the JSON's key:

```rust
let dollar = Label::key_only("currency", "dollar");
```

Similar in case value-only:

```rust
let dollar = Label::value_only("currency", "dollar");
```

Everything done lazy. Nothing happened until use call `parse_into`.

### Bugs

- [x] `get_span` inside `parse_obj` auto increase `start` & `end` by 1???
- [x] breaking when meet eacapse token ('\') 

## Improvements

- [x] use const fn when possible
- [ ] enhance keyword lookup

## Benchmark (large-file.json - 26mb)

jsode_v1: 0.21s user 0.14s system 99% cpu 0.349 total
jsode_v2: 0.30s user 0.07s system 99% cpu 0.310 total
json_serde: 0.15s user 0.07s system 86% cpu 0.251 total

## Issues

- [x] convert `Option<Result<u8,JsonError>>` -> `Result<Option<u8>,JsonError>`
- [x] rename this crate
- [x] apply github CI/CD
- [x] remove invalid keyword (Undefined)
- [ ] make `unsafe` resonate
- [ ] support `impl_deserialize` macro_rules, generate `iml Deserialize` trait on input types
- [ ] write document
- [ ] bring some usecases/examples
- [ ] revamp error leverage derive macro
- [ ] test on real & large json file
- [ ] add benchmark

## Todo

- [x] impl new JsonParser (v2)
    - [x] store AST on linear array (avoid recursive which causing stack overflow)
    - [x] rework Indexer
        - [x] reindex object value from absolute -> relative
    - [x] rework Deserialize
- [ ] rework Deserialize phase to entirely remove recursive
- [ ] Optimize
    - [ ] `common::hash_str`
    - [ ] replace `HashMap` with better solution
    - [ ] check `move_backward_then_consume_until` method

## Road to 0.1

- [x] support prelude module
- [x] enhance derive macro
    - [x] support Option property
    - [x] support lifetime
    - [x] support phantom data
    - [x] support parse Array (Vec<T>, ~~&[T]~~, ..)
    - ~~[ ] support tuple struct~~ (impossible because tuple don't have keys, tuple layout also look more like a array)
- [x] prefer using HashMap to store key rather than Vec
- [x] support property `#[prop = $prop_name]` for field mapping
- [x] completely zero copy

## Road to 0.2

- [ ] make it pass all test cases in JSON TestSuite
- [x] revamp project base on [JSON5](https://spec.json5.org/) specification
    - [x] escape character
    - [x] number
        - [x] integer
        - [x] hexadecimal
        - [x] fractional
        - [x] exponential
    - [x] new keywords (Infinity, NaN)
    - [x] comment
        - [x] single-line
        - [x] multi-line
- [ ] row & column tracking
- [ ] enhance error message
    - [ ] Diagnostic struct (visualize location of error on input source)
- [ ] more test cases
- [ ] benchmark

## Road to 0.3

- [ ] support pattern query
- [ ] support `Lazy<'l, T: Deserialize>` struct, benefit you to execute operator like eq(==), le(<), gt(>), gte(>=) on value without derialize it
- [ ] support property `#[msg = $err_msg]` for custom error message
- [ ] impl Deserialize on more type
    - [ ] `&[T]`
    - [ ] `HashMap<String, T>`

## Road to 0.4

- [ ] support `Serialize` macro (multi targets)
- [ ] support `no_std`
- [ ] support better number variants parsing (ex: 65_535, 2e16, 0x234, 2E, ...)
- [ ] support compile-time validation macro
- [ ] accept file as argument
    - [ ] support memory mapping via [`memmap2`](https://crates.io/crates/memmap2) crate (optional)

## Road to 1.0

- [ ] support parallel parsing
- [ ] separate `safe` & `unsafe` feature
