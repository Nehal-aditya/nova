//! Type Inference Engine
//!
//! Walks the NOVA AST and infers types for every expression, statement,
//! and declaration. Uses the unification engine (`unify.rs`) to solve
//! type constraints, the unit resolver for dimension checking, and the
//! trait registry for bound checking.
//!
//! ## Design
//!
//! The inferencer works in a single forward pass over the AST:
//!   1. Assign a fresh type variable to each unannotated binding.
//!   2. For each expression, infer its type and emit unification constraints.
//!   3. The unifier solves constraints eagerly (not deferred).
//!   4. After inference, apply the final substitution to get concrete types.
//!
//! This is Algorithm W (Hindley-Milner) with eager unification rather than
//! a separate constraint-solving phase — simpler to implement correctly for
//! a language with this size of type algebra.

use crate::types::{NovaType, TypeVar, TypeScheme, FunctionType, TensorShape};
use crate::unify::{Subst, UnifyError, unify};
use crate::traits::{TraitBound, TraitRegistry, BoundError};
use nova_unit_resolver::UnitResolver;
use std::collections::HashMap;

// ── Type environment ──────────────────────────────────────────────────────────

/// A scoped mapping from variable names to type schemes.
/// Supports nested scopes (for blocks, function bodies).
#[derive(Debug, Clone)]
pub struct TypeEnv {
    scopes: Vec<HashMap<String, TypeScheme>>,
}

impl TypeEnv {
    pub fn new() -> Self {
        TypeEnv { scopes: vec![HashMap::new()] }
    }

    /// Push a new scope.
    pub fn push_scope(&mut self) {
        self.scopes.push(HashMap::new());
    }

    /// Pop the innermost scope.
    pub fn pop_scope(&mut self) {
        if self.scopes.len() > 1 { self.scopes.pop(); }
    }

    /// Bind a name to a monomorphic type in the current scope.
    pub fn bind(&mut self, name: impl Into<String>, ty: NovaType) {
        self.scopes.last_mut().unwrap()
            .insert(name.into(), TypeScheme::mono(ty));
    }

    /// Bind a name to a type scheme (for let-polymorphism).
    pub fn bind_scheme(&mut self, name: impl Into<String>, scheme: TypeScheme) {
        self.scopes.last_mut().unwrap().insert(name.into(), scheme);
    }

    /// Look up a name, searching from innermost to outermost scope.
    pub fn lookup(&self, name: &str) -> Option<&TypeScheme> {
        for scope in self.scopes.iter().rev() {
            if let Some(s) = scope.get(name) { return Some(s); }
        }
        None
    }

    /// Instantiate a type scheme: replace quantified vars with fresh vars.
    pub fn instantiate(&self, scheme: &TypeScheme) -> NovaType {
        if scheme.quantified.is_empty() {
            return scheme.body.clone();
        }
        let fresh_map: HashMap<TypeVar, TypeVar> = scheme.quantified.iter()
            .map(|&v| (v, TypeVar::fresh()))
            .collect();
        subst_vars(&scheme.body, &fresh_map)
    }
}

impl Default for TypeEnv { fn default() -> Self { Self::new() } }

/// Replace type variables according to `map` (for scheme instantiation).
fn subst_vars(ty: &NovaType, map: &HashMap<TypeVar, TypeVar>) -> NovaType {
    match ty {
        NovaType::Var(v) => {
            if let Some(&fresh) = map.get(v) { NovaType::Var(fresh) }
            else { ty.clone() }
        }
        NovaType::Array(t)   => NovaType::Array(Box::new(subst_vars(t, map))),
        NovaType::Wave(t)    => NovaType::Wave(Box::new(subst_vars(t, map))),
        NovaType::Option(t)  => NovaType::Option(Box::new(subst_vars(t, map))),
        NovaType::Result(t, e) =>
            NovaType::Result(Box::new(subst_vars(t, map)), Box::new(subst_vars(e, map))),
        NovaType::Tuple(ts)  => NovaType::Tuple(ts.iter().map(|t| subst_vars(t, map)).collect()),
        NovaType::Function(ft) => NovaType::Function(FunctionType {
            params:   ft.params.iter().map(|p| subst_vars(p, map)).collect(),
            ret:      Box::new(subst_vars(&ft.ret, map)),
            parallel: ft.parallel,
        }),
        NovaType::Named { name, type_args } => NovaType::Named {
            name: name.clone(),
            type_args: type_args.iter().map(|a| subst_vars(a, map)).collect(),
        },
        _ => ty.clone(),
    }
}

