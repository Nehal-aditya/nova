# NOVA Phase 1: Type System & Semantic Analysis

**Status:** 📋 PLANNING

## Overview

Phase 1 transforms NOVA's AST into a semantically-validated, type-checked program ready for code generation. This requires building type checking, unified type inference, symbol table management, and dimensional analysis.

## Key Phases to Block Type Checkers Development

Before Phase 1 can be fully designed, the following **design questions** must be answered:

### 🔴 Critical Decisions Required

**Memory Model**
- Should NOVA use garbage collection, reference counting (ARC), regions, or all three?
- Impact: Function signatures, ownership rules, lifetime annotations
- Decision needed from: Project owner

**Error Handling Strategy**
- Should NOVA use `Result<T, E>` types, exceptions, or both?
- Should errors be built-in or library-defined?
- Impact: Type system complexity, error propagation

**Model Weight Mutability**
- During `autodiff` blocks, can model weights be modified?
- Should parameter updates be automatic or explicit?
- Impact: Type safety, API surface

**Dimensionless Type Representation**
- How should dimensionless values be represented? `Float` or `Float[dimensionless]` or `Float[1]`?
- Impact: Unit resolution complexity

**Autodiff Architecture**
- Static tape (like Tinygrad) or dynamic tape (like PyTorch)?
- Impact: AST representation, compilation strategy

**Parallel Safety**
- Are `parallel mission` inputs guaranteed immutable, or enforced by type system?
- Impact: Borrow checker rules, function type signatures

## Phase 1 Work Breakdown

### Stage 1A: Foundation (Independent of blocked decisions)

**Lexical Scope & Symbol Tables**
- Arena allocator for type environment
- Scope stack implementation
- Symbol resolution (function, struct, enum, model lookups)
- Duplicate definition detection

**Type Definitions Storage**
- Struct field resolution
- Enum variant resolution  
- Model layer definition lookup
- Type alias resolution

**Basic Type Inference**
- Integer literal inference: `5` → `Int`
- Float literal inference: `3.14` → `Float`
- String literal inference: `"hello"` → `String`
- Boolean literal inference: `true` → `Bool`

**Type Checking for Primitives**
- Binary op type rules: `Int + Int → Int`, `Float + Float → Float`
- Type mismatch errors
- Function call argument count checking
- Return type validation

### Stage 1B: Unit System (Can proceed immediately)

**Unit Expression Normalization**
- Parse unit expressions from `Float[m/s]` type annotations
- Representing: `kg`, `m`, `s`, `m/s`, `kg*m/s^2`, etc.
- Unit algebra: multiplication, division, exponentiation
- Unit equality checking (e.g., `m/s == m*s^-1`)

**Dimensional Analysis**
- Check that operations respect units:
  - `Float[m] + Float[m] → Float[m]` ✓
  - `Float[m] + Float[s]` → Type Error ✗
  - `Float[m] * Float[s] → Float[m*s]` ✓
- Custom unit substitution (e.g., `lightyear = 9.46e15 m`)

**Unit Inference**
- Infer output units from operations
- Propagate units through function calls
- Validate unit consistency across assignments

### Stage 1C: Complex Types (Blocked by decisions)

**Function Types**
- Represent mission declarations as types
- Handle parameter types and return types
- Generic function handling (if in design)
- Closures/lambdas (if in design)

**Model Type System**
- Represent model type as: layers + parameter types
- Track layer output shapes through stack
- Validate tensor dimensions through operations

**Array Generics**
- `Array[T]` → resolve `T` recursively
- Handle nested arrays: `Array[Array[Float]]`

**Borrow Semantics** (if memory model uses borrowing)
- Track mutable vs immutable references
- Check borrowing rules at assignment/call sites
- Prevent use-after-move

### Stage 1D: Control Flow Analysis

**Reachability Analysis**
- Warn on unreachable code after `return`/`break`
- Error if function has reachable code path without`return`

**Exhaustiveness Checking**
- Ensure `if/else` chains cover all paths returning values
- Pattern exhaustiveness for `match` (if added in Phase 2)

### Stage 1E: Semantic Validation

**Function Validation**
- Check that transmit arguments are valid
- Validate autodiff block contents (Phase 2+)
- Check pipeline operator compatibility

**Import/Constellation Validation**
- Resolve `absorb constellation.mission` references
- Track exported items
- Prevent circular dependencies

## Implementation Order

**Priority 1 (Foundation):**
1. Symbol table infrastructure
2. Basic type inference (literals)
3. Type checking for primitives
4. Error reporting framework

**Priority 2 (Units):**
5. Unit expression parsing from types
6. Unit normalization & algebra
7. Dimensional analysis for operations
8. Custom unit substitution

**Priority 3 (Functions & Control Flow):**
9. Function type checking
10. Return type validation
11. Reachability analysis

**Priority 4 (Advanced - after decisions):**
12. Generic types
13. Borrow semantics (if needed)
14. Model layer tracking
15. Complex pipeline validation

## Estimated Effort

- **Lines of code:** 3,000 - 5,000
- **Complexity:** High (type systems are complex)
- **Duration:** 2-4 weeks (depending on blocked decisions)

## Design Decisions Checklist

- [ ] Memory model (GC / ARC / regions)
- [ ] Error handling (Result / exceptions / both)
- [ ] Model weight mutability during autodiff
- [ ] Dimensionless type representation
- [ ] Autodiff static vs dynamic tape
- [ ] Parallel function safety constraints

**Once these are decided, Phase 1 planning can be finalized.**

## Deliverables

1. **Symbol resolution engine** (`symbol_table.rs`)
2. **Type inference engine** (`type_inference.rs`)
3. **Unit resolution & dimensional analysis** (`units.rs`)
4. **Control flow analyzer** (`flow_analysis.rs`)
5. **Comprehensive test suite** (200+ tests)
6. **Documentation** (design, API, examples)
7. **Error reporting** (friendly compiler messages)

---

**Awaiting:**
- Design decisions from project owner
- Blocking resolved in design discussion
- Ready to begin once decisions are made

**Note:** Phase 0 (lexer & parser) is complete and production-ready. Phase 1 can begin immediately.
