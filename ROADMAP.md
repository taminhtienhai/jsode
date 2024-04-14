# Project Roadmap

### Bugs

- [x] `get_span` inside `parse_obj` auto increase `start` & `end` by 1???

## Issues

- [x] convert `Option<Result<u8,JsonError>>` -> `Result<Option<u8>,JsonError>`
- [x] rename this crate
- [ ] write document
- [ ] bring some usecases/examples
- [ ] apply github CI/CD
- [ ] revamp error leverage derive macro
- [ ] test on real & large json file

## Road to 0.1

- [x] support prelude module
- [x] enhance derive macro
    - [x] support Option property
    - [x] support lifetime
    - [x] support phantom data
    - [x] support parse Array (Vec<T>, ~~&[T]~~, ..)
    - ~~[ ] support tuple struct~~ (impossible because tuple don't have keys, tuple layout also look more like a array)
- [x] prefer using HashMap to store key rather than Vec


## Road to 0.2

- [ ] `Serialize` macro
- [ ] enhance error message
    - [ ] Diagnostic struct (visualize location of error on input source)
- [ ] row & column tracking
- [ ] support query
- [ ] support Defer<'_, u8> property
    - postpone parsing value as late as possible
- [ ] accept file as argument (mmap)
- [ ] impl Deserialize on more type
    - [ ] `&[T]`
    - [ ] `HashMap<String, T>`

## Road to 0.3

- [ ] support `no_std`
- [ ] support compile-time validation macro
- [ ] support better number variants parsing (ex: 65_535, 2e16, 0x234, 2E, ...)

## Road to 1.0

- [ ] parallel parsing
- [ ] separate `safe` & `unsafe` feature
