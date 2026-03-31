# NOVA — Implementation Plan

## Overview

This document is the engineering roadmap for building the NOVA programming language: a compiled, statically typed, general-purpose language designed for the next generation of AI, data science, astronomy, cosmology, and rocket science.

NOVA's syntax is finalised at v0.2. The following decisions are locked:
- `absorb` for imports (replaces `from ... import`)
- `pipeline [...]` for ordered transform chains
- `→` as the return type arrow (`->` as ASCII fallback)
- `transmit(...)` for output
- `{r:.4}` format specifiers in string interpolation
- `cosmos.*` namespace for scientific constellations
- `nova.*` namespace for general-purpose constellations
- `.nv` as the file extension

---

## Phase 0 — Foundation (Weeks 1–4)

**Goal:** Working lexer, parser, and AST. No execution yet.

### 0.1 Formal grammar (BNF/EBNF)

Write the complete grammar covering:
- Literals: integers, floats, strings, unit-annotated floats (`3.0[m/s²]`), booleans
- Bindings: `let`, `var`, destructuring `let (a, b) = ...`
- Missions: `mission`, `parallel mission`, named and positional arguments, `→` return type
- Constellations: `constellation`, `absorb`, `export`
- Types: `Float[unit]`, `Tensor[T, shape]`, `Array[T]`, `Option[T]`, `Result[T,E]`, `Wave`, `Dataset`
- Expressions: pipe `|>`, lambda `s → ...`, `match`, `pipeline [...]`, `autodiff`, `gradient wrt`
- Model blocks: `model`, `layer`, `repeat`, named layer arguments
- Control flow: `for`, `while`, `match`, `if`/`else`, `break`, `return`
- Declarations: `struct`, `enum`, `unit`

Resolve ambiguities:
- `3.0[m]` vs `array[idx]` — unit annotation vs index (unit annotation always follows a float literal; index follows an identifier or closing bracket)
- `^` power vs `^` XOR — NOVA has no bitwise XOR; `^` is always power
- `@` matmul vs decorator — NOVA uses `@` only for matmul; annotations use `#[...]`
- `→` vs `->` — both accepted, formatter normalises to `→`

### 0.2 Lexer (Rust)

Token types:
- Keywords: `mission`, `parallel`, `constellation`, `absorb`, `let`, `var`, `model`, `layer`, `autodiff`, `gradient`, `wrt`, `for`, `in`, `match`, `return`, `export`, `from`, `struct`, `enum`, `unit`, `wave`, `on`, `device`, `repeat`, `transmit`, `if`, `else`, `while`, `break`, `pipeline`
- Literals: integer, float, unit-annotated float, string (with interpolation spans), bool
- Operators: `→`, `->`, `|>`, `@`, `^`, `=>`, `..`, `?`, `==`, `!=`, `<`, `>`, `<=`, `>=`, `+`, `-`, `*`, `/`, `=`
- Punctuation: `{`, `}`, `[`, `]`, `(`, `)`, `,`, `:`, `.`, `;`
- Identifiers, comments (`--` to end of line)
- Source location (file, line, column) on every token — required for error messages

Unit suffix lexing: parse `[m/s²]`, `[kg·m]`, `[N·m²/kg²]` — handle Unicode superscripts (`²`, `³`), middle-dot (`·`), slash, and negative exponents.

### 0.3 Parser (recursive descent)

Hand-written recursive descent parser. Advantages: full control over error messages, natural source span tracking, no grammar/parser mismatch.

Operator precedence (low to high):
1. `|>` (pipe)
2. `=>` (lambda/match arm)
3. `==`, `!=`, `<`, `>`, `<=`, `>=`
4. `+`, `-`
5. `*`, `/`
6. `@` (matmul)
7. `^` (power)
8. Unary: `-`, `?`
9. `.` (field access), `(...)` (call), `[...]` (index)

Produce a typed AST with source spans on every node.

### 0.4 AST node types

```
MissionDecl    { name, params, return_type, body, parallel: bool }
ConstellationDecl { name, items }
AbsorbDecl     { path, names }
ModelDecl      { name, layers }
LayerDecl      { kind, args, repeat: Option<u32>, body: Option<Vec<LayerDecl>> }
LetBind        { name, type_ann, value, mutable: bool }
UnitLiteral    { value: f64, unit: UnitExpr }
TensorLit      { elements, shape }
PipeExpr       { left, right }
Pipeline       { transforms: Vec<Expr> }
LambdaExpr     { param, body }
MatchExpr      { subject, arms: Vec<MatchArm> }
AutodiffBlock  { target, body }
GradientExpr   { expr, wrt: Vec<Ident> }
TransmitExpr   { template: InterpolatedString }
ForExpr        { var, iter, body }
StructDecl     { name, fields: Vec<(Ident, TypeExpr)> }
EnumDecl       { name, variants }
UnitDecl       { name, definition: UnitExpr }
```