// ── Type error ────────────────────────────────────────────────────────────────

/// A type error from the inference pass.
#[derive(Debug, Clone)]
pub enum TypeError {
    Unification(UnifyError),
    TraitBound(BoundError),
    UndefinedVariable { name: String, line: u32, col: u32 },
    UndefinedType     { name: String, line: u32, col: u32 },
    UnitError         { message: String, line: u32, col: u32 },
    Other             { message: String, line: u32, col: u32 },
}

impl TypeError {
    pub fn line(&self) -> u32 {
        match self {
            TypeError::Unification(e)          => e.line,
            TypeError::TraitBound(e)           => e.line,
            TypeError::UndefinedVariable { line, .. } => *line,
            TypeError::UndefinedType { line, .. }     => *line,
            TypeError::UnitError { line, .. }         => *line,
            TypeError::Other { line, .. }             => *line,
        }
    }
    pub fn message(&self) -> String {
        match self {
            TypeError::Unification(e)          => e.message.clone(),
            TypeError::TraitBound(e)           => e.message.clone(),
            TypeError::UndefinedVariable { name, .. } =>
                format!("undefined variable '{}'", name),
            TypeError::UndefinedType { name, .. } =>
                format!("undefined type '{}'", name),
            TypeError::UnitError { message, .. } => message.clone(),
            TypeError::Other { message, .. }     => message.clone(),
        }
    }
}

impl From<UnifyError> for TypeError {
    fn from(e: UnifyError) -> Self { TypeError::Unification(e) }
}

// ── TypeChecker ───────────────────────────────────────────────────────────────

/// The main type checker. Drives inference over NOVA AST constructs.
/// (In Phase 7, this will walk the C AST via FFI; here we expose its
/// logic for testing with in-process type expressions.)
pub struct TypeChecker {
    pub env:    TypeEnv,
    pub subst:  Subst,
    pub traits: TraitRegistry,
    pub units:  UnitResolver,
    pub errors: Vec<TypeError>,
}

impl TypeChecker {
    pub fn new() -> Self {
        let mut tc = TypeChecker {
            env:    TypeEnv::new(),
            subst:  Subst::new(),
            traits: TraitRegistry::new(),
            units:  UnitResolver::new(),
            errors: Vec::new(),
        };
        tc.seed_builtins();
        tc
    }

    /// Unify two types, recording an error on failure.
    pub fn unify(&mut self, lhs: &NovaType, rhs: &NovaType, line: u32, col: u32) {
        if let Err(e) = unify(lhs, rhs, &mut self.subst, line, col) {
            self.errors.push(TypeError::Unification(e));
        }
    }

    /// Apply current substitution to a type.
    pub fn apply(&self, ty: &NovaType) -> NovaType {
        self.subst.apply(ty)
    }

    /// Infer the type of a literal integer.
    pub fn infer_int_lit(&self) -> NovaType { NovaType::Int }

    /// Infer the type of a literal float (no unit).
    pub fn infer_float_lit(&self) -> NovaType { NovaType::Float }

    /// Infer the type of a unit literal: parse the unit string and
    /// produce Float[unit].
    pub fn infer_unit_lit(&mut self, unit_str: &str, line: u32, col: u32) -> NovaType {
        match self.units.resolve_unit_str(unit_str) {
            Ok(ru) => NovaType::UnitFloat { dim: ru.dim, unit_str: unit_str.into() },
            Err(e) => {
                self.errors.push(TypeError::UnitError {
                    message: e.message,
                    line, col,
                });
                NovaType::Float
            }
        }
    }

    /// Infer the type of a string literal.
    pub fn infer_string_lit(&self) -> NovaType { NovaType::Str }

    /// Infer the type of a bool literal.
    pub fn infer_bool_lit(&self) -> NovaType { NovaType::Bool }

