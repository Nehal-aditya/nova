//! Unification Engine
//!
//! Implements Robinson unification for `NovaType`, extended with:
//!   - Unit dimension equality (UnitFloat unifies only if dims match)
//!   - Tensor shape compatibility (rank must match; sizes unified if known)
//!   - Occurs check (prevents infinite types)
//!
//! The substitution map (`Subst`) is the mutable state that grows during
//! unification. `apply` applies a substitution to a type, chasing chains
//! to the final resolved type.

use crate::types::{NovaType, TypeVar, FunctionType, TensorShape};
use nova_unit_resolver::dimension::Dim;
use std::collections::HashMap;
use std::fmt;

// ── UnifyError ────────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct UnifyError {
    pub message: String,
    pub line: u32,
    pub col:  u32,
}

impl UnifyError {
    pub fn new(msg: impl Into<String>) -> Self {
        UnifyError { message: msg.into(), line: 0, col: 0 }
    }
    pub fn at(msg: impl Into<String>, line: u32, col: u32) -> Self {
        UnifyError { message: msg.into(), line, col }
    }
}

impl fmt::Display for UnifyError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.line > 0 {
            write!(f, "{}:{}: type error: {}", self.line, self.col, self.message)
        } else {
            write!(f, "type error: {}", self.message)
        }
    }
}

// ── Substitution ──────────────────────────────────────────────────────────────

/// A mapping from type variables to types.
/// Grows monotonically during unification.
#[derive(Debug, Default, Clone)]
pub struct Subst {
    map: HashMap<TypeVar, NovaType>,
}

impl Subst {
    pub fn new() -> Self { Self::default() }

    /// Bind a type variable to a type.
    /// Panics if the variable is already bound to a different type (a bug).
    pub fn bind(&mut self, var: TypeVar, ty: NovaType) {
        self.map.insert(var, ty);
    }

    /// Look up a type variable — returns None if unbound.
    pub fn lookup(&self, var: TypeVar) -> Option<&NovaType> {
        self.map.get(&var)
    }

    /// Apply the substitution to a type: chase all variable bindings.
    pub fn apply(&self, ty: &NovaType) -> NovaType {
        match ty {
            NovaType::Var(v) => {
                match self.lookup(*v) {
                    Some(bound) => {
                        let chased = self.apply(bound);
                        chased
                    }
                    None => ty.clone(),
                }
            }
            NovaType::Tensor { elem, shape } =>
                NovaType::Tensor { elem: Box::new(self.apply(elem)), shape: shape.clone() },
            NovaType::Array(t)   => NovaType::Array(Box::new(self.apply(t))),
            NovaType::Wave(t)    => NovaType::Wave(Box::new(self.apply(t))),
            NovaType::Option(t)  => NovaType::Option(Box::new(self.apply(t))),
            NovaType::Result(t, e) =>
                NovaType::Result(Box::new(self.apply(t)), Box::new(self.apply(e))),
            NovaType::Function(ft) => NovaType::Function(FunctionType {
                params:   ft.params.iter().map(|p| self.apply(p)).collect(),
                ret:      Box::new(self.apply(&ft.ret)),
                parallel: ft.parallel,
            }),
            NovaType::Named { name, type_args } => NovaType::Named {
                name: name.clone(),
                type_args: type_args.iter().map(|a| self.apply(a)).collect(),
            },
            NovaType::Tuple(ts) =>
                NovaType::Tuple(ts.iter().map(|t| self.apply(t)).collect()),
            NovaType::Row { fields } => NovaType::Row {
                fields: fields.iter().map(|(n, t)| (n.clone(), self.apply(t))).collect(),
            },
            // Ground types: no substitution needed
            _ => ty.clone(),
        }
    }

    pub fn len(&self) -> usize { self.map.len() }
    pub fn is_empty(&self) -> bool { self.map.is_empty() }
}

// ── Occurs check ──────────────────────────────────────────────────────────────

