//! NOVA Type Checker
//!
//! Phase 2b of the NOVA compiler pipeline.
//!
//! ## What this crate does
//!
//! Implements Hindley-Milner type inference extended with:
//!   - SI unit dimension vectors  (Float[m/s], Float[kg])
//!   - Tensor rank/shape types    (Tensor[Float[eV], 1024])
//!   - Trait bounds               (T: Measurable + Displayable)
//!
//! The checker walks a typed representation of the AST, assigns type
//! variables to unknown types, and unifies them via the constraint solver
//! in `unify.rs`. Unit constraints are delegated to `nova_unit_resolver`.
//!
//! ## Pipeline position
//!
//!   Parser (C) → Unit Resolver (Rust) → [Type Checker (Rust)] → Semantic (Rust)
//!
//! ## Key types
//!
//! - `NovaType`   — the type algebra (primitives, functions, generics, units, tensors)
//! - `TypeVar`    — a unification variable (unknown type)
//! - `TypeEnv`    — maps names → types in scope
//! - `TypeChecker`— drives inference and unification
//! - `TypeError`  — structured error with source location

pub mod types;
pub mod infer;
pub mod unify;
pub mod traits;
pub mod tensor_types;

pub use types::{NovaType, TypeVar, FunctionType, TypeScheme};
pub use infer::{TypeChecker, TypeEnv};
pub use unify::{UnifyError};
pub use traits::{TraitBound, TraitRegistry};