**Milestone:** Parse all example programs from the spec and both demo programs (`hello_universe.nv`, `stellar_main_sequence.nv`) without errors.

---

## Phase 1 — Type System (Weeks 5–10)

**Goal:** Full type inference and unit checking. Every mismatch caught at compile time.

### 1.1 Core type checker

Hindley-Milner Algorithm W, extended with:
- Unit dimension vectors (7-dimensional SI basis)
- Tensor shape types
- Row polymorphism for struct field access (enables `stars.mass`, `s.mass_solar`, etc.)

### 1.2 Unit resolver (dedicated pass, runs between parser and type checker)

1. Parse unit expressions into 7-dimensional SI vectors: `[m/s²]` → `(L=1, T=-2)`
2. Build unit alias table: `N = kg·m/s²`, `J = N·m`, `Pa = N/m²`, `W = J/s`, `AU = 1.496e11·m`, `ly = 9.461e15·m`, `pc = 3.086e16·m`, `M☉ = 1.989e30·kg`, `L☉ = 3.828e26·W`, `eV = 1.602e-19·J`, `atm = 101325·Pa`
3. User-defined `unit` declarations are added to the alias table
4. Auto-convert compatible units on addition/subtraction: `1.0[kg] + 500.0[g]` → `1.5[kg]`
5. Reject incompatible dimensions with a domain-aware error message naming the SI dimensions
6. Track derived unit recognition: `kg·m/s²` is reported as `N`, not the expanded form

### 1.3 Tensor shape types

- `Tensor[Float, 3x4]` — shape is part of the type
- `@` (matmul): `(A,B) @ (B,C) → (A,C)` — inner dimensions checked at compile time
- `reshape`, `transpose`, `concat`, `split` — output shapes statically known where possible; runtime check emitted otherwise
- Shape inference for `layer` chains inside `model` blocks

### 1.4 Model type checker

- `model` blocks produce a named struct type with a `.forward()` mission
- `layer` declarations are type-checked for compatible input/output shapes
- `autodiff` blocks verify the differentiand is a scalar `Float` (or `Float[dimensionless]`)
- `gradient(expr, wrt vars)` is typed: returns a tuple of the same types as `vars`

### 1.5 Pipeline type checking

Each transform in `pipeline [...]` is type-checked against the output of the previous:
- `filter(pred)` — `Array[T] → Array[T]`, pred must be `T → Bool`
- `map(f)` — `Array[T] → Array[U]` where `f: T → U`
- `drop_outliers(sigma:)` — `Array[{mass: Float, lum: Float, ...}] → Array[...]` (requires numeric fields)
- `sort_by(field)` — field must exist and be orderable

### 1.6 Error messages

Domain-aware, not generic:
- Unit mismatch: "cannot add `Float[kg]` and `Float[m]` — dimension [M] vs [L] — did you mean `as [kg]`?"
- Shape mismatch: "matmul `(3,4) @ (5,6)` — inner dimensions 4 and 5 do not match"
- Missing field: "type `Star` has no field `luminosity_solar` — did you mean `luminosity_solar`?" (edit distance suggestion)
- Parallel safety: "mission `f` mutates its input `data` — `parallel mission` inputs must be immutable"

**Milestone:** Type-check all example programs. Reject all intentionally broken programs with clear errors. `hello_universe.nv` compiles in under 50ms.

---

## Phase 2 — IR and Code Generation (Weeks 11–18)

**Goal:** Compile NOVA to native binaries via LLVM.

### 2.1 IR design

- Typed SSA form, higher-level than LLVM IR
- Units erased at IR level (compile-time only — zero runtime cost)
- Tensor ops represented as typed IR instructions: `matmul`, `reduce`, `elementwise`, `broadcast`
- Parallel missions annotated with `#parallel` — lowered to work-stealing in Phase 2.3

### 2.2 LLVM backend

- Use `inkwell` (Rust LLVM bindings)
- Lower missions to LLVM functions
- Lower `Float[unit]` to LLVM `double` (unit erased)
- Lower `Tensor` to heap-allocated strided arrays (row-major)
- Dispatch tensor matmul `@` to `cblas_dgemm` on CPU
- Optimisation pipeline: inlining, loop vectorisation, dead code elimination

