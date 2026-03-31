# NOVA Design Decisions Document

This document collects the unanswered design questions that will affect the compiler's type system and code generation strategy.

## 1. Memory Model

**Question:** How should NOVA manage memory?

### Options:

**Option A: Garbage Collection**
- Pros: Simplest to implement, no borrow checker, familiar to Python/Java developers
- Cons: Unpredictable pause times (bad for real-time systems, astronomy apps)
- Best for: Educational, rapid prototyping

**Option B: Reference Counting (ARC)**
- Pros: Predictable, works well with shared data, no runtime overhead
- Cons: Circular reference handling, overhead per clone, more complex API
- Best for: Systems programming, embedded

**Option C: Regions / Borrow Checking**
- Pros: No runtime overhead, zero-cost abstractions, Rust-like safety
- Cons: Steep learning curve, compiler complexity, strict rules
- Best for: Performance-critical scientific computing

**Option D: Hybrid (e.g., GC for user code, ARC for model objects)**
- Pros: Flexibility per use case
- Cons: Complex semantics, harder to explain to users

**Recommendation:** Option A (GC) - NOVA's target audience (scientists) typically prioritizes code clarity over performance tuning. Astronomy simulations have predictable pause requirements but can accept GC.

**Impact on Design:**
- Function signatures don't need lifetime parameters
- No borrow checker pass needed in Phase 1
- Type system simpler
- Runtime implementation deferred to Phase 3

---

## 2. Error Handling Strategy

**Question:** How should NOVA represent and propagate errors?

### Options:

**Option A: Result Types (`Result<T, E>`)**
- Pros: Composable, explicit, matches Rust/Scala patterns
- Cons: More verbose than exceptions, requires ceremony
- Best for: Systems programming, library code

**Option B: Exception-based (`throw`/`catch`)**
- Pros: Familiar to Python/Java users, clean happy path
- Cons: Implicit control flow, harder to trace, performance cost
- Best for: Rapid development, high-level scripting

**Option C: Both (choice per function)**
- Pros: Flexibility
- Cons: Inconsistent, confusing for users

**Option D: No built-in errors, use types (like Go)**
- Pros: Simple semantics
- Cons: Verbose, multi-value returns needed

**Recommendation:** Option A (Result Types) - More composable with functional pipeline syntax. Consistent with `transmit` (which can fail).

**Impact on Design:**
- Function return types can be `Result[T]` or `Option[T]`
- Type checker must understand Result/Option
- Error propagation with `?` operator (if implemented)

---

## 3. Model Weight Mutability

**Question:** Can model weights be modified during autodiff?

### Context:
```nova
model Network { ... }

mission train(mut model: Network) {
  autodiff {
    loss = model(input) - target
    -- Can we do: model.weights = model.weights - learning_rate * gradients?
    -- Or is autodiff pure (read-only) and updates happen outside?
  }
  return model  -- Return modified model
}
```

### Options:

**Option A: Weights are immutable during autodiff**
- Pros: Mathematical purity, easier to reason about
- Cons: Updates must happen outside autodiff block, extra complexity
- Best for: Functional programming style

**Option B: Weights are mutable, autodiff modifies in-place**
- Pros: Efficient, familiar (PyTorch default)
- Cons: Breaks functional purity, harder to compose

**Option C: Explicit update operators in autodiff**
- Pros: Clear intent
- Cons: New syntax, compiler complexity

**Recommendation:** Option A (weights immutable during autodiff, return new model) - Aligns with functional paradigm and NOVA's mathematical origins.

**Impact on Design:**
- Type system tracks mutable vs immutable references
- `autodiff` blocks cannot mutate captures
- Training loops explicitly rebind: `model = train(model)`

---

## 4. Dimensionless Type Representation

**Question:** How should dimensionless numbers be represented?

### Options:

**Option A: `Float` (no unit annotation)**
```nova
let x: Float = 3.14  -- dimensionless
let y: Float[meter] = 3.14  -- with unit
```
- Pros: Simple, clean
- Cons: Mix of unit and dimensionless (implicit unit = dimensionless)

**Option B: `Float[dimensionless]` (explicit)**
```nova
let x: Float[dimensionless] = 3.14
let y: Float[meter] = 3.14
```
- Pros: Explicit, symmetric
- Cons: Verbose

