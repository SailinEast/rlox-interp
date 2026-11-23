# rlox

Rust implementation of Lox from [*Crafting Interpreters*](https://craftinginterpreters.com/) (Part II, the jlox tree-walk interpreter).

Plan is to get the full interpreter working first, then go back and optimize once everything is implemented and tested.

## Running

```bash
# REPL
cargo run

# Run a script
cargo run -- path/to/script

# Tests
cargo test
```

## Progress

- [x] Ch. 4: Scanning - lexer borrows `&str`; handles single/two-char tokens, literals, identifiers, keywords via `match`, `//` line comments, `/* ... */` block comments with newline tracking, UTF-8 in strings/comments, and error reporting.
- [x] Ch. 5: Representing Code - `Expr` and `Literal` enums with `Display` formatting (skipped the Java codegen script and Visitor pattern).
- [x] Ch. 6: Parsing Expressions - binding-power Pratt parser instead of recursive descent for literals, grouping, unary, binary, comma (`,`), and right-associative ternary (`? :`) expressions, with `Result`-based syntax error reporting.
- [ ] Ch. 7: Evaluating Expressions
- [ ] Ch. 8: Statements and State
- [ ] Ch. 9: Control Flow
- [ ] Ch. 10: Functions
- [ ] Ch. 11: Resolving and Binding
- [ ] Ch. 12: Classes
- [ ] Ch. 13: Inheritance
- [ ] Optimization

## Rust notes

### Scanning (Ch. 4)
- Putting literal values right inside `TokenType` (`Number(f64)`, `String(String)`, `Identifier(String)`) avoids needing a separate wrapper field like in Java.
- `String` and `&str` are UTF-8 so you can't index them by char. Taking `&str` at API boundaries guarantees valid UTF-8, and since ASCII bytes (`< 0x80`) never show up inside multibyte UTF-8 sequences, looking ahead via `.as_bytes()` still keeps `&str` slicing safe for UTF-8 strings and comments.
- Borrowing `&'a str` in `Scanner<'a>` works because the scanner is short-lived, while keeping `Token` owned stops lifetimes from leaking into the AST, closures, and the REPL loop.
- Returning `std::process::ExitCode` from `main` instead of calling `process::exit`, taking `mut self` in `scan_tokens` to move out `Vec<Token>`, matching byte ranges (`b'0'..=b'9'`), and using `u8::is_ascii_digit()` / `u8::is_ascii_alphabetic()`.

### AST and Pratt parsing (Ch. 5-6)
- Recursive `enum Expr` + `match` replaces `jlox`'s `GenerateAst` script and Visitor classes.
- Recursive enum fields have infinite size at compile time, so nested expressions have to go behind `Box<Expr>`.
- You can't move an element out of the middle of a `Vec<Token>` (it would leave an uninitialized hole), so cloning tokens from `Parser` into `Expr` is expected here. Read-only helpers like `infix_bp(&TokenType)` should still borrow instead of taking owned `String`s.
- Returning `Result<Expr, ParseError<'a>>` borrowed from `&mut self` keeps `&mut self` locked across `?` and loop iterations. Making `ParseError` own its `Token` gets rid of that issue.
- `Result<Expr, ParseError>` + `?` replaces Java's `throw new ParseError()` unwinding, and `Option<(u8, u8)>` for binding powers avoids C-style sentinel constants.
- `?` isn't real stack unwinding like Java `throw` or Rust `panic!` - it's just syntactic sugar for a `match` and early `return Err(err.into())`, with normal local drops at each frame.
- Separating `prefix_bp` and `infix_bp` and using `(l_bp, r_bp)` pairs made adding left-associative `,` and right-associative `? :` take just a few lines.