/// Return true if `var` appears anywhere in `ty` under `subst`.
/// Used to prevent infinite types (e.g. a = List[a]).
fn occurs_in(var: TypeVar, ty: &NovaType, subst: &Subst) -> bool {
    match ty {
        NovaType::Var(v) => {
            if *v == var { return true; }
            // Chase the binding
            if let Some(bound) = subst.lookup(*v) {
                return occurs_in(var, bound, subst);
            }
            false
        }
        NovaType::Tensor { elem, .. } => occurs_in(var, elem, subst),
        NovaType::Array(t) | NovaType::Wave(t) | NovaType::Option(t) =>
            occurs_in(var, t, subst),
        NovaType::Result(t, e) =>
            occurs_in(var, t, subst) || occurs_in(var, e, subst),
        NovaType::Function(ft) =>
            ft.params.iter().any(|p| occurs_in(var, p, subst)) ||
            occurs_in(var, &ft.ret, subst),
        NovaType::Named { type_args, .. } =>
            type_args.iter().any(|a| occurs_in(var, a, subst)),
        NovaType::Tuple(ts) => ts.iter().any(|t| occurs_in(var, t, subst)),
        NovaType::Row { fields } => fields.iter().any(|(_, t)| occurs_in(var, t, subst)),
        _ => false,
    }
}

// ── Shape unification ─────────────────────────────────────────────────────────

fn unify_shapes(
    lhs: &TensorShape,
    rhs: &TensorShape,
    line: u32, col: u32,
) -> Result<TensorShape, UnifyError> {
    use TensorShape::*;
    match (lhs, rhs) {
        (Unknown, other) | (other, Unknown) => Ok(other.clone()),
        (Rank(a), Rank(b)) => {
            if a == b { Ok(Rank(*a)) }
            else {
                Err(UnifyError::at(
                    format!("tensor rank mismatch: rank {} vs rank {}", a, b),
                    line, col,
                ))
            }
        }
        (Known(a), Known(b)) => {
            if a == b { Ok(Known(a.clone())) }
            else {
                Err(UnifyError::at(
                    format!("tensor shape mismatch: {:?} vs {:?}", a, b),
                    line, col,
                ))
            }
        }
        (Rank(r), Known(dims)) | (Known(dims), Rank(r)) => {
            if *r == dims.len() { Ok(Known(dims.clone())) }
            else {
                Err(UnifyError::at(
                    format!("tensor rank {} does not match shape {:?}", r, dims),
                    line, col,
                ))
            }
        }
        (Symbolic(a), Symbolic(b)) => {
            if a.len() == b.len() { Ok(Symbolic(a.clone())) }
            else {
                Err(UnifyError::at("symbolic tensor shape rank mismatch", line, col))
            }
        }
        // Any other combination: accept the more specific one
        (Rank(r), Symbolic(s)) | (Symbolic(s), Rank(r)) => {
            if *r == s.len() { Ok(Symbolic(s.clone())) }
            else { Err(UnifyError::at("tensor rank mismatch", line, col)) }
        }
        (Known(dims), Symbolic(names)) | (Symbolic(names), Known(dims)) => {
            if dims.len() == names.len() { Ok(Known(dims.clone())) }
            else { Err(UnifyError::at("tensor rank mismatch", line, col)) }
        }
    }
}

// ── Unification ───────────────────────────────────────────────────────────────

