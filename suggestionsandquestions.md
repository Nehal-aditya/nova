# NOVA — Open Design Questions & Suggestions

This document collects unresolved design decisions, open questions, and forward-looking suggestions for NOVA. It is a living document — add new questions as they arise during implementation. Each entry has: the question, context, options, and a recommendation to consider. Decisions move to the specification once agreed.

---

## Locked decisions (for reference)

These syntax decisions are finalised in v0.2 and are not open questions:

| Decision | Chosen form |
|---|---|
| Import syntax | `absorb cosmos.stats.{ pearson, linear_fit }` |
| Transform pipeline | `pipeline [filter(...), map(...), drop_outliers(...)]` |
| Return type arrow | `→` (Unicode), `->` (ASCII alias), formatter normalises to `→` |
| Output built-in | `transmit("message {value:.4}")` |
| String interpolation | `{expr:.decimals}` or `{expr as [unit]:.decimals}` |
| Scientific namespace | `cosmos.*` |
| General-purpose namespace | `nova.*` |
| File extension | `.nv` |
| Package manager | `nebula` |

---

## Section 1 — Core language decisions (blockers for Phase 1)

### 1.1 Memory model

**Question:** Should NOVA use garbage collection, reference counting (ARC), or region-based memory?

**Context:** NOVA targets scientific workloads with predictable allocation patterns — batch tensors, fixed-size arrays, simulation grids. A GC pause in the middle of a training loop or a rocket trajectory integration step is unacceptable. But ownership rules (Rust-style) carry a steep learning curve that would put off scientists and researchers who are NOVA's primary audience.

**Options:**

- **GC with GC-free zones** — default GC, but missions annotated `#[nogc]` get a bump allocator and no GC pauses. Scientists use the default; performance-critical code opts out.
- **ARC (automatic reference counting)** — deterministic, no pauses, occasional retain cycle risk. Swift's model. Familiar to iOS developers; unfamiliar to scientists.
- **Region-based memory** — all memory for a mission is allocated into a region on entry, freed atomically on exit. No per-object tracking, no GC, no ownership annotations. Closest to how scientific code actually allocates: fill a batch, process it, discard it.
- **Ownership (Rust-style)** — maximum safety and performance; maximum learning curve.

**Recommendation to consider:** Region-based memory as the default. Each `mission` call owns a region. Tensors and arrays allocated inside a mission live in its region and are freed when the mission returns. For persistent state (model weights, loaded datasets), an explicit `persistent` annotation promotes the allocation to a long-lived region. This gives scientists a simple mental model — "everything in this function is freed when I return" — without ownership annotations or GC pauses.

---

### 1.2 Mutability of model weights during `autodiff`

**Question:** How is mutation of `model` weights inside an `autodiff` block represented in the type system?

**Context:** The canonical training loop is:

```nova
let loss = cross_entropy(net.forward(batch.x), batch.y)
autodiff(loss) { net.update(adam, lr=3e-4) }
```

`net.update(...)` mutates the weights of `net`. But `net` is declared with `let` (immutable). This is a contradiction that must be resolved before the type checker is designed.

**Options:**

- **Models are always reference types** — `model` types are implicitly reference-counted. `let net = StarClassifier.init()` binds a reference; `net.update(...)` mutates through the reference. `let` means the reference cannot be rebound, but the referent can be mutated.
- **`autodiff` block gets a mutable borrow** — inside `autodiff { ... }`, the captured model is implicitly mutable. Outside, it is immutable. The `autodiff` block is a scoped mutation zone.
- **`var` required for trained models** — `var net = StarClassifier.init()`. Mutation is explicit. The type checker requires `var` for any model that is updated.
- **Separate `Trainable[M]` wrapper type** — `let net = Trainable(StarClassifier.init())`. `net.update(...)` is a mutation on the wrapper.

**Recommendation to consider:** Models are reference types (option 1). This matches how every existing ML framework handles model weights. Scientists will not be surprised. The type system documents this with a distinct `Model` kind — `let net: StarClassifier` where `StarClassifier` is a `Model` type — and `let` means the variable cannot be rebound to a different model, but the weights can be updated.

---

### 1.3 Error handling

**Question:** Should NOVA use `Result[T, E]` types, exceptions, or both?

**Context:** Python (exception-based) is the language most NOVA scientists know. Rust and Haskell (`Result`/`Either`) produce more robust code. The `?` propagation operator has already been included in the operator table.

**Options:**

- **`Result[T, E]` only** — functional, explicit, composable with `?`. Learning curve for scientists used to exceptions.
- **Exceptions only** — familiar; hard to reason about in `parallel mission` blocks where exception handlers need to exist on the right thread.
- **Both** — library code uses `Result`; missions can `raise` exceptions at the top level.

