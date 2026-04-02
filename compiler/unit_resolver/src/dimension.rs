//! SI Dimension Vector
//!
//! Every physical quantity in NOVA is described by a 7-tuple of rational
//! exponents over the seven SI base dimensions:
//!
//!   [L, M, T, I, Θ, N, J]
//!    │  │  │  │  │  │  └── luminous intensity (candela)
//!    │  │  │  │  │  └───── amount of substance (mole)
//!    │  │  │  │  └──────── thermodynamic temperature (kelvin)
//!    │  │  │  └─────────── electric current (ampere)
//!    │  │  └────────────── time (second)
//!    │  └───────────────── mass (kilogram)
//!    └──────────────────── length (metre)
//!
//! Exponents are stored as i8 (range −128..127) which is more than sufficient
//! for any physical quantity encountered in science or engineering.
//! Rational exponents (e.g. [m^(1/2)]) are not yet supported — they are
//! represented as scaled integers if needed (future work).
//!
//! # Unit arithmetic rules
//!
//! | Operation          | Dimension result      |
//! |--------------------|-----------------------|
//! | a[U] * b[V]        | U + V  (component-wise addition)   |
//! | a[U] / b[V]        | U − V  (component-wise subtraction)|
//! | a[U] + b[V]        | ONLY if U == V; error otherwise    |
//! | a[U] − b[V]        | ONLY if U == V; error otherwise    |
//! | a[U] ^ n           | U * n  (scalar multiply)           |
//! | sqrt(a[U])         | U / 2  (half each exponent)        |

use std::fmt;

/// The 7-dimensional SI exponent vector.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Dim {
    /// Exponents: [L, M, T, I, Θ, N, J]
    pub exp: [i8; 7],
}

impl Dim {
    // ── Constructors ──────────────────────────────────────────────────────

    /// Dimensionless (scalar). All exponents zero.
    pub const DIMENSIONLESS: Dim = Dim { exp: [0, 0, 0, 0, 0, 0, 0] };

    /// Construct from explicit exponents.
    pub const fn new(l: i8, m: i8, t: i8, i: i8, theta: i8, n: i8, j: i8) -> Self {
        Dim { exp: [l, m, t, i, theta, n, j] }
    }

    // ── Pre-built SI dimensions ───────────────────────────────────────────

    pub const LENGTH:      Dim = Dim::new(1, 0,  0, 0, 0, 0, 0);
    pub const MASS:        Dim = Dim::new(0, 1,  0, 0, 0, 0, 0);
    pub const TIME:        Dim = Dim::new(0, 0,  1, 0, 0, 0, 0);
    pub const CURRENT:     Dim = Dim::new(0, 0,  0, 1, 0, 0, 0);
    pub const TEMPERATURE: Dim = Dim::new(0, 0,  0, 0, 1, 0, 0);
    pub const AMOUNT:      Dim = Dim::new(0, 0,  0, 0, 0, 1, 0);
    pub const LUMINOSITY:  Dim = Dim::new(0, 0,  0, 0, 0, 0, 1);

    // Derived SI dimensions
    pub const VELOCITY:     Dim = Dim::new( 1,  0, -1, 0, 0, 0, 0); // m/s
    pub const ACCELERATION: Dim = Dim::new( 1,  0, -2, 0, 0, 0, 0); // m/s²
    pub const FORCE:        Dim = Dim::new( 1,  1, -2, 0, 0, 0, 0); // N = kg·m/s²
    pub const ENERGY:       Dim = Dim::new( 2,  1, -2, 0, 0, 0, 0); // J = kg·m²/s²
    pub const POWER:        Dim = Dim::new( 2,  1, -3, 0, 0, 0, 0); // W = kg·m²/s³
    pub const PRESSURE:     Dim = Dim::new(-1,  1, -2, 0, 0, 0, 0); // Pa = kg/(m·s²)
    pub const FREQUENCY:    Dim = Dim::new( 0,  0, -1, 0, 0, 0, 0); // Hz = 1/s
    pub const AREA:         Dim = Dim::new( 2,  0,  0, 0, 0, 0, 0); // m²
    pub const VOLUME:       Dim = Dim::new( 3,  0,  0, 0, 0, 0, 0); // m³
    pub const DENSITY:      Dim = Dim::new(-3,  1,  0, 0, 0, 0, 0); // kg/m³
    pub const MOMENTUM:     Dim = Dim::new( 1,  1, -1, 0, 0, 0, 0); // kg·m/s
    pub const CHARGE:       Dim = Dim::new( 0,  0,  1, 1, 0, 0, 0); // C = A·s
    pub const VOLTAGE:      Dim = Dim::new( 2,  1, -3,-1, 0, 0, 0); // V = kg·m²/(A·s³)

    // ── Arithmetic ────────────────────────────────────────────────────────

    /// Multiply two quantities: add exponent vectors.
    #[inline]
    pub fn mul(self, rhs: Dim) -> Dim {
        Dim { exp: [
            self.exp[0] + rhs.exp[0],
            self.exp[1] + rhs.exp[1],
            self.exp[2] + rhs.exp[2],
            self.exp[3] + rhs.exp[3],
            self.exp[4] + rhs.exp[4],
            self.exp[5] + rhs.exp[5],
            self.exp[6] + rhs.exp[6],
        ]}
    }

