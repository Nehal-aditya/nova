//! NOVA Type Algebra
//!
//! `NovaType` is the core type representation used throughout the type checker.
//! It covers every type that can appear in a NOVA program.

use nova_unit_resolver::dimension::Dim;
use std::fmt;
use std::sync::atomic::{AtomicU32, Ordering};

// ── Type variable IDs ─────────────────────────────────────────────────────────

static NEXT_VAR_ID: AtomicU32 = AtomicU32::new(0);

/// A fresh, globally unique type variable.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TypeVar(pub u32);

impl TypeVar {
    pub fn fresh() -> Self {
        TypeVar(NEXT_VAR_ID.fetch_add(1, Ordering::Relaxed))
    }
}

impl fmt::Display for TypeVar {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Display as τ0, τ1, τ2 ... for readable error messages
        write!(f, "τ{}", self.0)
    }
}

// ── Tensor shape ──────────────────────────────────────────────────────────────

/// Tensor shape: either a concrete size, a symbolic name, or unknown.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TensorShape {
    /// Known static size: Tensor[Float, 1024]
    Known(Vec<usize>),
    /// Rank known, sizes symbolic: Tensor[Float, (n, m)]
    Symbolic(Vec<String>),
    /// Only rank is known: Tensor[Float, 2] (rank=2, sizes unknown)
    Rank(usize),
    /// Completely unknown shape
    Unknown,
}

impl fmt::Display for TensorShape {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TensorShape::Known(dims) => {
                let s: Vec<_> = dims.iter().map(|d| d.to_string()).collect();
                write!(f, "({})", s.join(", "))
            }
            TensorShape::Symbolic(names) => write!(f, "({})", names.join(", ")),
            TensorShape::Rank(r)         => write!(f, "rank={}", r),
            TensorShape::Unknown         => write!(f, "?"),
        }
    }
}

// ── The type algebra ──────────────────────────────────────────────────────────

/// Every type in NOVA.
#[derive(Debug, Clone, PartialEq)]
pub enum NovaType {

    // ── Unification variable ────────────────────────────────────────────────
    /// An unknown type that will be resolved by unification.
    Var(TypeVar),

    // ── Primitives ──────────────────────────────────────────────────────────
    /// `Int` — machine integer (64-bit signed)
    Int,
    /// `Float` — IEEE-754 double, no unit annotation
    Float,
    /// `Float[unit]` — dimensioned float. `dim` is the SI dimension vector.
    ///  `unit_str` is the original string (e.g. "m/s") for error messages.
    UnitFloat {
        dim: Dim,
        unit_str: String,
    },
    /// `Bool`
    Bool,
    /// `Str`
    Str,
    /// `Char`
    Char,
    /// `Void` — no value
    Void,
    /// `Never` — diverging type (panic, infinite loop)
    Never,

    // ── Tensor types ────────────────────────────────────────────────────────
    /// `Tensor[elem_type, shape]`
    /// elem_type may itself be a UnitFloat (e.g. Tensor[Float[eV], 1024])
    Tensor {
        elem: Box<NovaType>,
        shape: TensorShape,
    },

    // ── Collection types ────────────────────────────────────────────────────
    /// `Array[T]`
    Array(Box<NovaType>),
    /// `Wave[T]` — lazy stream
    Wave(Box<NovaType>),
    /// `Option[T]`
    Option(Box<NovaType>),
    /// `Result[T, E]`
    Result(Box<NovaType>, Box<NovaType>),

    // ── Function type ────────────────────────────────────────────────────────
    /// `(param_types) -> return_type`
    Function(FunctionType),

    // ── Named types ─────────────────────────────────────────────────────────
    /// A named struct, enum, or user-defined type.
    Named {
        name: String,
        /// Type arguments for generic named types.
        type_args: Vec<NovaType>,
    },

    // ── Tuple type (anonymous struct) ────────────────────────────────────────
    Tuple(Vec<NovaType>),

    // ── Row type (for anonymous struct literals { a: T, b: U }) ─────────────
    Row {
        fields: Vec<(String, NovaType)>,
    },
}

/// A NOVA function type: parameter types + return type.
#[derive(Debug, Clone, PartialEq)]
pub struct FunctionType {
    pub params:  Vec<NovaType>,
    pub ret:     Box<NovaType>,
    /// Whether this function is marked `parallel`.
    pub parallel: bool,
}