**Recommendation to consider:** `Result[T, E]` with the `?` propagation operator for library and pipeline code. A top-level `panic(msg)` for unrecoverable errors. No exception syntax in the language. Scientists who want "just crash on error" use `?` chaining or `unwrap()`. Robust code handles `Result` explicitly. The `pipeline [...]` construct propagates errors through the chain automatically.

---

### 1.4 Dimensionless quantity representation

**Question:** When `Float[m]` is divided by `Float[m]`, what is the result type?

**Context:** Dimensionless quantities appear everywhere: Mach number, efficiency, probability, signal-to-noise ratio, the argument to `ln(...)`. They should be distinguishable from an untracked float (which could have any unit) but compatible with `Float` in mathematical expressions.

**Options:**

- **`Float`** — dimension cancellation produces a plain `Float`. Simple. Can't distinguish "intentionally dimensionless" from "forgot to annotate."
- **`Float[1]`** — explicit dimensionless type. `ln(x)` requires `Float[1]`. Conversions from `Float[%]` to `Float[1]` are explicit (`as [1]`).
- **`Float[dimensionless]`** — more readable alias for `Float[1]`. Same semantics.

**Recommendation to consider:** `Float[1]` as the canonical dimensionless type, with `Float[dimensionless]` as a display alias. Implicit conversion to `Float` in arithmetic (so `sqrt(efficiency)` works without a cast), but explicit conversion required for function arguments that are declared `Float` (so unit safety is preserved at API boundaries). `Float[%]` is `Float[1/100]` — it converts to `Float[1]` with `as [1]`.

---

### 1.5 Autodiff strategy: static trace vs dynamic tape

**Question:** Should `autodiff` use a static (compile-time) trace or a dynamic (runtime) tape?

**Context:** Static graphs (JAX/XLA) enable aggressive ahead-of-time optimisation. Dynamic graphs (PyTorch) are more flexible and easier to debug, especially with control flow inside the forward pass.

**Options:**

- **Static trace** — `autodiff` blocks are specialised at compile time into explicit gradient computation. Fast. Cannot handle data-dependent control flow inside the differentiable computation.
- **Dynamic tape** — a tape of operations is built at runtime during the forward pass. Gradient is computed by replaying the tape in reverse. Flexible. Small overhead.
- **Hybrid (trace with dynamic fallback)** — trace static patterns (linear chains of tensor ops); emit dynamic tape for branches and loops.

**Recommendation to consider:** Dynamic tape for v1. It handles all real training patterns including data-dependent branching in model architectures. Move to a static trace for hot paths (detected by profiling) in v2.

---

### 1.6 Parallel safety model

**Question:** How does NOVA prevent data races in `parallel mission` blocks?

**Context:** Two parallel workers writing to the same `Array` index is a data race. NOVA must either prevent this or make it safe.

**Options:**

- **Immutable inputs** — `parallel mission` inputs are implicitly immutable. The mission receives a read-only view; outputs are new values (functional style). No mutation inside a parallel mission.
- **Compile-time access analysis** — analyse access patterns at compile time; reject missions where two workers could write the same index.
- **Runtime synchronisation** — atomic operations or locks. Flexible; slower; leaks complexity to the programmer.

**Recommendation to consider:** Immutable inputs for v1. A `parallel mission` cannot mutate its inputs — it receives copies and returns new values. This is safe by construction and maps cleanly to the map-reduce patterns that dominate scientific pipelines. Mutable shared state in parallel contexts is a Phase 5 concern.

---

## Section 2 — Type system

### 2.1 Integer units

**Question:** Should `Int` support unit annotations?

**Context:** Counting things has units: `Int[photons]`, `Int[samples]`, `Int[stars]`. These are more like phantom type labels than SI dimensions.

**Recommendation to consider:** Allow `Int[label]` as a phantom type for documentation and category safety — `Int[photons]` cannot be added to `Int[stars]` — but don't give integer labels SI dimensions. Arithmetic between `Int[photons]` and `Float[photons]` promotes to `Float[photons]`.

---

### 2.2 Tensor units

**Question:** Can a tensor carry a unit? E.g. `Tensor[Float[m/s], 3]` for a velocity vector?

**Context:** A 3-vector of velocities is naturally typed `Tensor[Float[m/s], 3]`. Neural network outputs are dimensionless. Physics-informed networks mix both.

**Recommendation to consider:** Allow `Tensor[Float[unit], shape]`. Elementwise ops preserve units. `softmax`, `relu`, and other activation functions require dimensionless input — checked at compile time. Neural network layer outputs are `Float[1]` (dimensionless) by definition unless annotated otherwise.