**Option C: `Float[1]` (numeric encoding)**
```nova
let x: Float[1] = 3.14  -- unit is 1 (dimensionless)
```
- Pros: Mathematical purity
- Cons: Non-obvious to users

**Recommendation:** Option A (`Float` = dimensionless) - Simpler, most intuitive.

**Impact on Design:**
- Unit resolution: `Float[meter] / Float[second]` → `Float[meter/second]`
- Operations between `Float` and `Float[unit]` are errors
- No implicit unit coercion

---

## 5. Autodiff Architecture

**Question:** Should autodiff use static or dynamic computational graphs?

### Context:
```nova
autodiff {
  predicate = model(x)
  loss = (predicate - target) ^ 2
  gradients = backward()  -- When computed?
}
```

### Options:

**Option A: Static Graphs ("eager")**
- Pros: Compile-time analysis possible, compile once run many times
- Cons: Can't handle dynamic control flow in differentiable code
- Examples: JAX, Theano

**Option B: Dynamic Graphs ("lazy")**
- Pros: Handles dynamic control flow, familiar (PyTorch)
- Cons: Runtime overhead, cannot optimize ahead of time
- Examples: PyTorch, TensorFlow eager

**Option C: Hybrid (user annotations to choose)**
- Pros: Flexibility
- Cons: Complex semantics, compiler coordination

**Recommendation:** Option A (static graphs with runtime scheduling) - Astronomy computations are typically static; emphasize compile-time safety.

**Impact on Design:**
- No loops/branches in autodiff blocks (or restricted forms)
- Deeper compile-time analysis
- Potential automatic optimization opportunities

---

## 6. Parallel Function Safety

**Question:** How are parallel functions guaranteed to be safe?

### Context:
```nova
parallel mission compute(data: Array[Float]) → Array[Float] {
  return data |> map(x => x * 2)
}
```

### Options:

**Option A: Compile-time analysis (borrow checker)**
- Pros: No runtime cost, guaranteed safety
- Cons: Requires borrow checker, steep learning curve
- Models: Rust, Cyclone

**Option B: Runtime safety checks**
- Pros: Simpler type system
- Cons: Runtime cost, less predictable

**Option C: User must prove safety (type annotations)**
```nova
pure parallel mission compute(immutable data: Array[Float]) { ... }
```
- Pros: Explicit, flexible
- Cons: Boilerplate, user responsibility

**Option D: Trust the user (no checks)**
- Pros: Simplest
- Cons: Potential data races, undefined behavior

**Recommendation:** Option C (user annotations with type checking) - Middle ground between Rust's strictness and Python's trust.

**Impact on Design:**
- Add `pure` / `immutable` / `mutable` keywords
- Type checker validates parallelizable operations
- Standard library functions marked as pure/impure

---

## 7. Generic Types

**Question:** Should NOVA support generic programming?

### Context:
```nova
struct Pair(T, U) {
  first: T
  second: U
}

mission first(pair: Pair(Int, String)) → Int {
  return pair.first
}
```

### Options:

**Option A: Full parametric polymorphism (like Haskell/Rust)**
- Pros: Powerful, type-safe, no runtime overhead
- Cons: Complex compiler, hard to debug
- Best for: Production systems

**Option B: Limited generics (single type param)**
- Pros: Simpler, handles common cases
- Cons: Restrictive

**Option C: Runtime type dispatch (like Python)**
- Pros: Simple in compiler, flexible
- Cons: No compile-time safety

**Option D: None (monomorphic only)**
- Pros: Simplest
- Cons: Code duplication

**Recommendation:** Option A (full generics) initially, Phase 2 - NOVA needs this for Array[T], Option[T], etc.

---

## Summary: Recommendation Path

### For Phase 1 Planning:
1. **Memory Model:** Garbage Collection
2. **Errors:** Result types
3. **Weights Immutability:** Immutable during autodiff, return new
4. **Dimensionless:** `Float` = dimensionless, `Float[unit]` = with unit
5. **Autodiff:** Static graphs
6. **Parallel Safety:** User annotations + type checking
7. **Generics:** Full parametric (Phase 2), deferred

### Decisions to Finalize:
- [ ] Confirm memory model selection
- [ ] Confirm error handling approach
- [ ] Confirm autodiff mutability semantics
- [ ] Confirm parallel safety strategy

**Once confirmed, Phase 1 design is locked in.**

---

**Document version:** 0.1  
**Last updated:** March 31, 2026  
**Status:** Draft - Awaiting approval
