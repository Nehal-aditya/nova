//! Custom Unit Registry
//!
//! Handles NOVA's `unit` declaration:
//!   unit parsec    = 3.086e16[m]
//!   unit M_star    = 1.989e30[kg]
//!   unit earth_rad = 6.371e6[m]
//!
//! Custom units are resolved at compile time to a (Dim, scale) pair
//! by looking up their base unit in the SI table and applying the
//! given scale factor.
//!
//! Custom units are scoped to their constellation (module). The registry
//! is built incrementally as the resolver walks the AST.

use crate::dimension::Dim;
use crate::si_table::UnitEntry;
use std::collections::HashMap;

/// A resolved custom unit entry.
#[derive(Debug, Clone)]
pub struct CustomUnit {
    /// The resolved SI dimension.
    pub dim: Dim,
    /// Scale factor relative to the SI base unit.
    pub scale: f64,
    /// The name as declared in NOVA source.
    pub name: String,
    /// The source expression (e.g. "3.086e16[m]"), for error messages.
    pub source_expr: String,
}

/// The custom unit registry for one compilation unit.
#[derive(Debug, Default)]
pub struct CustomUnitRegistry {
    units: HashMap<String, CustomUnit>,
}

impl CustomUnitRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    /// Register a custom unit.
    /// `base_entry` is the resolved SI entry for the base unit.
    /// `factor` is the numeric value in front of the base unit.
    ///
    /// Example: `unit parsec = 3.086e16[m]`
    ///   name       = "parsec"
    ///   factor     = 3.086e16
    ///   base_entry = SI["m"] = { dim: LENGTH, scale: 1.0 }
    ///   → registered as { dim: LENGTH, scale: 3.086e16 }
    pub fn register(
        &mut self,
        name: impl Into<String>,
        factor: f64,
        base_entry: &UnitEntry,
        source_expr: impl Into<String>,
    ) {
        let name = name.into();
        let cu = CustomUnit {
            dim: base_entry.dim,
            scale: factor * base_entry.scale,
            name: name.clone(),
            source_expr: source_expr.into(),
        };
        self.units.insert(name, cu);
    }

    /// Look up a custom unit by name.
    pub fn get(&self, name: &str) -> Option<&CustomUnit> {
        self.units.get(name)
    }

    /// Convert a custom unit to a `UnitEntry` for use in the resolver.
    pub fn to_unit_entry(&self, name: &str) -> Option<UnitEntry> {
        self.units.get(name).map(|cu| UnitEntry {
            dim: cu.dim,
            scale: cu.scale,
            display: "custom",
        })
    }

    pub fn contains(&self, name: &str) -> bool {
        self.units.contains_key(name)
    }

    pub fn iter(&self) -> impl Iterator<Item = (&str, &CustomUnit)> {
        self.units.iter().map(|(k, v)| (k.as_str(), v))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::si_table::build_si_table;
    use crate::dimension::Dim;

    #[test]
    fn register_parsec() {
        let si = build_si_table();
        let mut reg = CustomUnitRegistry::new();
        reg.register("parsec", 3.086e16, &si["m"], "3.086e16[m]");
        let cu = reg.get("parsec").expect("parsec registered");
        assert_eq!(cu.dim, Dim::LENGTH);
        assert!((cu.scale - 3.086e16).abs() < 1e10);
    }

    #[test]
    fn register_solar_luminosity() {
        let si = build_si_table();
        let mut reg = CustomUnitRegistry::new();
        reg.register("L_star", 3.828e26, &si["W"], "3.828e26[W]");
        let cu = reg.get("L_star").expect("L_star registered");
        assert_eq!(cu.dim, Dim::POWER);
    }

    #[test]
    fn to_unit_entry_roundtrip() {
        let si = build_si_table();
        let mut reg = CustomUnitRegistry::new();
        reg.register("earth_rad", 6.371e6, &si["m"], "6.371e6[m]");
        let entry = reg.to_unit_entry("earth_rad").expect("entry");
        assert_eq!(entry.dim, Dim::LENGTH);
        assert!((entry.scale - 6.371e6).abs() < 1.0);
    }

    #[test]
    fn missing_unit_returns_none() {
        let reg = CustomUnitRegistry::new();
        assert!(reg.get("nonexistent").is_none());
    }
}