    /// Look up a variable in the environment.
    pub fn infer_ident(&mut self, name: &str, line: u32, col: u32) -> NovaType {
        match self.env.lookup(name) {
            Some(scheme) => self.env.instantiate(scheme),
            None => {
                self.errors.push(TypeError::UndefinedVariable {
                    name: name.to_string(), line, col,
                });
                NovaType::Var(TypeVar::fresh()) // recover with a fresh var
            }
        }
    }

    /// Infer the type of a binary arithmetic expression.
    /// For unit-annotated operands, computes the result dimension.
    pub fn infer_binop_arith(
        &mut self,
        op: BinOp,
        lhs_ty: &NovaType,
        rhs_ty: &NovaType,
        line: u32, col: u32,
    ) -> NovaType {
        let lhs = self.apply(lhs_ty);
        let rhs = self.apply(rhs_ty);

        match op {
            BinOp::Add | BinOp::Sub => {
                // Both operands must have the same type (inc. unit)
                self.unify(&lhs, &rhs, line, col);
                lhs
            }
            BinOp::Mul => {
                // Unit multiplication: dim = lhs.dim + rhs.dim
                match (&lhs, &rhs) {
                    (
                        NovaType::UnitFloat { dim: d1, unit_str: u1 },
                        NovaType::UnitFloat { dim: d2, unit_str: u2 },
                    ) => {
                        let result_dim = d1.mul(*d2);
                        // Build a display unit string
                        let unit_str = format!("{}*{}", u1, u2);
                        NovaType::UnitFloat { dim: result_dim, unit_str }
                    }
                    (NovaType::UnitFloat { .. }, NovaType::Float) => lhs.clone(),
                    (NovaType::Float, NovaType::UnitFloat { .. }) => rhs.clone(),
                    _ => {
                        // Numeric: unify to same type
                        self.unify(&lhs, &rhs, line, col);
                        lhs
                    }
                }
            }
            BinOp::Div => {
                match (&lhs, &rhs) {
                    (
                        NovaType::UnitFloat { dim: d1, unit_str: u1 },
                        NovaType::UnitFloat { dim: d2, unit_str: u2 },
                    ) => {
                        let result_dim = d1.div(*d2);
                        let unit_str = format!("{}/{}", u1, u2);
                        NovaType::UnitFloat { dim: result_dim, unit_str }
                    }
                    (NovaType::UnitFloat { .. }, NovaType::Float) => lhs.clone(),
                    (NovaType::Float, NovaType::Float) => NovaType::Float,
                    _ => {
                        self.unify(&lhs, &rhs, line, col);
                        lhs
                    }
                }
            }
            BinOp::Pow => {
                // Power of unit type: exponent must be numeric integer
                match &lhs {
                    NovaType::UnitFloat { dim, unit_str } => {
                        // For now, constant-fold when rhs is Int literal
                        // Full constant propagation is in the semantic pass
                        NovaType::UnitFloat { dim: dim.pow(2), unit_str: format!("{}^2", unit_str) }
                    }
                    _ => lhs.clone(),
                }
            }
            BinOp::Eq | BinOp::Ne => {
                self.unify(&lhs, &rhs, line, col);
                NovaType::Bool
            }
            BinOp::Lt | BinOp::Gt | BinOp::Le | BinOp::Ge => {
                self.unify(&lhs, &rhs, line, col);
                NovaType::Bool
            }
            BinOp::And | BinOp::Or => {
                self.unify(&lhs, &NovaType::Bool, line, col);
                self.unify(&rhs, &NovaType::Bool, line, col);
                NovaType::Bool
            }
            BinOp::Matmul => {
                // Handled separately by tensor_types::infer_matmul
                // Here we just return a fresh var (tensor pass fills it in)
                NovaType::Var(TypeVar::fresh())
            }
        }
    }

