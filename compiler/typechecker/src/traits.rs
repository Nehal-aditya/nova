//! Trait System
//!
//! Implements NOVA's trait bounds — borrowed from Rust but simplified for
//! scientific programmers.
//!
//! A `TraitBound` says "type T must implement trait X".
//! A `TraitRegistry` holds all trait definitions and their implementations.
//! The type checker calls `check_bounds` to verify that a type satisfies
//! the traits required by a generic mission.

use crate::types::NovaType;
use std::collections::{HashMap, HashSet};

// ── Trait definition ──────────────────────────────────────────────────────────

/// A single trait bound: "T : TraitName"
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TraitBound {
    pub trait_name: String,
}

impl TraitBound {
    pub fn new(name: impl Into<String>) -> Self {
        TraitBound { trait_name: name.into() }
    }
}

/// A trait definition: name + required method signatures (as strings for now).
#[derive(Debug, Clone)]
pub struct TraitDef {
    pub name:    String,
    /// Method signatures (simplified: just names for Phase 7).
    pub methods: Vec<String>,
}

// ── Trait registry ────────────────────────────────────────────────────────────

/// Maps trait names → definitions, and type names → implemented traits.
#[derive(Debug, Default)]
pub struct TraitRegistry {
    /// trait name → definition
    traits: HashMap<String, TraitDef>,
    /// type name → set of trait names it implements
    impls: HashMap<String, HashSet<String>>,
}

impl TraitRegistry {
    pub fn new() -> Self {
        let mut r = TraitRegistry::default();
        r.register_builtins();
        r
    }

    /// Register a trait definition.
    pub fn define_trait(&mut self, def: TraitDef) {
        self.traits.insert(def.name.clone(), def);
    }

    /// Declare that a type implements a trait.
    pub fn add_impl(&mut self, type_name: impl Into<String>, trait_name: impl Into<String>) {
        self.impls
            .entry(type_name.into())
            .or_default()
            .insert(trait_name.into());
    }

    /// Check whether a type satisfies a set of trait bounds.
    /// Returns Ok(()) if all bounds are satisfied, Err with details otherwise.
    pub fn check_bounds(
        &self,
        ty: &NovaType,
        bounds: &[TraitBound],
        line: u32,
        col: u32,
    ) -> Result<(), BoundError> {
        for bound in bounds {
            self.check_one_bound(ty, bound, line, col)?;
        }
        Ok(())
    }

    fn check_one_bound(
        &self,
        ty: &NovaType,
        bound: &TraitBound,
        line: u32,
        col: u32,
    ) -> Result<(), BoundError> {
        let type_name = self.type_name(ty);

        // Built-in trait satisfaction rules
        match bound.trait_name.as_str() {
            // All numeric types satisfy Numeric
            "Numeric" => {
                if ty.is_numeric() { return Ok(()); }
            }
            // All types satisfy Display (best-effort)
            "Display" => return Ok(()),
            // All numeric types satisfy Add, Sub, Mul, Div, Neg
            "Add" | "Sub" | "Mul" | "Div" | "Neg" => {
                if ty.is_numeric() { return Ok(()); }
            }
            // Float and UnitFloat satisfy Transcendental (ln, exp, sqrt, etc.)
            "Transcendental" => {
                if matches!(ty, NovaType::Float | NovaType::UnitFloat { .. }) {
                    return Ok(());
                }
            }
            _ => {}
        }

        // Check the impl registry
        if let Some(trait_set) = self.impls.get(&type_name) {
            if trait_set.contains(&bound.trait_name) {
                return Ok(());
            }
        }

        Err(BoundError {
            type_name,
            trait_name: bound.trait_name.clone(),
            message: format!(
                "type `{}` does not implement trait `{}`\n  \
                 hint: add `impl {} for {}` or derive it",
                ty, bound.trait_name, bound.trait_name, ty
            ),
            line,
            col,
        })
    }

    /// Resolve a NovaType to a string name for registry lookup.
    fn type_name(&self, ty: &NovaType) -> String {
        match ty {
            NovaType::Int                    => "Int".into(),
            NovaType::Float                  => "Float".into(),
            NovaType::UnitFloat { .. }       => "Float".into(), // unit floats are Floats
            NovaType::Bool                   => "Bool".into(),
            NovaType::Str                    => "Str".into(),
            NovaType::Char                   => "Char".into(),
            NovaType::Tensor { .. }          => "Tensor".into(),
            NovaType::Array(_)               => "Array".into(),
            NovaType::Named { name, .. }     => name.clone(),
            _                                => "<unknown>".into(),
        }
    }

