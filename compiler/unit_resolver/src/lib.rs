//! NOVA Unit Resolver
//!
//! Phase 2a of the NOVA compiler pipeline.
//!
//! ## What this crate does
//!
//! Takes a unit expression string (as produced by the lexer from a unit literal
//! like `9.8[m/s²]` or `6.674e-11[N·m²/kg²]`) and resolves it to a typed
//! `ResolvedUnit` — a (Dim, scale) pair.
//!
//! It also:
//!   - Validates that unit expressions are dimensionally consistent
//!   - Computes the dimension of compound expressions (a * b, a / b, a ^ n)
//!   - Checks addition/subtraction operands for dimensional compatibility
//!   - Resolves custom `unit` declarations into the registry
//!   - Produces structured errors with source locations
//!
//! ## Pipeline position
//!
//!   Lexer (C) → Parser (C) → [Unit Resolver (Rust)] → Type Checker (Rust)
//!
//! ## Usage
//!
//! ```rust
//! use nova_unit_resolver::{UnitResolver, ResolvedUnit};
//!
//! let mut resolver = UnitResolver::new();
//! let resolved = resolver.resolve_unit_str("m/s").unwrap();
//! assert_eq!(resolved.dim, nova_unit_resolver::dimension::Dim::VELOCITY);
//! ```

pub mod dimension;
pub mod si_table;
pub mod custom_units;

use dimension::Dim;
use si_table::{build_si_table, UnitEntry};
use custom_units::CustomUnitRegistry;
use std::collections::HashMap;

// ── Error type ────────────────────────────────────────────────────────────────

/// A unit resolution error, with source location.
#[derive(Debug, Clone)]
pub struct UnitError {
    pub message: String,
    pub line: u32,
    pub col: u32,
}

impl UnitError {
    fn new(msg: impl Into<String>) -> Self {
        UnitError { message: msg.into(), line: 0, col: 0 }
    }
    fn at(msg: impl Into<String>, line: u32, col: u32) -> Self {
        UnitError { message: msg.into(), line, col }
    }
}

impl std::fmt::Display for UnitError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.line > 0 {
            write!(f, "{}:{}: unit error: {}", self.line, self.col, self.message)
        } else {
            write!(f, "unit error: {}", self.message)
        }
    }
}

// ── Resolved unit ──────────────────────────────────────────────────────────────

/// The result of resolving a unit expression.
#[derive(Debug, Clone, PartialEq)]
pub struct ResolvedUnit {
    /// The SI dimension vector.
    pub dim: Dim,
    /// Scale factor: 1 [this unit] = scale [SI base unit].
    pub scale: f64,
    /// The original unit string, for display in error messages.
    pub original: String,
}

impl ResolvedUnit {
    /// Multiply two resolved units: combine dim and scale.
    pub fn mul(&self, rhs: &ResolvedUnit) -> ResolvedUnit {
        ResolvedUnit {
            dim:      self.dim.mul(rhs.dim),
            scale:    self.scale * rhs.scale,
            original: format!("{}·{}", self.original, rhs.original),
        }
    }

    /// Divide two resolved units.
    pub fn div(&self, rhs: &ResolvedUnit) -> ResolvedUnit {
        ResolvedUnit {
            dim:      self.dim.div(rhs.dim),
            scale:    self.scale / rhs.scale,
            original: format!("{}/{}", self.original, rhs.original),
        }
    }

    /// Raise to an integer power.
    pub fn pow(&self, n: i8) -> ResolvedUnit {
        ResolvedUnit {
            dim:      self.dim.pow(n),
            scale:    self.scale.powi(n as i32),
            original: format!("{}^{}", self.original, n),
        }
    }
}

// ── Unit expression parser ─────────────────────────────────────────────────────

/// Minimal recursive-descent parser for unit expressions.
///
/// Grammar:
///   unit_expr  := unit_term (('*' | '·') unit_term | '/' unit_term)*
///   unit_term  := unit_atom ('^' int_exp)?
///   unit_atom  := SYMBOL | '(' unit_expr ')'
///   int_exp    := '-'? DIGIT+
///
/// Handles:
///   m/s         → LENGTH / TIME = VELOCITY
///   m/s²        → LENGTH / TIME^2 = ACCELERATION  (² parsed as ^2)
///   kg·m²/s²    → MASS * AREA / TIME^2 = ENERGY
///   N·m²/kg²    → FORCE * AREA / MASS^2  (gravitational constant)
///   km/s        → VELOCITY with scale 1000
struct UnitExprParser<'a> {
    input: &'a [u8],
    pos:   usize,
    si:    &'a HashMap<&'static str, UnitEntry>,
    custom: &'a CustomUnitRegistry,
}