/// A polymorphic type scheme: ∀ type_vars. body_type
/// Used for let-polymorphism (functions can be generic).
#[derive(Debug, Clone)]
pub struct TypeScheme {
    /// Universally quantified type variables.
    pub quantified: Vec<TypeVar>,
    /// The body type (may contain `quantified` variables).
    pub body: NovaType,
}

impl TypeScheme {
    /// A monomorphic scheme — no quantified variables.
    pub fn mono(t: NovaType) -> Self {
        TypeScheme { quantified: vec![], body: t }
    }
}

// ── Display ───────────────────────────────────────────────────────────────────

impl fmt::Display for NovaType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            NovaType::Var(v)           => write!(f, "{}", v),
            NovaType::Int              => write!(f, "Int"),
            NovaType::Float            => write!(f, "Float"),
            NovaType::UnitFloat { unit_str, .. } => write!(f, "Float[{}]", unit_str),
            NovaType::Bool             => write!(f, "Bool"),
            NovaType::Str              => write!(f, "Str"),
            NovaType::Char             => write!(f, "Char"),
            NovaType::Void             => write!(f, "Void"),
            NovaType::Never            => write!(f, "Never"),
            NovaType::Tensor { elem, shape } =>
                write!(f, "Tensor[{}, {}]", elem, shape),
            NovaType::Array(t)         => write!(f, "Array[{}]", t),
            NovaType::Wave(t)          => write!(f, "Wave[{}]", t),
            NovaType::Option(t)        => write!(f, "Option[{}]", t),
            NovaType::Result(t, e)     => write!(f, "Result[{}, {}]", t, e),
            NovaType::Function(ft)     => {
                let params: Vec<_> = ft.params.iter().map(|p| p.to_string()).collect();
                write!(f, "({}) -> {}", params.join(", "), ft.ret)
            }
            NovaType::Named { name, type_args } => {
                if type_args.is_empty() {
                    write!(f, "{}", name)
                } else {
                    let args: Vec<_> = type_args.iter().map(|a| a.to_string()).collect();
                    write!(f, "{}[{}]", name, args.join(", "))
                }
            }
            NovaType::Tuple(ts) => {
                let parts: Vec<_> = ts.iter().map(|t| t.to_string()).collect();
                write!(f, "({})", parts.join(", "))
            }
            NovaType::Row { fields } => {
                let parts: Vec<_> = fields.iter()
                    .map(|(n, t)| format!("{}: {}", n, t))
                    .collect();
                write!(f, "{{ {} }}", parts.join(", "))
            }
        }
    }
}

// ── Helpers ───────────────────────────────────────────────────────────────────

impl NovaType {
    /// Return true if this is a numeric type (Int, Float, UnitFloat).
    pub fn is_numeric(&self) -> bool {
        matches!(self, NovaType::Int | NovaType::Float | NovaType::UnitFloat { .. })
    }

    /// Return true if this is a unit-annotated float.
    pub fn is_unit_float(&self) -> bool {
        matches!(self, NovaType::UnitFloat { .. })
    }

    /// Return true if this type contains any unresolved type variables.
    pub fn is_concrete(&self) -> bool {
        !self.contains_var()
    }

    /// Return true if this type contains a type variable anywhere.
    pub fn contains_var(&self) -> bool {
        match self {
            NovaType::Var(_)           => true,
            NovaType::UnitFloat { .. }
            | NovaType::Int | NovaType::Float | NovaType::Bool
            | NovaType::Str | NovaType::Char | NovaType::Void
            | NovaType::Never          => false,
            NovaType::Tensor { elem, .. } => elem.contains_var(),
            NovaType::Array(t) | NovaType::Wave(t) | NovaType::Option(t) => t.contains_var(),
            NovaType::Result(t, e)     => t.contains_var() || e.contains_var(),
            NovaType::Function(ft)     =>
                ft.params.iter().any(|p| p.contains_var()) || ft.ret.contains_var(),
            NovaType::Named { type_args, .. } =>
                type_args.iter().any(|a| a.contains_var()),
            NovaType::Tuple(ts)        => ts.iter().any(|t| t.contains_var()),
            NovaType::Row { fields }   => fields.iter().any(|(_, t)| t.contains_var()),
        }
    }

