# Project Roadmap

### Lexer

- [x] prefer `*const u8`(raw_pointer) instead of `Iterator<Item = u8>`(iterator)
- [x] replace `char` (utf32) iterator with `u8` (ascii) because it is overwhelming
    - JSON token doesn't escape ascii boundary, so why don't we just directly use ascii over utf32

### Parser

- [x] parse token stream into JsonValue tree
- [x] tracking '[]' and '{}' brackets 
- [ ] json's object key must be unique
- [ ] more helpful error message
- [ ] making parsing run parallel (hard)

### Indexer

- [x] ~~Change output from `JsonOutput` -> `JsonValue`~~
    - defer the parsing/copy as late as possible
- [ ] make JsonValue able to index/get it's value when given a key
    - [x] index object
    - [x] index array
    - [ ] improve index object performance (currently O(n))
- [ ] allow parse JsonOutput into simple primitive value
    - [x] parse simple cases
    - [ ] number has pretty large range of variants, consider support them in future release (ex: 65_535, 2e16, 0x234, 2E, ...)
- [ ] support type check on JsonOutput(idea: `fn is<T>(&self) -> bool`)

### Macro

- [ ] Deserialize derive macro
- [ ] Serialize derive macro

### Deserialize & Search

- [x] Deserialize AST into Struct
- [ ] Able to query on AST
    - for example `.query(".abilities[].moves")`

### Bug

- [x] `get_span` inside `parse_obj` auto increase `start` & `end` by 1???

## Issues

- [x] convert `Option<Result<u8,JsonError>>` -> `Result<Option<u8>,JsonError>`
- [x] rename this crate
- [ ] revamp error leverage derive macro
- [ ] test on real & large json file

## Road to 0.1

- [x] support prelude module
- [x] enhance derive macro
    - [x] support Option property
    - [x] support lifetime
    - [x] support phantom data
    - ~~[ ] support tuple struct~~ (impossible because tuple don't have keys)
    - [x] support parse Array (Vec<T>, ~~&[T]~~, ..)
    - [ ] support enum (optional)
- [x] prefer using HashMap to store key rather than Vec


## Road to 0.2

- [ ] enhance error message
- [ ] row/col tracking
- [ ] support query
- [ ] support Defer<'_, u8> property
    - postpone parsing value as late as possible
- [ ] accept file as argument (mmap) 

## Road to 0.3

- [ ] support `no_std`