    /// Divide two quantities: subtract exponent vectors.
    #[inline]
    pub fn div(self, rhs: Dim) -> Dim {
        Dim { exp: [
            self.exp[0] - rhs.exp[0],
            self.exp[1] - rhs.exp[1],
            self.exp[2] - rhs.exp[2],
            self.exp[3] - rhs.exp[3],
            self.exp[4] - rhs.exp[4],
            self.exp[5] - rhs.exp[5],
            self.exp[6] - rhs.exp[6],
        ]}
    }

    /// Raise to an integer power: multiply each exponent by n.
    #[inline]
    pub fn pow(self, n: i8) -> Dim {
        Dim { exp: self.exp.map(|e| e * n) }
    }

    /// Check that two dimensions match (for addition/subtraction).
    #[inline]
    pub fn compatible_add(self, rhs: Dim) -> bool {
        self == rhs
    }

    /// Return true if this is dimensionless.
    #[inline]
    pub fn is_dimensionless(self) -> bool {
        self == Dim::DIMENSIONLESS
    }

    /// Human-readable name for this dimension (for error messages).
    pub fn name(self) -> String {
        if self == Dim::DIMENSIONLESS  { return "dimensionless".into(); }
        if self == Dim::LENGTH         { return "[L] length".into(); }
        if self == Dim::MASS           { return "[M] mass".into(); }
        if self == Dim::TIME           { return "[T] time".into(); }
        if self == Dim::CURRENT        { return "[I] current".into(); }
        if self == Dim::TEMPERATURE    { return "[Θ] temperature".into(); }
        if self == Dim::AMOUNT         { return "[N] amount".into(); }
        if self == Dim::LUMINOSITY     { return "[J] luminosity".into(); }
        if self == Dim::VELOCITY       { return "[L·T⁻¹] velocity".into(); }
        if self == Dim::ACCELERATION   { return "[L·T⁻²] acceleration".into(); }
        if self == Dim::FORCE          { return "[M·L·T⁻²] force".into(); }
        if self == Dim::ENERGY         { return "[M·L²·T⁻²] energy".into(); }
        if self == Dim::POWER          { return "[M·L²·T⁻³] power".into(); }
        if self == Dim::PRESSURE       { return "[M·L⁻¹·T⁻²] pressure".into(); }
        if self == Dim::FREQUENCY      { return "[T⁻¹] frequency".into(); }
        if self == Dim::MOMENTUM       { return "[M·L·T⁻¹] momentum".into(); }

        // Generic: render as L^a · M^b · T^c · ...
        let names = ["L", "M", "T", "I", "Θ", "N", "J"];
        let parts: Vec<String> = self.exp.iter().zip(names.iter())
            .filter(|(&e, _)| e != 0)
            .map(|(&e, &n)| {
                if e == 1 { n.to_string() }
                else { format!("{}^{}", n, e) }
            })
            .collect();
        format!("[{}]", parts.join("·"))
    }
}

impl fmt::Display for Dim {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name())
    }
}

// ── Index into dimension by base ──────────────────────────────────────────────
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Base { L=0, M=1, T=2, I=3, Theta=4, N=5, J=6 }

impl std::ops::Index<Base> for Dim {
    type Output = i8;
    fn index(&self, b: Base) -> &i8 { &self.exp[b as usize] }
}

// ── Tests ─────────────────────────────────────────────────────────────────────
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mul_kg_ms2_gives_newton() {
        // kg * m/s² = N = [1, 1, -2, 0, 0, 0, 0]
        let result = Dim::MASS.mul(Dim::ACCELERATION);
        assert_eq!(result, Dim::FORCE);
    }

    #[test]
    fn div_m_s_gives_velocity() {
        let result = Dim::LENGTH.div(Dim::TIME);
        assert_eq!(result, Dim::VELOCITY);
    }

    #[test]
    fn pow_length_2_gives_area() {
        let result = Dim::LENGTH.pow(2);
        assert_eq!(result, Dim::AREA);
    }

    #[test]
    fn compatible_add_same_dim() {
        assert!(Dim::MASS.compatible_add(Dim::MASS));
    }

    #[test]
    fn incompatible_add_different_dims() {
        assert!(!Dim::MASS.compatible_add(Dim::LENGTH));
    }

    #[test]
    fn dimensionless_check() {
        assert!(Dim::DIMENSIONLESS.is_dimensionless());
        assert!(!Dim::MASS.is_dimensionless());
    }

    #[test]
    fn force_divided_by_mass_gives_acceleration() {
        let result = Dim::FORCE.div(Dim::MASS);
        assert_eq!(result, Dim::ACCELERATION);
    }

    #[test]
    fn energy_name() {
        assert!(Dim::ENERGY.name().contains("energy"));
    }

    #[test]
    fn rocket_equation_units() {
        // Tsiolkovsky: Δv = Isp * g0 * ln(m_wet/m_dry)
        // Isp: [T], g0: [L·T⁻²], ln(...): dimensionless
        // Result: [T] * [L·T⁻²] = [L·T⁻¹] = velocity ✓
        let result = Dim::TIME.mul(Dim::ACCELERATION);
        assert_eq!(result, Dim::VELOCITY);
    }
}