### 2.3 Parallel lowering

- `parallel mission` → fork-join at IR level using Rayon (Rust work-stealing)
- `pipeline [...]` stages inside parallel context → fused where data-independent, sequenced where dependent
- Dependency analysis: if stage N reads only the output of stage N-1 and no shared state, stages can be pipelined

### 2.4 Autodiff lowering

- Reverse-mode (backprop) for scalar losses — covers all standard training loops
- Forward-mode for vector-to-scalar functions — covers Jacobian-vector products
- `autodiff(loss) { ... }` → append gradient computation IR after forward pass
- `gradient(expr, wrt vars)` → forward-mode trace
- Computation graph stored as IR metadata — no runtime graph object

### 2.5 Minimum standard library for Phase 2

Built-in missions available without `absorb`:
- `transmit(str)` — stdout
- `sqrt`, `ln`, `exp`, `log10`, `sin`, `cos`, `tan`, `abs`, `pi`, `e`
- Physical constants: `G`, `c`, `h`, `k_B`, `N_A`, `sigma_SB`
- `Array`: `map`, `filter`, `fold`, `zip`, `sort_by`, `len`, `min`, `max`, `sum`, `mean`, `std`, `median`
- `Tensor`: `@`, `softmax`, `relu`, `gelu`, `sigmoid`, `norm`, `reshape`, `transpose`, `einsum`, `argmax`, `concat`, `split`
- `Dataset`: `batches`, `split`, `shuffle`
- `panic(msg)` — unrecoverable error

**Milestone:** Compile and run `hello_universe.nv` and the rocket delta-v example end-to-end. Output matches expected values.

---

## Phase 3 — Toolchain (Weeks 19–26)

**Goal:** A complete, usable development experience.

### 3.1 Package manager — Nebula

- `nova.toml` manifest: name, version, authors, dependencies, constellation paths
- `nebula new <name>` — scaffold a new project
- `nebula add <constellation>` — add a dependency from the Nebula Registry
- `nebula build` — compile
- `nebula run` — compile and execute
- `nebula test` — run all `test mission` blocks
- Lock file: `nova.lock` (content-addressed, reproducible builds)

### 3.2 Test framework (built into the language)

```nova
test mission delta_v_earth_escape() {
  let dv = delta_v(isp=311.0[s], m_wet=549054.0[kg], m_dry=25600.0[kg])
  assert_approx(dv, 9740.0[m/s], tolerance=10.0[m/s])
}

test mission unit_mismatch_caught() {
  assert_raises(TypeError, { 1.0[kg] + 1.0[m] })
}
```

- `assert_eq`, `assert_approx`, `assert_unit`, `assert_raises`, `assert_shape`
- Unit-aware: `assert_approx(result, 9.8[m/s²], tolerance=0.01[m/s²])`

### 3.3 Language server — nova-ls

LSP implementation (Rust, tower-lsp):
- Hover: show inferred type with full unit annotation, e.g. `isp : Float[s]`
- Go-to-definition: missions, constellations, struct fields
- Unit error highlighting: red underline on unit mismatches, with hover explanation
- Mission signature help: show parameter names, types, and units as you type a call
- Tensor shape tooltips: hover over a tensor variable to see its shape
- Auto-import: suggest `absorb` lines for unrecognised names

### 3.4 REPL — nova-repl

```
nova> let v = 11200.0[m/s]
v : Float[m/s] = 11200.0

nova> v as [km/s]
: Float[km/s] = 11.2

nova> :unit v
dimension: [L T⁻¹]  SI base: m/s  value: 11200.0

nova> :shape my_tensor
Tensor[Float, 3x4x128]
```

- `:type expr` — show inferred type
- `:unit expr` — show SI dimension vector
- `:shape expr` — show tensor shape
- `:absorb cosmos.stats` — absorb a constellation for the session

### 3.5 Formatter — nova-fmt

Opinionated, non-configurable (like gofmt):
- Normalise `->` to `→`
- Align `=` signs in `let` blocks
- Normalise unit spacing: `kg·m/s²` not `kg * m / s^2`
- Consistent pipeline indentation: each step on its own line, indented 4 spaces
- No trailing whitespace, Unix line endings

**Milestone:** Full toolchain working. A NOVA project can be created, written, formatted, tested, and run with `nebula` commands. The language server provides type-on-hover and unit error highlighting in VS Code.

---

## Phase 4 — Standard Constellations (Weeks 27–40)

