# CONTRIBUTING to NOVA Compiler

Welcome to the NOVA compiler project! This guide explains how to set up the development environment, build the compiler, and contribute code.

## Quick Start

### Prerequisites
- Rust 1.70+ ([Install](https://rustup.rs/))
- Cargo (comes with Rust)
- Git

### Setup (5 minutes)

```bash
# Clone the repository
git clone https://github.com/DeepcometAI/nova-ai-lang
cd nova-ai-lang

# Enter compiler directory
cd nova-compiler

# Build the compiler
cargo build --release

# Run tests
cargo test

# Try an example
cargo run --release -- examples/hello_universe.nv
```

## Project Structure

```
nova-compiler/
├── src/
│   ├── main.rs           Entry point, CLI handling
│   ├── lib.rs            Public API exports
│   ├── lexer.rs          Tokenization
│   ├── parser.rs         AST construction
│   ├── ast.rs            AST type definitions
│   ├── error.rs          Error types & reporting
│   └── (future)          Type checker, codegen, etc.
├── tests/
│   └── parser_tests.rs   331 integration tests
├── examples/
│   ├── hello_universe.nv
│   ├── model_example.nv
│   ├── constellation_example.nv
│   ├── stellar_analysis.nv
│   ├── units_example.nv
│   └── tests.nv
├── Cargo.toml
└── README.md
```

## Development Workflow

### 1. Running Tests

```bash
# Run all tests
cargo test

# Run specific test
cargo test test_simple_mission

# Run with output
cargo test -- --nocapture

# Quick test run (without optimization)
cargo test --lib
```

### 2. Building

```bash
# Debug build (fast compile, slow run)
cargo build

# Release build (slow compile, fast run)
cargo build --release

# Check for errors without compiling
cargo check
```

### 3. Linting & Formatting

```bash
# Check code style
cargo clippy

# Automatically format code
cargo fmt

# Check formatting
cargo fmt -- --check
```

### 4. Creating Examples

New `.nv` files in `examples/` are automatically parsed during testing. To test your example:

```bash
cargo run --release -- examples/my_example.nv
```

The compiler will either:
- Print "Parse successful" if the grammar is valid
- Print an error if there's a syntax error

## Current Phases

### ✅ Phase 0: Lexer & Parser
- **Status:** Complete
- **Loc:** ~3,700 lines
- **Tests:** 31 passing
- **What works:** Tokenization and AST construction
- **What's not done:** Type checking, code generation

**Key entry points:**
- [`Lexer::new()`](nova-compiler/src/lexer.rs#L1) - Create lexer from source
- [`Lexer::tokenize()`](nova-compiler/src/lexer.rs#L100) - Produce tokens
- [`Parser::new()`](nova-compiler/src/parser.rs#L1) - Create parser from tokens
- [`Parser::parse_program()`](nova-compiler/src/parser.rs#L60) - Produce AST

### 📋 Phase 1: Type System (Planning)
- **Status:** Blocked on design decisions (see [`DESIGN_DECISIONS.md`](../DESIGN_DECISIONS.md))
- **Est. Loc:** 3,000-5,000 lines
- **Milestones:**
  1. Symbol table & scope resolution
  2. Type inference for literals
  3. Type checking for primitives
  4. Unit system & dimensional analysis
  5. Function validation
  6. Control flow analysis

**Cannot start until:**
- Memory model decided (GC vs ARC vs borrow checking)
- Error handling strategy finalized
- Model weight mutability specified

### 🔮 Phase 2+: Code Generation, Runtime, Standard Library
- Deferred until Phase 1 complete

## Code Style Guide

### Naming Conventions

```rust
// Functions and variables: snake_case
fn parse_mission_def(parser: &mut Parser) -> Result<MissionDecl>

// Types and structs: PascalCase
pub struct MissionDecl { ... }
pub enum TokenKind { ... }

// Constants: UPPER_SNAKE_CASE
const MAX_IDENTIFIER_LENGTH: usize = 256;
```

### Error Handling

```rust
// Use the Error::parse helper for parser errors
Err(Error::parse(
    "Expected identifier, got keyword".to_string(),
    location
))

// Include context for debugging
Err(Error::lex(
    format!("Invalid character '{}' in unit expression", ch),
    loc
))
```

### Comments

```rust
// Single line comments for implementation details
// Start with lowercase for line-ends sentences

// Multi-line comments explain WHY, not WHAT
// The code shows WHAT it does.
// Good comment explains the design decision behind it.

/// Documentation comments for public APIs
/// Use `///` for functions, types, and modules
pub fn important_function() { ... }
```

### Testing

```rust
// Test module structure
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_feature_name() {
        // Arrange
        let input = "...";
        
        // Act
        let result = parse(input);
        
        // Assert
        assert_eq!(result.items.len(), 1);
    }
}
```

## Adding Features

### Scenario: Adding a New Token Type

1. **Add to `TokenKind` enum** in `lexer.rs`
   ```rust
   pub enum TokenKind {
       // ...
       Transmit,        // NEW
   }
   ```

2. **Update lexer** to recognize it
   ```rust
   "transmit" => TokenKind::Transmit,
   ```

3. **Update parser** to handle it
   ```rust
   TokenKind::Transmit => {
       self.advance();
       // Handle transmit statement
   }
   ```

4. **Add test** in `tests/parser_tests.rs`
   ```rust
   #[test]
   fn test_transmit_statement() {
       let code = r#"transmit("hello")"#;
       // ...
   }
   ```

5. **Run tests** to ensure nothing broke
   ```bash
   cargo test
   ```

### Scenario: Adding a New Statement Type

1. **Add variant to `Statement` enum** in `ast.rs`
   ```rust
   pub enum Statement {
       // ...
       MyStatement { /* fields */ }
   }
   ```

2. **Update parser** in `parser.rs`
   ```rust
   fn parse_statement(&mut self) -> Result<Statement> {
       match &self.current().kind {
           // ...
           TokenKind::MyKeyword => self.parse_my_statement(),
       }
   }
   ```

3. **Implement parser method**
   ```rust
   fn parse_my_statement(&mut self) -> Result<Statement> {
       // Implementation
       Ok(Statement::MyStatement { ... })
   }
   ```

4. **Write tests**
5. **Update documentation**

## Testing Guidelines

### Test Categories

**Unit Tests** - Test single functions
```rust
#[test]
fn lexer_recognizes_keyword() {
    let mut lexer = Lexer::new("mission", 0);
    let tokens = lexer.tokenize().unwrap();
    assert_eq!(tokens[0].kind, TokenKind::Mission);
}
```

**Integration Tests** - Test parser end-to-end
```rust
#[test]
fn parser_accepts_valid_program() {
    let code = "mission main() → Void { }";
    let mut lexer = Lexer::new(code, 0);
    let tokens = lexer.tokenize().unwrap();
    let mut parser = Parser::new(tokens);
    let program = parser.parse_program().unwrap();
    assert_eq!(program.items.len(), 1);
}
```

**Example Tests** - Parse `.nv` examples
```bash
# Any file in examples/ that parses without error passes
cargo run --release -- examples/my_example.nv
```

### Coverage Goals

- Phase 0 (Lexer & Parser): 100% of grammar coverage
- Phase 1 (Type Checker): 95%+ test coverage
- Phase 2+ (Codegen): 90%+ test coverage

## Common Tasks

### Run a single test repeatedly while debugging
```bash
cargo test test_name -- --nocapture --test-threads=1
```

### View detailed error messages
```bash
RUST_BACKTRACE=full cargo test
```

### Build documentation
```bash
cargo doc --open
```

### Profile compiler performance
```bash
time cargo build --release
cargo build --release --timings
```

## Debugging

### Print debug info in lexer/parser
```rust
eprintln!("DEBUG: token = {:?}", token);
```

### Check token stream
```bash
cargo run --release -- examples/hello_universe.nv  # Shows token count
```

### Step through parser manually
Edit a test, add debug prints, run `cargo test -- --nocapture`.

## Performance Considerations

- **Lexer:** O(n) where n = input length
- **Parser:** O(n log n) typical (recursive descent with lookahead)
- **AST memory:** ~100 bytes per AST node
- **Target:** Parse 10,000 LOC in <100ms

## Compatibility

- **Rust minimum:** 1.70
- **Platforms:** Linux, macOS, Windows
- **Compiler:** Only `rustc` and `cargo` required

## Documentation

### Essential Documentation

- `README.md` - Project overview
- `PHASE_0_COMPLETE.md` - Phase 0 details
- `PHASE_1_ROADMAP.md` - Next steps
- `DESIGN_DECISIONS.md` - Design rationale
- Code comments - Explain WHY

### Building API Docs

```bash
cargo doc --document-private-items --open
```

## Sending a Pull Request

1. **Fork** the repository
2. **Create a branch** for your feature
   ```bash
   git checkout -b feat/my-feature-name
   ```
3. **Make changes** and write tests
4. **Run full test suite**
   ```bash
   cargo test && cargo clippy && cargo fmt
   ```
5. **Commit** with descriptive message
   ```bash
   git commit -m "feat: add support for while loops"
   ```
6. **Push** your branch
7. **Create Pull Request** with description of changes

### PR Checklist

- [ ] Tests pass (`cargo test`)
- [ ] Code is formatted (`cargo fmt`)
- [ ] No clippy warnings (`cargo clippy`)
- [ ] Documentation updated
- [ ] PR description explains the change
- [ ] Examples provided if applicable

## Getting Help

### Resources

- [Rust Book](https://doc.rust-lang.org/book/) - Learn Rust
- [Rust by Example](https://doc.rust-lang.org/rust-by-example/) - Examples
- [Compiler Book](https://craftinginterpreters.com/) - Compiler design
- [NOVA Spec](../description.md) - Language specification

### Asking Questions

- Open an issue with `[question]` tag
- Include minimal reproducible example
- Show what you've tried

## Maintainer Guidelines

Only for core maintainers:
- Code review before merge
- Ensure tests pass on CI/CD
- Update version in `Cargo.toml` for releases
- Tag releases in git
- Update changelog

---

**Keep compiler code clean, tested, and documented!**

**Questions?** Open an issue or start a discussion.