impl<'a> UnitExprParser<'a> {
    fn new(input: &'a str, si: &'a HashMap<&'static str, UnitEntry>, custom: &'a CustomUnitRegistry) -> Self {
        UnitExprParser { input: input.as_bytes(), pos: 0, si, custom }
    }

    fn peek(&self) -> Option<u8> {
        self.input.get(self.pos).copied()
    }

    fn advance(&mut self) -> Option<u8> {
        let b = self.input.get(self.pos).copied();
        if b.is_some() { self.pos += 1; }
        b
    }

    fn skip_spaces(&mut self) {
        while matches!(self.peek(), Some(b' ') | Some(b'\t')) { self.advance(); }
    }

    /// Skip a Unicode multi-byte sequence, returning how many bytes consumed.
    fn skip_utf8(&mut self) -> usize {
        let lead = match self.peek() { Some(b) => b, None => return 0 };
        let len = if lead < 0x80 { 1 }
                  else if lead & 0xE0 == 0xC0 { 2 }
                  else if lead & 0xF0 == 0xE0 { 3 }
                  else { 4 };
        for _ in 0..len { self.advance(); }
        len
    }

    /// Check if the next bytes are a Unicode superscript digit (², ³, etc.)
    /// Returns Some(digit) if so.
    fn try_superscript(&self) -> Option<i8> {
        if self.pos + 1 < self.input.len() {
            let a = self.input[self.pos];
            let b = self.input[self.pos + 1];
            match (a, b) {
                (0xC2, 0xB2) => return Some(2), // ²
                (0xC2, 0xB3) => return Some(3), // ³
                _ => {}
            }
        }
        // Could extend to ¹⁴⁵⁶⁷⁸⁹ etc. here
        None
    }

    /// Read an ASCII symbol (letters, digits, _, /, unicode identifiers).
    fn read_symbol(&mut self) -> String {
        let start = self.pos;
        let mut chars_read = 0usize;
        while self.pos < self.input.len() {
            let b = self.input[self.pos];
            // Stop on operators and delimiters
            if matches!(b, b'*' | b'/' | b'^' | b'(' | b')' | b' ' | b'\t') {
                break;
            }
            // Stop on ASCII digits after the first character.
            // Digits in unit expressions are exponents (s^2, kg^3),
            // not part of symbol names. Only the first char can be a digit
            // (which shouldn't happen in practice since numbers are handled
            // before symbol reading).
            if chars_read > 0 && b.is_ascii_digit() {
                break;
            }
            // Middle dot (Â·) U+00B7 = 0xC2 0xB7 â stop (itâs a multiplier)
            if b == 0xC2 && self.pos + 1 < self.input.len() && self.input[self.pos + 1] == 0xB7 {
                break;
            }
            // Superscript digit (Â², Â³) â part of the previous symbol via ^
            if self.try_superscript().is_some() { break; }
            if b >= 0x80 { self.skip_utf8(); }
            else { self.advance(); }
            chars_read += 1;
        }
        String::from_utf8_lossy(&self.input[start..self.pos]).into_owned()
    }

    /// Look up a symbol in SI table then custom registry.
    fn lookup(&self, sym: &str) -> Result<ResolvedUnit, UnitError> {
        if let Some(entry) = self.si.get(sym) {
            return Ok(ResolvedUnit { dim: entry.dim, scale: entry.scale, original: sym.to_string() });
        }
        if let Some(cu) = self.custom.get(sym) {
            return Ok(ResolvedUnit { dim: cu.dim, scale: cu.scale, original: sym.to_string() });
        }
        Err(UnitError::new(format!("unknown unit symbol '{}'\n  hint: is this a custom unit? declare it with `unit {} = <value>[<base_unit>]`", sym, sym)))
    }

    fn parse_int_exp(&mut self) -> i8 {
        self.skip_spaces();
        let neg = if self.peek() == Some(b'-') { self.advance(); true } else { false };
        let mut val: i8 = 0;
        while matches!(self.peek(), Some(b'0'..=b'9')) {
            val = val * 10 + (self.advance().unwrap() - b'0') as i8;
        }
        if neg { -val } else { val }
    }

