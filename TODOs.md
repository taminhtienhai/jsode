### Lexer

- [ ] prefer `*const u8`(raw_pointer) instead of `Iterator<Item = u8>`(iterator)
- [ ] replace `char` (utf32) iterator with `u8` (ascii) because it is overwhelming
    - JSON token doesn't escape ascii boundary, so why don't we just directly use ascii over utf32

### Parser