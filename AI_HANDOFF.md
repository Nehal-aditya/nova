# NOVA AI Agent Handoff (repo-local)

This document is a **single-source handoff** for continuing work on `nova-ai-lang` with another AI agent (or a future you) without re-triage.

## Project identity (what you’re building)

**NOVA** is a compiled, statically typed, general-purpose language designed for **AI + data science + astronomy + rocket science**. The distinguishing feature is **units-as-types** + **tensor shapes**, plus mission-themed syntax (`mission`, `absorb`, `pipeline`, `transmit`).

- **Languages used intentionally**:
  - **C**: lexer + parser
  - **Rust**: unit resolver + typechecker + semantic analysis + LLVM codegen
  - **Java**: interface validator (planned)
  - **Python**: stdlib prototype + REPL prototype (planned)

## What exists today (high-confidence state)

### Compiler

- **C lexer**: implemented under `compiler/lexer/`
- **C parser**: implemented under `compiler/parser/`
- **Rust unit resolver**: implemented under `compiler/unit_resolver/`
- **Rust typechecker**: implemented under `compiler/typechecker/`
- **Rust semantic**: implemented as core scope + borrow/lifetime scaffolding with unit tests (`compiler/semantic/`)
- **Rust autodiff**: a working computation-graph + backprop engine with unit tests (`compiler/autodiff/`)
- **Rust tensor lowering**: shape/broadcast + matmul-shape/strategy utilities with unit tests (`compiler/tensor_lowering/`)
- **Rust LLVM IR emission**: textual IR emitter exists under `compiler/codegen/` and has been improved to output structurally valid textual LLVM IR for core operations (still not a full end-to-end compiler pipeline wired from the C AST).
- **Java interface validator**: implemented as a Maven module with unit tests (`compiler/interface_validator/`)

### Standard library (Python-backed prototype)

Stdlib lives in `stdlib/` and is importable from Python by adding `stdlib/` to `sys.path`.

- **Scientific constellations (`cosmos.*`)**:
  - `cosmos.data`: `Wave` + CSV/FITS/Parquet/Arrow readers; plus `read_hdf5` (requires `h5py`), `read_netcdf` (requires `netCDF4`); and pipeline helper functions.
  - `cosmos.stats`: pearson/spearman/linear_fit/polyfit + basic summary stats
  - `cosmos.ml`: basic activations + losses + simple optimizers + init helpers
  - `cosmos.plot`: matplotlib plotting primitives (`scatter`, `histogram`, `regression_line`, etc.)
  - `cosmos.astro`: small astro helpers + `read_fits()` delegating to `cosmos.data.read_fits`
  - `cosmos.orbital`: `delta_v`, `kepler_period`, `gravitational_parameter`, `hohmann_delta_v`
  - `cosmos.spectral`: doppler + Wien peak + simple redshift helpers
  - `cosmos.signal`, `cosmos.quantum`, `cosmos.thermo`, `cosmos.geo`, `cosmos.chem`: minimal functional cores

- **General-purpose constellations (`nova.*`)**:
  - `nova.fs`: basic filesystem ops
  - `nova.cli`: argparse-based CLI helper functions
  - `nova.fmt`: JSON/YAML/TOML helpers (YAML/TOML depend on optional libs)
  - `nova.net`: tiny HTTP server router + `http_get/http_post` client helpers
  - `nova.db`: SQLite wrapper
  - `nova.concurrent`: `Channel` + `spawn()`
  - `nova.crypto`: hashing + HMAC
  - `nova.test`: assertion helpers

## How to run the minimal working version

### 1) Stdlib demo (smoke test)

From repo root:

```bash
python STDLIB_DEMO.py
```

This will:
- run a few `cosmos.*` computations
- insert results into an in-memory SQLite DB (`nova.db`)
- save a plot to `stdlib_demo_plot.png`

### 2) Step 12 integration tests (stdlib)

```bash
python tests/integration/run_stdlib_step12_unittest.py
```

### 3) Step 13 REPL (interactive)

```bash
python -m toolchain.nova_repl
```

Convenience launchers:
- From repo root: `python nova_repl.py`
- From `compiler/`: `python nova_repl.py`

### 4) Run “everything” (Windows PowerShell)

```powershell
powershell -ExecutionPolicy Bypass -File scripts/run_all_tests.ps1
```

If you don’t have Rust/Java installed yet:

```powershell
powershell -ExecutionPolicy Bypass -File scripts/run_all_tests.ps1 -SkipRust -SkipJava
```

Useful REPL commands:
- `:help`
- `:absorb cosmos.stats.{ pearson, linear_fit }`
- `:type <expr>` (Python type for now)
- `:vars`, `:reset`, `:quit`

## Dependencies (Python)

`stdlib/requirements.txt` and `stdlib/requirements-minimal.txt` exist. Optional capabilities:
- FITS reading uses `astropy` if installed, otherwise errors for `.fits` inputs.
- HDF5 reading uses `h5py` (optional).
- NetCDF reading uses `netCDF4` (optional).
- YAML/TOML support in `nova.fmt` requires `pyyaml`, `tomli`, `tomli_w`.

## Key design constraints to preserve (do not break)

- **Units are compile-time only**: unit dimensions must never become runtime baggage.
- **Unit mismatch must be a compile-time error** (core promise).
- **No implicit coercions** between incompatible units.
- Prefer **domain-aware error messages** (name SI dimensions).
- Keep the multi-language split: don’t “simplify” by rewriting everything in one language.

## Known gaps / next concrete tasks

### Stdlib breadth (Step 12)

Docs mention broader coverage than the current minimal prototype:
- `cosmos.data`: Parquet/Arrow/HDF5/NetCDF coverage can be expanded and hardened; FITS should be first-class once `astropy` is included in minimal dev deps.
- `cosmos.astro`: coordinate transforms, magnitude helpers, catalogue lookup, etc.
- `nova.net`: websockets/gRPC are not implemented (prototype is HTTP-only).
- `nova.db`: migrations/kv/time-series are not implemented (SQLite only).

### Compiler wiring (biggest “minimal compiler” blocker)

Right now, the repo has:
- **C AST** produced by the parser
- **Rust typechecker/unit resolver** that work on Rust-side representations

But there is **no fully wired bridge** from the C AST into the Rust semantic/type pipeline + codegen. The next major step for a “minimal compiler” is:
- define a stable serialized AST boundary (e.g., JSON, flatbuffers, or C FFI structs)
- ingest that into Rust
- run semantic + typecheck
- emit LLVM IR / run with an LLVM toolchain

### “Minimal compiler” milestone (recommended)

Implement a narrow slice end-to-end:
- Parse a tiny subset (`mission main() -> Void { transmit("hi") }`)
- Lower to a Rust-side IR
- Emit textual LLVM IR
- (Optional) run with LLVM tools when available

## Pointers to key files

- Lexer: `compiler/lexer/src/lexer.c`, `compiler/lexer/include/token.h`
- Parser: `compiler/parser/src/parser.c`, `compiler/parser/include/ast.h`
- Unit resolver: `compiler/unit_resolver/src/*`
- Typechecker: `compiler/typechecker/src/*`
- Codegen IR emitter: `compiler/codegen/src/ir_emitter.rs`
- Stdlib: `stdlib/cosmos/*`, `stdlib/nova/*`
- Stdlib tests: `tests/integration/run_stdlib_step12_unittest.py`
- REPL: `toolchain/nova_repl/repl.py`
- Root REPL launcher: `nova_repl.py`
- Compiler-dir REPL launcher: `compiler/nova_repl.py`
- All-tests script (Windows): `scripts/run_all_tests.ps1`