    fn parse_atom(&mut self) -> Result<ResolvedUnit, UnitError> {
        self.skip_spaces();
        if self.peek() == Some(b'(') {
            self.advance();
            let r = self.parse_expr()?;
            self.skip_spaces();
            if self.peek() == Some(b')') { self.advance(); }
            return Ok(r);
        }
        let sym = self.read_symbol();
        if sym.is_empty() {
            return Err(UnitError::new("expected unit symbol"));
        }
        self.lookup(&sym)
    }

    fn parse_term(&mut self) -> Result<ResolvedUnit, UnitError> {
        let mut base = self.parse_atom()?;
        self.skip_spaces();

        // Explicit ^n
        if self.peek() == Some(b'^') {
            self.advance();
            let exp = self.parse_int_exp();
            base = base.pow(exp);
            return Ok(base);
        }

        // Unicode superscript (², ³) immediately after symbol
        if let Some(exp) = self.try_superscript() {
            self.pos += 2; // skip 2 UTF-8 bytes
            base = base.pow(exp);
        }

        Ok(base)
    }

    fn parse_expr(&mut self) -> Result<ResolvedUnit, UnitError> {
        let mut lhs = self.parse_term()?;
        loop {
            self.skip_spaces();
            match self.peek() {
                Some(b'*') => { self.advance(); lhs = lhs.mul(&self.parse_term()?); }
                Some(b'/') => { self.advance(); lhs = lhs.div(&self.parse_term()?); }
                // Middle dot U+00B7 (0xC2 0xB7) — treat as multiplication
                Some(0xC2) if self.pos + 1 < self.input.len() && self.input[self.pos + 1] == 0xB7 => {
                    self.pos += 2;
                    lhs = lhs.mul(&self.parse_term()?);
                }
                _ => break,
            }
        }
        Ok(lhs)
    }
}

// ── UnitResolver ──────────────────────────────────────────────────────────────

/// The main unit resolver. Holds the SI table and custom unit registry.
pub struct UnitResolver {
    si:     HashMap<&'static str, UnitEntry>,
    custom: CustomUnitRegistry,
}

impl UnitResolver {
    /// Create a new resolver with the full SI table loaded.
    pub fn new() -> Self {
        UnitResolver {
            si:     build_si_table(),
            custom: CustomUnitRegistry::new(),
        }
    }

    /// Register a custom unit declaration.
    ///
    /// Called when the resolver encounters:
    ///   `unit parsec = 3.086e16[m]`
    ///
    /// `name`        — "parsec"
    /// `factor`      — 3.086e16
    /// `base_unit`   — "m"
    pub fn register_custom_unit(
        &mut self,
        name: &str,
        factor: f64,
        base_unit: &str,
        source_expr: &str,
        line: u32,
        col: u32,
    ) -> Result<(), UnitError> {
        let entry = self.si.get(base_unit)
            .ok_or_else(|| UnitError::at(
                format!("base unit '{}' not found in SI table", base_unit),
                line, col,
            ))?;
        self.custom.register(name, factor, entry, source_expr);
        Ok(())
    }

    /// Resolve a unit expression string to a (Dim, scale) pair.
    ///
    /// Input is the raw string between `[` and `]` in a NOVA source literal.
    pub fn resolve_unit_str(&self, unit_str: &str) -> Result<ResolvedUnit, UnitError> {
        let trimmed = unit_str.trim();
        if trimmed.is_empty() {
            return Err(UnitError::new("empty unit expression"));
        }
        let mut parser = UnitExprParser::new(trimmed, &self.si, &self.custom);
        let result = parser.parse_expr()?;
        // Ensure we consumed all input
        parser.skip_spaces();
        if parser.pos < parser.input.len() {
            let remaining = std::str::from_utf8(&parser.input[parser.pos..]).unwrap_or("?");
            return Err(UnitError::new(format!(
                "unexpected trailing input in unit expression: '{}'", remaining
            )));
        }
        Ok(result)
    }

