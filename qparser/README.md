# qparser: Q parser crate

This crate provides parsing and evaluation of Q expressions using `chumsky`.

Features:
- Integer and floating-point literals
- Addition (`+`), subtraction (`-`), multiplication (`*`), and division (`/`)
- Correct operator precedence and associativity
- Parenthesized expressions

## Usage
```toml
[dependencies]
qparser = "0.1.0"
```

```rust
use qparser::{parse, eval_str};

let ast = parse("1+2*3").unwrap();
let result = eval_str("(1+2)*3").unwrap();
assert_eq!(result, "9");
```

See `src/ast.rs` for the AST definitions and `src/parser.rs` for the parser implementation.