**Goal:** Implement `cosmos.*` and `nova.*` in full.

### Priority order for `cosmos.*`

1. `cosmos.data` — CSV, Parquet, FITS, Arrow (unblocks all real programs)
2. `cosmos.stats` — pearson, spearman, linear_fit, polyfit, distributions (unblocks the stellar demo)
3. `cosmos.ml` — tensor ops, layers, optimisers, losses (unblocks AI programs)
4. `cosmos.plot` — scatter, histogram, regression_line, H-R diagram (unblocks visualisation)
5. `cosmos.astro` — FITS catalogue, coordinate transforms, magnitude, redshift
6. `cosmos.orbital` — Kepler, Tsiolkovsky, trajectory integration
7. `cosmos.spectral` — blackbody, emission lines, Doppler
8. `cosmos.signal` — FFT, wavelets, filters
9. `cosmos.thermo` — heat transfer, ideal gas, entropy
10. `cosmos.quantum` — Schrödinger, operators, quantum gates
11. `cosmos.chem` — periodic table, stoichiometry
12. `cosmos.geo` — geodesy, great-circle

### Priority order for `nova.*`

1. `nova.fs` — file system (needed for `load_csv`, `write_parquet`)
2. `nova.cli` — argument parsing (needed for useful command-line programs)
3. `nova.fmt` — JSON, YAML, TOML (needed for config files)
4. `nova.net` — HTTP server/client (general-purpose web use)
5. `nova.db` — SQL, key-value
6. `nova.concurrent` — channels, atomics
7. `nova.crypto` — hashing, signing
8. `nova.test` — property testing, benchmarks

**Milestone:** The stellar main sequence demo runs end-to-end with real FITS data. The Hubble constant example produces a correct estimate. A transformer model trains on a toy dataset.

---

## Phase 5 — GPU and Distributed (Weeks 41–52)

**Goal:** Scale to multi-GPU training and distributed scientific pipelines.

### 5.1 GPU backend

- CUDA target for NVIDIA (emit PTX via LLVM NVPTX backend)
- Metal target for Apple Silicon (emit MSL via LLVM Metal backend)
- Tensor ops dispatch to GPU when `on device(gpu)` block is present or when a GPU is available and the tensor exceeds a size threshold
- `on device(gpu) { ... }` block for explicit placement

```nova
on device(gpu) {
  let logits = net.forward(batch.x)   -- runs on GPU
  let loss   = cross_entropy(logits, batch.y)
  autodiff(loss) { net.update(adam, lr=3e-4) }
}
```

### 5.2 Distributed missions

- `distributed mission` keyword — distributes across machines via MPI or a custom NOVA scheduler
- `shard` annotation on `Dataset` — automatic data sharding across workers
- Gradient aggregation (all-reduce) across workers for distributed training

### 5.3 Mixed precision

- `Float16`, `BFloat16` types
- Automatic loss scaling for `Float16` training
- Accumulation always in `Float32` unless explicitly overridden

### 5.4 Jupyter kernel — nova-kernel

- Implement the Jupyter messaging protocol
- Each cell evaluates in a persistent REPL session
- Rich output: `cosmos.plot` missions emit inline SVG/PNG in the notebook
- Unit and type annotations shown below each cell output

---

## Open questions

See `suggestionsandquestions.md` for the complete list of unresolved design decisions.

The following are blockers that must be decided before Phase 1:

| Decision | Why it blocks Phase 1 |
|---|---|
| Memory model (GC vs ARC vs regions) | Shapes the type checker and IR |
| Mutability of model weights | Must be precise before type checker handles `autodiff` |
| Error handling (`Result` vs exceptions) | Changes the AST and type rules |
| Dimensionless type representation | Required by unit resolver design |
| Autodiff strategy (static vs dynamic graph) | Changes the IR |
| Parallel safety model (immutable inputs vs other) | Changes type rules for `parallel mission` |

---

## Success criteria

- `hello_universe.nv` compiles in under 100ms on a modern laptop
- `stellar_main_sequence.nv` runs correctly on a real FITS catalogue
- `1.0[kg] + 1.0[m]` produces a clear compile error naming the SI dimensions
- A 24-layer transformer model trains on CPU in a `parallel mission` with no explicit thread management
- `nebula build` on a cold cache completes in under 5 seconds for a 1000-line project
- The language server shows full unit annotations on hover in VS Code and Neovim
- The Hubble constant example produces H₀ in `Float[km/s/Mpc]` — the unit guaranteed by the type system