    /// Pre-register built-in trait definitions.
    fn register_builtins(&mut self) {
        let builtins = [
            ("Numeric",       vec!["zero", "one"]),
            ("Display",       vec!["display"]),
            ("Add",           vec!["add"]),
            ("Sub",           vec!["sub"]),
            ("Mul",           vec!["mul"]),
            ("Div",           vec!["div"]),
            ("Neg",           vec!["neg"]),
            ("Transcendental",vec!["ln", "exp", "sqrt", "sin", "cos"]),
            ("Measurable",    vec!["measure"]),
            ("Displayable",   vec!["display", "display_with_unit"]),
            ("Optimiser",     vec!["step", "reset"]),
        ];
        for (name, methods) in builtins {
            self.define_trait(TraitDef {
                name:    name.into(),
                methods: methods.iter().map(|&m| m.into()).collect(),
            });
        }

        // Built-in type → trait impls
        for ty in &["Int", "Float"] {
            for tr in &["Numeric", "Add", "Sub", "Mul", "Div", "Neg", "Display"] {
                self.add_impl(*ty, *tr);
            }
        }
        self.add_impl("Float", "Transcendental");
        self.add_impl("Str", "Display");
        self.add_impl("Bool", "Display");
    }
}

// ── Bound error ───────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct BoundError {
    pub type_name:  String,
    pub trait_name: String,
    pub message:    String,
    pub line:       u32,
    pub col:        u32,
}

impl std::fmt::Display for BoundError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{}: {}", self.line, self.col, self.message)
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────
#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::NovaType;
    use nova_unit_resolver::dimension::Dim;

    fn reg() -> TraitRegistry { TraitRegistry::new() }

    #[test]
    fn float_satisfies_numeric() {
        let r = reg();
        assert!(r.check_bounds(&NovaType::Float, &[TraitBound::new("Numeric")], 0, 0).is_ok());
    }

    #[test]
    fn int_satisfies_add() {
        let r = reg();
        assert!(r.check_bounds(&NovaType::Int, &[TraitBound::new("Add")], 0, 0).is_ok());
    }

    #[test]
    fn unit_float_satisfies_numeric() {
        let r = reg();
        let uf = NovaType::UnitFloat { dim: Dim::MASS, unit_str: "kg".into() };
        assert!(r.check_bounds(&uf, &[TraitBound::new("Numeric")], 0, 0).is_ok());
    }

    #[test]
    fn unit_float_satisfies_transcendental() {
        let r = reg();
        let uf = NovaType::UnitFloat { dim: Dim::TIME, unit_str: "s".into() };
        assert!(r.check_bounds(&uf, &[TraitBound::new("Transcendental")], 0, 0).is_ok());
    }

    #[test]
    fn bool_does_not_satisfy_numeric() {
        let r = reg();
        assert!(r.check_bounds(&NovaType::Bool, &[TraitBound::new("Numeric")], 0, 0).is_err());
    }

    #[test]
    fn str_does_not_satisfy_add() {
        let r = reg();
        // Str doesn't satisfy Add (no string concatenation with + in NOVA)
        let err = r.check_bounds(&NovaType::Str, &[TraitBound::new("Add")], 5, 10);
        assert!(err.is_err());
        assert_eq!(err.unwrap_err().line, 5);
    }

    #[test]
    fn multiple_bounds_all_satisfied() {
        let r = reg();
        let bounds = vec![TraitBound::new("Numeric"), TraitBound::new("Add"), TraitBound::new("Display")];
        assert!(r.check_bounds(&NovaType::Float, &bounds, 0, 0).is_ok());
    }

    #[test]
    fn custom_impl_registered() {
        let mut r = reg();
        r.add_impl("Star", "Measurable");
        let ty = NovaType::Named { name: "Star".into(), type_args: vec![] };
        assert!(r.check_bounds(&ty, &[TraitBound::new("Measurable")], 0, 0).is_ok());
    }

    #[test]
    fn bound_error_has_hint() {
        let r = reg();
        let ty = NovaType::Named { name: "Nebula".into(), type_args: vec![] };
        let err = r.check_bounds(&ty, &[TraitBound::new("Optimiser")], 0, 0).unwrap_err();
        assert!(err.message.contains("hint"));
        assert!(err.message.contains("Optimiser"));
    }
}