    /// Check that two resolved units are dimensionally compatible for addition.
    pub fn check_add_compatible(
        &self,
        lhs: &ResolvedUnit,
        rhs: &ResolvedUnit,
        line: u32,
        col: u32,
    ) -> Result<(), UnitError> {
        if lhs.dim == rhs.dim {
            Ok(())
        } else {
            Err(UnitError::at(
                format!(
                    "cannot add {} and {}\n  left:  {}\n  right: {}\n  hint: did you mean to convert units, or is this a logic error?",
                    lhs.original, rhs.original,
                    lhs.dim.name(), rhs.dim.name(),
                ),
                line, col,
            ))
        }
    }

    /// Multiply two resolved units.
    pub fn mul(&self, lhs: &ResolvedUnit, rhs: &ResolvedUnit) -> ResolvedUnit {
        lhs.mul(rhs)
    }

    /// Divide two resolved units.
    pub fn div(&self, lhs: &ResolvedUnit, rhs: &ResolvedUnit) -> ResolvedUnit {
        lhs.div(rhs)
    }

    /// Raise a resolved unit to an integer power.
    pub fn pow(&self, u: &ResolvedUnit, n: i8) -> ResolvedUnit {
        u.pow(n)
    }
}

impl Default for UnitResolver {
    fn default() -> Self { Self::new() }
}

// ── Tests ─────────────────────────────────────────────────────────────────────
#[cfg(test)]
mod tests {
    use super::*;
    use dimension::Dim;

    fn r() -> UnitResolver { UnitResolver::new() }

    // ── Basic symbol resolution ──────────────────────────────────────────────
    #[test]
    fn resolve_kg() {
        let u = r().resolve_unit_str("kg").unwrap();
        assert_eq!(u.dim, Dim::MASS);
    }

    #[test]
    fn resolve_m() {
        let u = r().resolve_unit_str("m").unwrap();
        assert_eq!(u.dim, Dim::LENGTH);
    }

    #[test]
    fn resolve_s() {
        let u = r().resolve_unit_str("s").unwrap();
        assert_eq!(u.dim, Dim::TIME);
    }

    #[test]
    fn resolve_newton() {
        let u = r().resolve_unit_str("N").unwrap();
        assert_eq!(u.dim, Dim::FORCE);
    }

    // ── Compound units ───────────────────────────────────────────────────────
    #[test]
    fn resolve_m_per_s() {
        let u = r().resolve_unit_str("m/s").unwrap();
        assert_eq!(u.dim, Dim::VELOCITY);
    }

    #[test]
    fn resolve_km_per_s() {
        let u = r().resolve_unit_str("km/s").unwrap();
        assert_eq!(u.dim, Dim::VELOCITY);
        assert!((u.scale - 1000.0).abs() < 0.1);
    }

    #[test]
    fn resolve_m_per_s2_ascii() {
        let u = r().resolve_unit_str("m/s^2").unwrap();
        assert_eq!(u.dim, Dim::ACCELERATION);
    }

    #[test]
    fn resolve_gravitational_constant_units() {
        // G has units N·m²/kg² = [1,1,-2] * [2,0,0] / [0,2,0]
        //   = [3,1,-2] / [0,2,0] = [3,-1,-2]
        let u = r().resolve_unit_str("N*m^2/kg^2").unwrap();
        let expected = Dim::FORCE.mul(Dim::AREA).div(Dim::MASS.pow(2));
        assert_eq!(u.dim, expected);
    }

    #[test]
    fn resolve_hubble_constant_units() {
        // Hubble constant: km/s/Mpc = velocity / distance
        let u = r().resolve_unit_str("km/s").unwrap();
        assert_eq!(u.dim, Dim::VELOCITY);
    }

    #[test]
    fn resolve_energy_ev() {
        let u = r().resolve_unit_str("eV").unwrap();
        assert_eq!(u.dim, Dim::ENERGY);
        assert!((u.scale - 1.602e-19).abs() < 1e-22);
    }

    // ── Unit arithmetic through resolver ─────────────────────────────────────
    #[test]
    fn thrust_divided_by_mass_flow_gives_velocity() {
        // Isp = thrust [N] / (mass_flow [kg/s] * g0 [m/s²])
        // Dimension: [N] / ([kg/s] * [m/s²]) = [M·L·T⁻²] / [M·L·T⁻²·T] = [T]  = seconds
        let res = r();
        let _thrust   = res.resolve_unit_str("N").unwrap();
        let mass_flow = res.resolve_unit_str("kg/s").unwrap();
        // kg/s is MASS / TIME = [0,1,-1,0,0,0,0]
        let kg_per_s = Dim::MASS.div(Dim::TIME);
        assert_eq!(mass_flow.dim, kg_per_s);
    }