---

### 2.3 Wave type (lazy sequences)

**Question:** What is the full semantics of `Wave`?

**Context:** `Wave` is the lazy stream / sequence type used as the type of the `catalog` argument in the stellar demo. It needs precise semantics for: how it is constructed, how it interacts with `pipeline [...]`, whether it is pull-based or push-based, and how it integrates with GPU and distributed execution.

**Recommendation to consider:** `Wave` is a pull-based lazy sequence — values are produced on demand. A `Wave[T]` is analogous to a Rust `Iterator<Item=T>` or a Haskell lazy list. `pipeline [...]` applied to a `Wave` is lazy — no work is done until the result is consumed (e.g. by `transmit`, `scatter`, or written to a file). A `parallel mission` that consumes a `Wave` processes elements in chunks across workers.

---

## Section 3 — AI and autodiff

### 3.1 Higher-order differentiation

**Question:** Should NOVA support second-order and higher derivatives (Hessians)?

**Context:** Newton's method, natural gradient descent, and some physics simulations require second derivatives.

**Recommendation to consider:** Yes, via composable `gradient` expressions: `gradient(gradient(loss, wrt params), wrt params)` produces a Hessian-vector product. This is a consequence of making `gradient` a first-class expression rather than a special statement. No additional syntax needed.

---

### 3.2 Device placement

**Question:** How does NOVA decide whether a tensor lives on CPU or GPU?

**Options:**

- **Automatic** — runtime decides based on tensor size and operation; programmer sets no hints.
- **`on device(gpu) { ... }` block** — explicit placement for everything inside the block.
- **Tensor constructor** — `Tensor([1,2,3], device=.gpu)`.

**Recommendation to consider:** `on device(gpu) { ... }` blocks for explicit placement, with a runtime heuristic that moves tensors to GPU automatically when a GPU operation is invoked and the tensor exceeds a configurable size threshold. Crossing device boundaries outside a block emits a compiler warning.

---

### 3.3 Custom layer types

**Question:** How does a user define a custom `layer` type beyond the built-ins?

**Context:** The built-in layers (`dense`, `conv1d`, `attention`, etc.) cover most cases. But novel architectures need custom layers.

**Recommendation to consider:**

```nova
struct FourierLayer {
  n_modes : Int
  weights : Tensor[Float, _x_]
}

impl Layer for FourierLayer {
  mission forward(self, x: Tensor[Float, _]) → Tensor[Float, _] { ... }
  mission backward(self, grad: Tensor[Float, _]) → Tensor[Float, _] { ... }
}
```

A `struct` that implements the `Layer` trait can be used inside a `model` block as a named layer. The compiler checks that `forward` and `backward` have compatible shapes.

---

## Section 4 — Parallelism

### 4.1 Nested parallelism

**Question:** What happens when a `parallel mission` calls another `parallel mission`?

**Context:** Naive nested parallelism oversubscribes cores.

**Recommendation to consider:** A single global work-stealing pool (Rayon). Nested `parallel mission` calls submit tasks to the same pool. No new threads are spawned. Oversubscription cannot happen by construction.

---

### 4.2 Streaming parallelism for `Wave`

**Question:** How does `pipeline [...]` parallelise over a `Wave` (lazy stream)?

**Context:** A `Wave` is potentially infinite (e.g. a live sensor feed). Parallelism must be chunked and bounded.

**Recommendation to consider:** When a `parallel mission` consumes a `Wave` through `pipeline [...]`, the runtime chunks the wave into fixed-size windows (default: 1024 elements, configurable) and distributes chunks across workers. The output is another `Wave` whose element order matches the input.

---

## Section 5 — Ecosystem

### 5.1 Nebula Registry name

**Question:** What is the public package registry called?

**Suggestions:** `The Nebula Registry`, `Nova Central`, `Constellation Hub`, `The Observatory`, `Nebulae`

**Recommendation to consider:** `The Nebula Registry` — consistent with the package manager name (`nebula`), evocative of NOVA's astronomical inspiration.

---

### 5.2 Interoperability with Python and C

**Question:** How does NOVA call existing scientific Python libraries (SciPy, Astropy) and C libraries (BLAS, LAPACK, cfitsio)?

**Options:**

- **C FFI only** — NOVA calls C functions via a `foreign` declaration. Python interop goes through C extensions.
- **Python bridge** — a `python { ... }` block embeds Python calls inline.
- **Native NOVA constellations** — community writes NOVA wrappers for popular libraries.