    /// Collect all type variables contained in this type.
    pub fn free_vars(&self) -> Vec<TypeVar> {
        let mut vars = Vec::new();
        self.collect_vars(&mut vars);
        vars
    }

    fn collect_vars(&self, out: &mut Vec<TypeVar>) {
        match self {
            NovaType::Var(v) => { if !out.contains(v) { out.push(*v); } }
            NovaType::Tensor { elem, .. } => elem.collect_vars(out),
            NovaType::Array(t) | NovaType::Wave(t) | NovaType::Option(t) => t.collect_vars(out),
            NovaType::Result(t, e) => { t.collect_vars(out); e.collect_vars(out); }
            NovaType::Function(ft) => {
                for p in &ft.params { p.collect_vars(out); }
                ft.ret.collect_vars(out);
            }
            NovaType::Named { type_args, .. } => {
                for a in type_args { a.collect_vars(out); }
            }
            NovaType::Tuple(ts) => { for t in ts { t.collect_vars(out); } }
            NovaType::Row { fields } => { for (_, t) in fields { t.collect_vars(out); } }
            _ => {}
        }
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────
#[cfg(test)]
mod tests {
    use super::*;
    use nova_unit_resolver::dimension::Dim;

    #[test]
    fn fresh_vars_are_unique() {
        let a = TypeVar::fresh();
        let b = TypeVar::fresh();
        assert_ne!(a, b);
    }

    #[test]
    fn unit_float_display() {
        let t = NovaType::UnitFloat { dim: Dim::VELOCITY, unit_str: "m/s".into() };
        assert_eq!(t.to_string(), "Float[m/s]");
    }

    #[test]
    fn tensor_display() {
        let elem = NovaType::UnitFloat { dim: Dim::ENERGY, unit_str: "eV".into() };
        let t = NovaType::Tensor { elem: Box::new(elem), shape: TensorShape::Rank(1) };
        assert_eq!(t.to_string(), "Tensor[Float[eV], rank=1]");
    }

    #[test]
    fn function_display() {
        let t = NovaType::Function(FunctionType {
            params: vec![NovaType::Float, NovaType::Float],
            ret: Box::new(NovaType::UnitFloat { dim: Dim::VELOCITY, unit_str: "m/s".into() }),
            parallel: false,
        });
        assert_eq!(t.to_string(), "(Float, Float) -> Float[m/s]");
    }

    #[test]
    fn contains_var_true_for_var() {
        assert!(NovaType::Var(TypeVar::fresh()).contains_var());
    }

    #[test]
    fn contains_var_false_for_concrete() {
        assert!(!NovaType::Float.contains_var());
        assert!(!NovaType::UnitFloat { dim: Dim::MASS, unit_str: "kg".into() }.contains_var());
    }

    #[test]
    fn is_numeric() {
        assert!(NovaType::Int.is_numeric());
        assert!(NovaType::Float.is_numeric());
        assert!(NovaType::UnitFloat { dim: Dim::MASS, unit_str: "kg".into() }.is_numeric());
        assert!(!NovaType::Bool.is_numeric());
        assert!(!NovaType::Str.is_numeric());
    }

    #[test]
    fn free_vars_collects_all() {
        let v1 = TypeVar::fresh();
        let v2 = TypeVar::fresh();
        let t = NovaType::Tuple(vec![NovaType::Var(v1), NovaType::Var(v2)]);
        let fv = t.free_vars();
        assert_eq!(fv.len(), 2);
        assert!(fv.contains(&v1));
        assert!(fv.contains(&v2));
    }

    #[test]
    fn option_display() {
        let t = NovaType::Option(Box::new(NovaType::Float));
        assert_eq!(t.to_string(), "Option[Float]");
    }

    #[test]
    fn result_display() {
        let t = NovaType::Result(
            Box::new(NovaType::UnitFloat { dim: Dim::VELOCITY, unit_str: "m/s".into() }),
            Box::new(NovaType::Named { name: "UnitError".into(), type_args: vec![] }),
        );
        assert_eq!(t.to_string(), "Result[Float[m/s], UnitError]");
    }
}