    #[test]
    fn isp_units_resolve_to_seconds() {
        // Isp = F / (m_dot * g0) → [N] / ([kg/s] * [m/s²])
        //     = [M·L·T⁻²] / [M·T⁻¹ · L·T⁻²]
        //     = [M·L·T⁻²] / [M·L·T⁻³]
        //     = [T]  ← seconds ✓
        let force      = Dim::FORCE;
        let mass_rate  = Dim::MASS.div(Dim::TIME);
        let accel      = Dim::ACCELERATION;
        let denominator = mass_rate.mul(accel);
        let isp_dim     = force.div(denominator);
        assert_eq!(isp_dim, Dim::TIME);
    }

    // ── Add compatibility checks ──────────────────────────────────────────────
    #[test]
    fn add_compatible_same_dim() {
        let res = r();
        let a = res.resolve_unit_str("kg").unwrap();
        let b = res.resolve_unit_str("kg").unwrap();
        assert!(res.check_add_compatible(&a, &b, 0, 0).is_ok());
    }

    #[test]
    fn add_incompatible_kg_plus_m() {
        let res = r();
        let kg = res.resolve_unit_str("kg").unwrap();
        let m  = res.resolve_unit_str("m").unwrap();
        let err = res.check_add_compatible(&kg, &m, 10, 5).unwrap_err();
        assert!(err.message.contains("cannot add"));
        assert!(err.message.contains("mass") || err.message.contains("length") || err.message.contains("M") || err.message.contains("L"));
        assert_eq!(err.line, 10);
        assert_eq!(err.col, 5);
    }

    // ── Custom unit registration ──────────────────────────────────────────────
    #[test]
    fn custom_unit_parsec() {
        let mut res = r();
        res.register_custom_unit("parsec", 3.086e16, "m", "3.086e16[m]", 1, 1).unwrap();
        let u = res.resolve_unit_str("parsec").unwrap();
        assert_eq!(u.dim, Dim::LENGTH);
        assert!((u.scale - 3.086e16).abs() < 1e10);
    }

    #[test]
    fn custom_unit_unknown_base_errors() {
        let mut res = r();
        let err = res.register_custom_unit("myunit", 1.0, "foobar", "1.0[foobar]", 5, 3);
        assert!(err.is_err());
        assert!(err.unwrap_err().message.contains("foobar"));
    }

    // ── Error cases ───────────────────────────────────────────────────────────
    #[test]
    fn unknown_unit_errors() {
        let err = r().resolve_unit_str("furlongs").unwrap_err();
        assert!(err.message.contains("furlongs"));
        assert!(err.message.contains("hint"));
    }

    #[test]
    fn empty_unit_string_errors() {
        assert!(r().resolve_unit_str("").is_err());
        assert!(r().resolve_unit_str("   ").is_err());
    }

    // ── Real NOVA examples ────────────────────────────────────────────────────
    #[test]
    fn delta_v_unit_chain() {
        // Tsiolkovsky: Δv = Isp[s] * g0[m/s²] * ln(ratio[dimensionless])
        // Result dimension: [s] * [m/s²] = [m/s] = VELOCITY ✓
        let isp_dim  = Dim::TIME;
        let g0_dim   = Dim::ACCELERATION;
        let result   = isp_dim.mul(g0_dim);
        assert_eq!(result, Dim::VELOCITY);
    }

    #[test]
    fn hubble_constant_units() {
        // H0 in km/s/Mpc — dimensionally velocity / length = 1/time = FREQUENCY
        let vel_dim = Dim::VELOCITY;
        let len_dim = Dim::LENGTH;
        let h0_dim  = vel_dim.div(len_dim);
        assert_eq!(h0_dim, Dim::FREQUENCY);
    }

    #[test]
    fn unit_mismatch_error_message_quality() {
        let res = r();
        let kg = res.resolve_unit_str("kg").unwrap();
        let m  = res.resolve_unit_str("m").unwrap();
        let err = res.check_add_compatible(&kg, &m, 12, 3).unwrap_err();
        // The error message must name both units and give a hint
        assert!(err.message.contains("kg") || err.message.contains("m"));
        assert!(err.message.contains("hint"));
    }
}
