# NOVA Compiler — Phase 0: Lexer + Parser

This is the Phase 0 implementation of the NOVA programming language compiler. It performs **lexical analysis** and **parsing** to produce an Abstract Syntax Tree (AST).

**Status:** Lexer and recursive-descent parser are complete. Type checking, unit resolution, and code generation are not yet implemented.

## Build

### Prerequisites

- Rust 1.70+ (install from https://rustup.rs/)
- Cargo (included with Rust)

### Compile

```bash
cd nova-compiler
cargo build --release
```

The binary will be at `target/release/nova` (or `nova.exe` on Windows).

### Run

```bash
# Parse a NOVA file
./target/release/nova hello_universe.nv

# Parse an inline code snippet
./target/release/nova -c "mission main() → Void { transmit(\"Hello\"); }"

# Show help
./target/release/nova --help
```

## What It Does

The compiler currently:

1. **Lexer** (`src/lexer.rs`)
   - Tokenizes NOVA source into a stream of tokens
   - Handles unit annotations: `3.0[m/s²]`
   - Recognizes all NOVA keywords and operators
   - Tracks source location (file, line, column) for error reporting

2. **Parser** (`src/parser.rs`)
   - Recursive descent parser with operator precedence
   - Builds a complete AST from tokens
   - Handles missions, constellations, models, pipelines, type annotations
   - Error recovery and reporting

3. **AST** (`src/ast.rs`)
   - Complete node types for NOVA constructs
   - Supports all language features from spec v0.2

## Example Programs

### hello_universe.nv

```nova
mission main() → Void {
  transmit("Hello, universe!");
}
```

Run:
```bash
cargo run --release -- examples/hello_universe.nv
```

Expected output:
```
✓ Lexer: 11 tokens
✓ Parser: 1 top-level items
  [0] mission main : None
Parse successful: examples/hello_universe.nv
```

### units_example.nv

```nova
mission delta_v(isp: Float, m_wet: Float, m_dry: Float) → Float {
  let g = 9.80665;
  let dv = isp * g * (m_wet / m_dry);
  return dv;
}
```

### model_example.nv

```nova
model StarClassifier {
  layer embedding(vocab=32000, dim=512)
  layer dense(256, activation=.relu)
  layer dense(128, activation=.relu)
  layer dense(7, activation=.softmax)
}
```

## Architecture

```
Input (NOVA source)
    ↓
Lexer (tokenization)
    ↓
Tokens
    ↓
Parser (recursive descent)
    ↓
AST
    ↓
[Type Checker - Phase 1]
    ↓
[LLVM Backend - Phase 2]
    ↓
Native Binary
```

## Running Tests

```bash
cd nova-compiler
cargo test
```

Unit tests are embedded in the lexer and parser modules.

## File Structure

```
nova-compiler/
├── Cargo.toml                 -- Project manifest
├── src/
│   ├── main.rs               -- CLI entry point
│   ├── lib.rs                -- Library root
│   ├── lexer.rs              -- Tokenizer
│   ├── parser.rs             -- Parser
│   ├── ast.rs                -- AST node definitions
│   └── error.rs              -- Error types
└── examples/
    ├── hello_universe.nv      -- Minimal example
    ├── units_example.nv       -- Unit annotations
    └── model_example.nv       -- Neural network model
```

## Next Steps (Phase 1)

The next phase will implement:

1. **Type Checker** — Hindley-Milner inference with units and tensor shapes
2. **Unit Resolver** — Parse `[m/s²]` into SI dimension vectors
3. **Error Messages** — Domain-aware errors for scientists
4. **Symbol Table** — Scope and binding tracking

The parser output (AST) feeds directly into the type checker.

## Known Limitations (Phase 0)

- ✗ No type checking
- ✗ No unit resolution — `[m/s²]` is stored as a string
- ✗ No code generation
- ✗ Pipeline expressions parsed but not semantically validated
- ✗ Limited error recovery
- ✗ No support for custom operators
- ✗ String interpolation parsed as plain strings (not yet split into parts)

## Design Notes

### Lexer

- Hand-written (not generated from a grammar)
- Tracks source locations on every token
- Handles UTF-8 including Unicode superscripts (²) and middle-dot (·)
- Comments: `-- to end of line`

### Parser

- Recursive descent, no external parsing library
- Full operator precedence: `|>` < `=>` < `==` < `+` < `*` < `@` < `^`
- Handles both `→` (Unicode) and `->` (ASCII)
- Named and positional arguments in function calls

### AST

- Complete coverage of spec v0.2 syntax
- Source location on every meaningful node
- No semantic validation at this stage

## Contributing

To add a new language feature:

1. Add token types to `lexer.rs::TokenKind`
2. Add lexing logic to `Lexer::lex_one()` 
3. Define AST nodes in `ast.rs`
4. Add parsing logic to `parser.rs`
5. Add an example to `examples/`

## License

MIT, same as NOVA.

---

**NOVA Phase 0 Compiler**  
*A foundation for the next generation of scientific computing languages.*