    /// Check a function call: callee type must be a function, args must match params.
    /// Returns the inferred return type.
    pub fn infer_call(
        &mut self,
        callee_ty: &NovaType,
        arg_types: &[NovaType],
        line: u32, col: u32,
    ) -> NovaType {
        let callee = self.apply(callee_ty);
        match callee {
            NovaType::Function(ref ft) => {
                if ft.params.len() != arg_types.len() {
                    self.errors.push(TypeError::Other {
                        message: format!(
                            "wrong number of arguments: expected {}, got {}",
                            ft.params.len(), arg_types.len()
                        ),
                        line, col,
                    });
                    return NovaType::Var(TypeVar::fresh());
                }
                for (param, arg) in ft.params.iter().zip(arg_types.iter()) {
                    self.unify(param, arg, line, col);
                }
                *ft.ret.clone()
            }
            NovaType::Var(v) => {
                // Callee type unknown: create a function type and unify
                let ret_var = TypeVar::fresh();
                let fn_ty = NovaType::Function(FunctionType {
                    params: arg_types.to_vec(),
                    ret:    Box::new(NovaType::Var(ret_var)),
                    parallel: false,
                });
                unify(&NovaType::Var(v), &fn_ty, &mut self.subst, line, col).ok();
                NovaType::Var(ret_var)
            }
            _ => {
                self.errors.push(TypeError::Other {
                    message: format!("type {} is not callable", callee),
                    line, col,
                });
                NovaType::Var(TypeVar::fresh())
            }
        }
    }

    /// Check that a return type matches the declared mission return type.
    pub fn check_return(&mut self, actual: &NovaType, expected: &NovaType, line: u32, col: u32) {
        self.unify(actual, expected, line, col);
    }

    /// Check trait bounds for a generic type parameter.
    pub fn check_bounds(&mut self, ty: &NovaType, bounds: &[TraitBound], line: u32, col: u32) {
        if let Err(e) = self.traits.check_bounds(ty, bounds, line, col) {
            self.errors.push(TypeError::TraitBound(e));
        }
    }

    pub fn had_errors(&self) -> bool { !self.errors.is_empty() }

    pub fn print_errors(&self) {
        for e in &self.errors {
            eprintln!("{}:{}: error: {}", e.line(), 0, e.message());
        }
    }

    /// Seed the environment with built-in mission types.
    fn seed_builtins(&mut self) {

        // transmit: (Str) -> Void
        self.env.bind("transmit", NovaType::Function(FunctionType {
            params: vec![NovaType::Str],
            ret: Box::new(NovaType::Void),
            parallel: false,
        }));

        // ln: (Float) -> Float  (also works for UnitFloat via Float unification)
        self.env.bind("ln", NovaType::Function(FunctionType {
            params: vec![NovaType::Float],
            ret: Box::new(NovaType::Float),
            parallel: false,
        }));

        // sqrt: (Float) -> Float
        self.env.bind("sqrt", NovaType::Function(FunctionType {
            params: vec![NovaType::Float],
            ret: Box::new(NovaType::Float),
            parallel: false,
        }));

        // log10: (Float) -> Float
        self.env.bind("log10", NovaType::Function(FunctionType {
            params: vec![NovaType::Float],
            ret: Box::new(NovaType::Float),
            parallel: false,
        }));

        // cross_entropy: (Tensor, Tensor) -> Float
        let t_var = TypeVar::fresh();
        self.env.bind("cross_entropy", NovaType::Function(FunctionType {
            params: vec![
                NovaType::Tensor { elem: Box::new(NovaType::Var(t_var)), shape: TensorShape::Unknown },
                NovaType::Tensor { elem: Box::new(NovaType::Var(t_var)), shape: TensorShape::Unknown },
            ],
            ret: Box::new(NovaType::Float),
            parallel: false,
        }));

        // mse: (Tensor, Tensor) -> Float
        let t_var2 = TypeVar::fresh();
        self.env.bind("mse", NovaType::Function(FunctionType {
            params: vec![
                NovaType::Tensor { elem: Box::new(NovaType::Var(t_var2)), shape: TensorShape::Unknown },
                NovaType::Tensor { elem: Box::new(NovaType::Var(t_var2)), shape: TensorShape::Unknown },
            ],
            ret: Box::new(NovaType::Float),
            parallel: false,
        }));

        // pearson: (Array[Float], Array[Float]) -> Float
        self.env.bind("pearson", NovaType::Function(FunctionType {
            params: vec![
                NovaType::Array(Box::new(NovaType::Float)),
                NovaType::Array(Box::new(NovaType::Float)),
            ],
            ret: Box::new(NovaType::Float),
            parallel: false,
        }));
    }
}