/// Unify two types, extending `subst` with any new variable bindings.
/// Returns an error if unification fails.
pub fn unify(
    lhs: &NovaType,
    rhs: &NovaType,
    subst: &mut Subst,
    line: u32,
    col: u32,
) -> Result<(), UnifyError> {
    // Apply current substitution before comparing
    let lhs = subst.apply(lhs);
    let rhs = subst.apply(rhs);

    match (&lhs, &rhs) {
        // Two variables: bind one to the other
        (NovaType::Var(a), NovaType::Var(b)) if a == b => Ok(()),
        (NovaType::Var(v), ty) | (ty, NovaType::Var(v)) => {
            if occurs_in(*v, ty, subst) {
                return Err(UnifyError::at(
                    format!("occurs check failed: {} appears in {}", v, ty),
                    line, col,
                ));
            }
            subst.bind(*v, ty.clone());
            Ok(())
        }

        // Primitives: must be identical
        (NovaType::Int,   NovaType::Int)   => Ok(()),
        (NovaType::Float, NovaType::Float) => Ok(()),
        (NovaType::Bool,  NovaType::Bool)  => Ok(()),
        (NovaType::Str,   NovaType::Str)   => Ok(()),
        (NovaType::Char,  NovaType::Char)  => Ok(()),
        (NovaType::Void,  NovaType::Void)  => Ok(()),
        (NovaType::Never, _) => Ok(()), // Never unifies with anything
        (_, NovaType::Never) => Ok(()),

        // Float unifies with UnitFloat (annotation is being inferred)
        (NovaType::Float, NovaType::UnitFloat { .. }) => Ok(()),
        (NovaType::UnitFloat { .. }, NovaType::Float) => Ok(()),

        // UnitFloat: dimensions must match
        (
            NovaType::UnitFloat { dim: d1, unit_str: u1 },
            NovaType::UnitFloat { dim: d2, unit_str: u2 },
        ) => {
            if d1 == d2 {
                Ok(())
            } else {
                Err(UnifyError::at(
                    format!(
                        "unit dimension mismatch: Float[{}] ({}) vs Float[{}] ({})\n  \
                         hint: check that both sides of this operation have compatible units",
                        u1, dim_name(*d1), u2, dim_name(*d2)
                    ),
                    line, col,
                ))
            }
        }

        // Tensor: unify element type and shape
        (
            NovaType::Tensor { elem: e1, shape: s1 },
            NovaType::Tensor { elem: e2, shape: s2 },
        ) => {
            unify(e1, e2, subst, line, col)?;
            unify_shapes(s1, s2, line, col)?;
            Ok(())
        }

        // Collections
        (NovaType::Array(a),   NovaType::Array(b))   => unify(a, b, subst, line, col),
        (NovaType::Wave(a),    NovaType::Wave(b))     => unify(a, b, subst, line, col),
        (NovaType::Option(a),  NovaType::Option(b))   => unify(a, b, subst, line, col),
        (NovaType::Result(a1, e1), NovaType::Result(a2, e2)) => {
            unify(a1, a2, subst, line, col)?;
            unify(e1, e2, subst, line, col)
        }

        // Function types: arity + param types + return type
        (NovaType::Function(f1), NovaType::Function(f2)) => {
            if f1.params.len() != f2.params.len() {
                return Err(UnifyError::at(
                    format!(
                        "function arity mismatch: {} params vs {} params",
                        f1.params.len(), f2.params.len()
                    ),
                    line, col,
                ));
            }
            for (p1, p2) in f1.params.iter().zip(f2.params.iter()) {
                unify(p1, p2, subst, line, col)?;
            }
            unify(&f1.ret, &f2.ret, subst, line, col)
        }

        // Named types: name + type args
        (
            NovaType::Named { name: n1, type_args: a1 },
            NovaType::Named { name: n2, type_args: a2 },
        ) => {
            if n1 != n2 {
                return Err(UnifyError::at(
                    format!("type mismatch: {} vs {}", n1, n2),
                    line, col,
                ));
            }
            if a1.len() != a2.len() {
                return Err(UnifyError::at(
                    format!("type argument count mismatch for {}", n1),
                    line, col,
                ));
            }
            for (t1, t2) in a1.iter().zip(a2.iter()) {
                unify(t1, t2, subst, line, col)?;
            }
            Ok(())
        }

        // Tuples
        (NovaType::Tuple(ts1), NovaType::Tuple(ts2)) => {
            if ts1.len() != ts2.len() {
                return Err(UnifyError::at(
                    format!("tuple arity mismatch: {} vs {}", ts1.len(), ts2.len()),
                    line, col,
                ));
            }
            for (t1, t2) in ts1.iter().zip(ts2.iter()) {
                unify(t1, t2, subst, line, col)?;
            }
            Ok(())
        }

        // Mismatch — produce a clear error
        _ => Err(UnifyError::at(
            format!(
                "type mismatch: expected {}, found {}",
                lhs, rhs
            ),
            line, col,
        )),
    }
}

// ── Helper ─────────────────────────────────────────────────────────────────────
fn dim_name(d: Dim) -> String { d.name() }

