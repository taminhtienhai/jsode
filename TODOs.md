### Lexer

- [x] prefer `*const u8`(raw_pointer) instead of `Iterator<Item = u8>`(iterator)
- [x] replace `char` (utf32) iterator with `u8` (ascii) because it is overwhelming
    - JSON token doesn't escape ascii boundary, so why don't we just directly use ascii over utf32

### Parser

- [x] parse token stream into JsonValue tree
- [ ] more helpful error message


### Indexer

- [x] make JsonValue able to index/get it's value when given a key
    - how to access `&str` source via JsonValue?
- [ ] allow parse ast into struct