impl Default for TypeChecker { fn default() -> Self { Self::new() } }

// ── BinOp enum (mirrors the C AST's NovaTokenType for operators) ─────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BinOp {
    Add, Sub, Mul, Div, Pow, Matmul,
    Eq, Ne, Lt, Gt, Le, Ge,
    And, Or,
}

// ── Tests ─────────────────────────────────────────────────────────────────────
#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::NovaType;
    use nova_unit_resolver::dimension::Dim;

    fn tc() -> TypeChecker { TypeChecker::new() }

    // ── Literal inference ────────────────────────────────────────────────────
    #[test]
    fn infer_int_lit() {
        assert_eq!(tc().infer_int_lit(), NovaType::Int);
    }

    #[test]
    fn infer_float_lit() {
        assert_eq!(tc().infer_float_lit(), NovaType::Float);
    }

    #[test]
    fn infer_unit_lit_kg() {
        let mut t = tc();
        let ty = t.infer_unit_lit("kg", 0, 0);
        assert!(!t.had_errors(), "unexpected errors: {:?}", t.errors);
        match ty {
            NovaType::UnitFloat { dim, .. } => assert_eq!(dim, Dim::MASS),
            other => panic!("expected UnitFloat, got {:?}", other),
        }
    }

    #[test]
    fn infer_unit_lit_m_per_s() {
        let mut t = tc();
        let ty = t.infer_unit_lit("m/s", 0, 0);
        assert!(!t.had_errors());
        match ty {
            NovaType::UnitFloat { dim, .. } => assert_eq!(dim, Dim::VELOCITY),
            _ => panic!("expected UnitFloat[m/s]"),
        }
    }

    #[test]
    fn infer_unit_lit_unknown_errors() {
        let mut t = tc();
        t.infer_unit_lit("foobar", 5, 3);
        assert!(t.had_errors());
        let msg = t.errors[0].message();
        assert!(msg.contains("foobar"));
    }

    // ── Arithmetic type rules ─────────────────────────────────────────────────
    #[test]
    fn add_same_unit_ok() {
        let mut t = tc();
        let kg = NovaType::UnitFloat { dim: Dim::MASS, unit_str: "kg".into() };
        let result = t.infer_binop_arith(BinOp::Add, &kg.clone(), &kg, 0, 0);
        assert!(!t.had_errors());
        assert!(matches!(result, NovaType::UnitFloat { .. }));
    }

    #[test]
    fn add_different_units_fails() {
        let mut t = tc();
        let kg = NovaType::UnitFloat { dim: Dim::MASS,   unit_str: "kg".into() };
        let m  = NovaType::UnitFloat { dim: Dim::LENGTH, unit_str: "m".into()  };
        t.infer_binop_arith(BinOp::Add, &kg, &m, 12, 3);
        assert!(t.had_errors());
        let msg = t.errors[0].message();
        assert!(msg.contains("unit dimension mismatch"));
        assert!(msg.contains("hint"));
    }

    #[test]
    fn mul_units_combines_dims() {
        // kg * m/s² = N (force)
        let mut t = tc();
        let kg  = NovaType::UnitFloat { dim: Dim::MASS,         unit_str: "kg".into()  };
        let acc = NovaType::UnitFloat { dim: Dim::ACCELERATION, unit_str: "m/s".into() };
        let result = t.infer_binop_arith(BinOp::Mul, &kg, &acc, 0, 0);
        assert!(!t.had_errors());
        match result {
            NovaType::UnitFloat { dim, .. } => assert_eq!(dim, Dim::FORCE),
            _ => panic!("expected UnitFloat"),
        }
    }

    #[test]
    fn div_units_subtracts_dims() {
        // m / s = m/s (velocity)
        let mut t = tc();
        let m = NovaType::UnitFloat { dim: Dim::LENGTH, unit_str: "m".into() };
        let s = NovaType::UnitFloat { dim: Dim::TIME,   unit_str: "s".into() };
        let result = t.infer_binop_arith(BinOp::Div, &m, &s, 0, 0);
        assert!(!t.had_errors());
        match result {
            NovaType::UnitFloat { dim, .. } => assert_eq!(dim, Dim::VELOCITY),
            _ => panic!("expected UnitFloat[m/s]"),
        }
    }

    #[test]
    fn comparison_returns_bool() {
        let mut t = tc();
        let f = NovaType::Float;
        let result = t.infer_binop_arith(BinOp::Lt, &f, &f, 0, 0);
        assert_eq!(result, NovaType::Bool);
    }

    // ── Variable lookup ───────────────────────────────────────────────────────
    #[test]
    fn lookup_defined_var() {
        let mut t = tc();
        t.env.bind("mass", NovaType::UnitFloat { dim: Dim::MASS, unit_str: "kg".into() });
        let ty = t.infer_ident("mass", 0, 0);
        assert!(!t.had_errors());
        assert!(matches!(ty, NovaType::UnitFloat { .. }));
    }

    #[test]
    fn lookup_undefined_var_errors() {
        let mut t = tc();
        t.infer_ident("undefined_var", 7, 2);
        assert!(t.had_errors());
        assert!(t.errors[0].message().contains("undefined_var"));
    }

    // ── Call inference ────────────────────────────────────────────────────────
    #[test]
    fn call_transmit_ok() {
        let mut t = tc();
        let callee = t.infer_ident("transmit", 0, 0);
        let ret = t.infer_call(&callee, &[NovaType::Str], 0, 0);
        assert!(!t.had_errors());
        assert_eq!(ret, NovaType::Void);
    }

    #[test]
    fn call_wrong_arity_errors() {
        let mut t = tc();
        let callee = t.infer_ident("ln", 0, 0);
        // ln takes 1 arg; we give 2
        t.infer_call(&callee, &[NovaType::Float, NovaType::Float], 5, 1);
        assert!(t.had_errors());
        assert!(t.errors[0].message().contains("wrong number"));
    }

    // ── Return type checking ──────────────────────────────────────────────────
    #[test]
    fn return_type_mismatch_errors() {
        let mut t = tc();
        let actual   = NovaType::UnitFloat { dim: Dim::VELOCITY, unit_str: "m/s".into() };
        let expected = NovaType::UnitFloat { dim: Dim::MASS,     unit_str: "kg".into() };
        t.check_return(&actual, &expected, 10, 4);
        assert!(t.had_errors());
        assert!(t.errors[0].message().contains("unit dimension mismatch"));
    }

    // ── delta_v end-to-end type inference ─────────────────────────────────────
    #[test]
    fn delta_v_type_chain() {
        // Simulate: Isp[s] * g0[m/s^2] → Float[m/s]  (Tsiolkovsky Δv)
        // Uses Dim constants directly — no unit string parsing involved.
        let mut t = tc();

        let isp_ty = NovaType::UnitFloat { dim: Dim::TIME,         unit_str: "s".into()    };
        let g0_ty  = NovaType::UnitFloat { dim: Dim::ACCELERATION, unit_str: "accel".into() };
        let result = t.infer_binop_arith(BinOp::Mul, &isp_ty, &g0_ty, 0, 0);

        // TIME * ACCELERATION = [0,0,1] + [1,0,-2] = [1,0,-1] = VELOCITY
        assert!(!t.had_errors(), "unexpected errors: {:?}", t.errors);
        match result {
            NovaType::UnitFloat { dim, .. } => assert_eq!(dim, Dim::VELOCITY),
            _ => panic!("expected UnitFloat[velocity], got {:?}", result),
        }
    }

    // ── Scope management ──────────────────────────────────────────────────────
    #[test]
    fn inner_scope_shadows_outer() {
        let mut t = tc();
        t.env.bind("x", NovaType::Int);
        t.env.push_scope();
        t.env.bind("x", NovaType::Float);
        let ty = t.infer_ident("x", 0, 0);
        assert_eq!(t.apply(&ty), NovaType::Float);
        t.env.pop_scope();
        let ty2 = t.infer_ident("x", 0, 0);
        assert_eq!(t.apply(&ty2), NovaType::Int);
    }
}