// ── Tests ─────────────────────────────────────────────────────────────────────
#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{NovaType, TypeVar, TensorShape};
    use nova_unit_resolver::dimension::Dim;

    fn subst() -> Subst { Subst::new() }

    #[test]
    fn unify_int_int_ok() {
        let mut s = subst();
        assert!(unify(&NovaType::Int, &NovaType::Int, &mut s, 0, 0).is_ok());
    }

    #[test]
    fn unify_var_to_int() {
        let mut s = subst();
        let v = TypeVar::fresh();
        unify(&NovaType::Var(v), &NovaType::Int, &mut s, 0, 0).unwrap();
        assert_eq!(s.apply(&NovaType::Var(v)), NovaType::Int);
    }

    #[test]
    fn unify_int_float_fails() {
        let mut s = subst();
        assert!(unify(&NovaType::Int, &NovaType::Float, &mut s, 0, 0).is_err());
    }

    #[test]
    fn unify_float_with_unit_float_ok() {
        // Float unifies with Float[m/s] — annotation is being inferred
        let mut s = subst();
        let uf = NovaType::UnitFloat { dim: Dim::VELOCITY, unit_str: "m/s".into() };
        assert!(unify(&NovaType::Float, &uf, &mut s, 0, 0).is_ok());
    }

    #[test]
    fn unify_same_unit_floats_ok() {
        let mut s = subst();
        let a = NovaType::UnitFloat { dim: Dim::MASS, unit_str: "kg".into() };
        let b = NovaType::UnitFloat { dim: Dim::MASS, unit_str: "kg".into() };
        assert!(unify(&a, &b, &mut s, 0, 0).is_ok());
    }

    #[test]
    fn unify_kg_plus_m_fails_with_hint() {
        let mut s = subst();
        let kg = NovaType::UnitFloat { dim: Dim::MASS,   unit_str: "kg".into() };
        let m  = NovaType::UnitFloat { dim: Dim::LENGTH, unit_str: "m".into()  };
        let err = unify(&kg, &m, &mut s, 12, 3).unwrap_err();
        assert!(err.message.contains("unit dimension mismatch"));
        assert!(err.message.contains("hint"));
        assert_eq!(err.line, 12);
        assert_eq!(err.col,   3);
    }

    #[test]
    fn unify_tensor_same_shape_ok() {
        let mut s = subst();
        let a = NovaType::Tensor {
            elem: Box::new(NovaType::Float),
            shape: TensorShape::Rank(2),
        };
        let b = NovaType::Tensor {
            elem: Box::new(NovaType::Float),
            shape: TensorShape::Rank(2),
        };
        assert!(unify(&a, &b, &mut s, 0, 0).is_ok());
    }

    #[test]
    fn unify_tensor_rank_mismatch_fails() {
        let mut s = subst();
        let a = NovaType::Tensor { elem: Box::new(NovaType::Float), shape: TensorShape::Rank(2) };
        let b = NovaType::Tensor { elem: Box::new(NovaType::Float), shape: TensorShape::Rank(3) };
        assert!(unify(&a, &b, &mut s, 0, 0).is_err());
    }

    #[test]
    fn unify_function_types_ok() {
        let mut s = subst();
        let a = NovaType::Function(FunctionType {
            params: vec![NovaType::Float, NovaType::Float],
            ret: Box::new(NovaType::Float),
            parallel: false,
        });
        let b = NovaType::Function(FunctionType {
            params: vec![NovaType::Float, NovaType::Float],
            ret: Box::new(NovaType::Float),
            parallel: false,
        });
        assert!(unify(&a, &b, &mut s, 0, 0).is_ok());
    }

    #[test]
    fn unify_function_arity_mismatch_fails() {
        let mut s = subst();
        let a = NovaType::Function(FunctionType {
            params: vec![NovaType::Float],
            ret: Box::new(NovaType::Float),
            parallel: false,
        });
        let b = NovaType::Function(FunctionType {
            params: vec![NovaType::Float, NovaType::Float],
            ret: Box::new(NovaType::Float),
            parallel: false,
        });
        assert!(unify(&a, &b, &mut s, 0, 0).is_err());
    }

    #[test]
    fn occurs_check_prevents_infinite_type() {
        // Unifying a = Array[a] should fail
        let mut s = subst();
        let v = TypeVar::fresh();
        let recursive = NovaType::Array(Box::new(NovaType::Var(v)));
        assert!(unify(&NovaType::Var(v), &recursive, &mut s, 0, 0).is_err());
    }

    #[test]
    fn never_unifies_with_anything() {
        let mut s = subst();
        assert!(unify(&NovaType::Never, &NovaType::Int, &mut s, 0, 0).is_ok());
        assert!(unify(&NovaType::Float, &NovaType::Never, &mut s, 0, 0).is_ok());
    }

    #[test]
    fn subst_apply_chains() {
        // a -> b -> Int  should resolve to Int
        let mut s = Subst::new();
        let a = TypeVar::fresh();
        let b = TypeVar::fresh();
        s.bind(a, NovaType::Var(b));
        s.bind(b, NovaType::Int);
        assert_eq!(s.apply(&NovaType::Var(a)), NovaType::Int);
    }

    #[test]
    fn unify_named_types_different_names_fails() {
        let mut s = subst();
        let a = NovaType::Named { name: "Star".into(), type_args: vec![] };
        let b = NovaType::Named { name: "Galaxy".into(), type_args: vec![] };
        assert!(unify(&a, &b, &mut s, 0, 0).is_err());
    }
}