**Recommendation to consider:** C FFI for v1 (required for BLAS/LAPACK). Encourage native NOVA constellations for major libraries as the primary long-term strategy — `cosmos.astro` replaces Astropy, `cosmos.ml` replaces PyTorch. A Python bridge in v2 for the long tail.

```nova
foreign mission cblas_dgemm(
  order: Int, transA: Int, transB: Int,
  m: Int, n: Int, k: Int,
  alpha: Float, A: Ptr[Float], lda: Int,
  B: Ptr[Float], ldb: Int,
  beta: Float, C: Ptr[Float], ldc: Int
) → Void from "cblas"
```

---

### 5.3 NOVA for education

**Question:** Should there be an educational subset of NOVA?

**Context:** NOVA is an ideal language for teaching physics and data science together — units prevent the most common student errors, and `pipeline [...]` teaches functional thinking naturally.

**Recommendation to consider:** A browser-based NOVA sandbox (WASM compilation) with a restricted subset: no `parallel`, no `foreign`, no `on device`. Used in university courses for computational physics and data science. The H-R diagram, stellar main sequence, and Hubble constant examples from the standard demos become the curriculum.

---

## Section 6 — Forward-looking suggestions

These are not blockers for any current phase. Track for future versions.

### 6.1 Symbolic mathematics

A `symbolic` type carrying an expression tree, enabling computer algebra:

```nova
let F = symbolic(m * a)               -- Newton's second law
let a_expr = F.solve_for(a)           -- a = F / m
let numeric = a_expr.eval(F=10.0[N], m=2.0[kg])  -- 5.0[m/s²]
```

### 6.2 Proof annotations

Missions carry postcondition annotations checked by an SMT solver:

```nova
mission delta_v(...) → Float[m/s]
  ensures result > 0.0[m/s]
  ensures result < 30000.0[m/s]   -- physically reasonable upper bound
{
  ...
}
```

Useful for safety-critical aerospace and medical device code.

### 6.3 Uncertainty propagation

A `Measured[T]` type that carries a value and its standard uncertainty:

```nova
let g = Measured(9.80665, ±0.00005)[m/s²]
let F = Measured(10.0, ±0.1)[N]
let m = F / g    -- Measured[Float[kg]] with propagated uncertainty
```

Arithmetic on `Measured` values propagates uncertainty using standard quadrature rules. Essential for experimental physics.

### 6.4 Plot mission as a first-class statement

```nova
plot(
  x: stars.mass,    xlabel: "Mass (M☉)",
  y: stars.lum,     ylabel: "log₁₀ Luminosity (L☉)",
  color_by: stars.surface_temp,
  title: "Stellar main sequence"
)
```

A built-in `plot` mission that automatically labels axes with units, produces inline output in the REPL and Jupyter, and writes SVG/PNG to file in script mode. Eliminates the matplotlib import-and-configure boilerplate that slows down scientific exploration.

### 6.5 Dimensional analysis hints in error messages

When a unit error is a recognisable physical law violation, the compiler cites the law:

```nova
let F = mass * velocity   -- Float[kg·m/s]
-- Error: mission `apply_force` expects Float[N] (= Float[kg·m/s²])
--        you provided Float[kg·m/s] (momentum, not force)
--        Newton's second law: F = m·a, not F = m·v
--        hint: did you mean `mass * acceleration`?
```

### 6.6 NOVA for embedded and real-time systems

A `#[realtime]` annotation on a mission that enforces:
- No heap allocation
- No GC
- Statically bounded stack depth
- No calls to non-realtime missions

Enables NOVA in flight computers, sensor firmware, and control systems — closing the loop between the simulation language and the hardware it models.

### 6.7 Native H-R diagram and sky map types

```nova
absorb cosmos.plot.{ hr_diagram, sky_map, orbital_plot }

hr_diagram(stars,
  x: b_minus_v,   xlabel: "Colour index B−V",
  y: abs_magnitude, ylabel: "Absolute magnitude Mᵥ",
  color_by: spectral_class)
```

Built-in plot types for the specific visualisations that appear in every astronomy course and paper — not just generic scatter plots.

---

## Decisions needed before Phase 1 begins

| Question | Section | Must resolve by |
|---|---|---|
| Memory model | 1.1 | Before type checker design |
| Mutability of model weights | 1.2 | Before type checker handles `autodiff` |
| Error handling strategy | 1.3 | Before parser/AST design |
| Dimensionless type | 1.4 | Before unit resolver design |
| Autodiff strategy | 1.5 | Before IR design |
| Parallel safety model | 1.6 | Before IR/parallel lowering |

---

*Add new questions below this line. Format: question, context, options, recommendation.*
