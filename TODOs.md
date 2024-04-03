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

- [ ] make JsonValue able to index/get it's value when given a key
    - [x] index object
    - [x] index array
    - [ ] improve index object performance (currently O(n))
- [ ] allow parse JsonOutput into simple primitive value
    - [x] parse simple cases
    - [ ] number has pretty large range of variants, consider support them in future release (ex: 65_535, 2e16, 0x234, 2E, ...)
- [ ] allow parse ast into struct
- [ ] support type check on JsonOutput(idea: `fn is<T>(&self) -> bool`)

### Deserialize & Search

- [ ] Deserialize AST into Struct
- [ ] Able to query on AST

### Bug

- [x] `get_span` inside `parse_obj` auto increase `start` & `end` by 1???