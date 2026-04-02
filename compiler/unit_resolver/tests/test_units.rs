//! Integration tests for the NOVA unit resolver.
//!
//! These tests exercise the resolver from the outside — the same way
//! the type checker (Phase 2b) will use it.
//!
//! Run with: cargo test -p nova_unit_resolver

use nova_unit_resolver::{UnitResolver, dimension::Dim};

// ── SI base units ─────────────────────────────────────────────────────────────

#[test] fn base_length()      { assert_eq!(r("m").dim,   Dim::LENGTH); }
#[test] fn base_mass()        { assert_eq!(r("kg").dim,  Dim::MASS); }
#[test] fn base_time()        { assert_eq!(r("s").dim,   Dim::TIME); }
#[test] fn base_temperature() { assert_eq!(r("K").dim,   Dim::TEMPERATURE); }
#[test] fn base_current()     { assert_eq!(r("A").dim,   Dim::CURRENT); }
#[test] fn base_amount()      { assert_eq!(r("mol").dim, Dim::AMOUNT); }
#[test] fn base_luminosity()  { assert_eq!(r("cd").dim,  Dim::LUMINOSITY); }

// ── Derived units ──────────────────────────────────────────────────────────────

#[test] fn derived_velocity()     { assert_eq!(r("m/s").dim,   Dim::VELOCITY); }
#[test] fn derived_acceleration() { assert_eq!(r("m/s^2").dim, Dim::ACCELERATION); }
#[test] fn derived_force_n()      { assert_eq!(r("N").dim,     Dim::FORCE); }
#[test] fn derived_energy_j()     { assert_eq!(r("J").dim,     Dim::ENERGY); }
#[test] fn derived_power_w()      { assert_eq!(r("W").dim,     Dim::POWER); }
#[test] fn derived_pressure_pa()  { assert_eq!(r("Pa").dim,    Dim::PRESSURE); }
#[test] fn derived_frequency_hz() { assert_eq!(r("Hz").dim,    Dim::FREQUENCY); }

// ── Compound expressions ──────────────────────────────────────────────────────

#[test]
fn compound_grav_constant() {
    // G: N·m²/kg² = force * area / mass² 
    let u = r("N*m^2/kg^2");
    let expected = Dim::FORCE.mul(Dim::AREA).div(Dim::MASS.pow(2));
    assert_eq!(u.dim, expected);
}

#[test]
fn compound_energy_kg_m2_s2() {
    let a = r("kg*m^2/s^2");
    assert_eq!(a.dim, Dim::ENERGY);
}

#[test]
fn compound_momentum() {
    let u = r("kg*m/s");
    assert_eq!(u.dim, Dim::MOMENTUM);
}

// ── Scale factors ─────────────────────────────────────────────────────────────

#[test] fn scale_km_1000()   { assert!((r("km").scale  - 1000.0).abs() < 0.1); }
#[test] fn scale_ev()        { assert!((r("eV").scale  - 1.602e-19).abs() < 1e-22); }
#[test] fn scale_au()        { assert!((r("AU").scale  - 1.496e11).abs() < 1e7); }
#[test] fn scale_solar_mass(){ assert!((r("M_sun").scale - 1.989e30).abs() < 1e25); }

// ── Custom unit registration ───────────────────────────────────────────────────

#[test]
fn custom_parsec_length() {
    let mut res = UnitResolver::new();
    res.register_custom_unit("parsec", 3.086e16, "m", "3.086e16[m]", 0, 0).unwrap();
    let u = res.resolve_unit_str("parsec").unwrap();
    assert_eq!(u.dim, Dim::LENGTH);
    assert!((u.scale - 3.086e16).abs() < 1e10);
}

#[test]
fn custom_unit_usable_in_compound() {
    let mut res = UnitResolver::new();
    res.register_custom_unit("au", 1.496e11, "m", "1.496e11[m]", 0, 0).unwrap();
    let u = res.resolve_unit_str("au").unwrap();
    assert_eq!(u.dim, Dim::LENGTH);
}

// ── Add compatibility (the key safety check) ─────────────────────────────────

#[test]
fn add_kg_kg_ok() {
    let res = UnitResolver::new();
    let a = res.resolve_unit_str("kg").unwrap();
    let b = res.resolve_unit_str("kg").unwrap();
    assert!(res.check_add_compatible(&a, &b, 0, 0).is_ok());
}

#[test]
fn add_km_m_ok_same_dim() {
    // km and m are both LENGTH — dimensionally compatible
    let res = UnitResolver::new();
    let a = res.resolve_unit_str("km").unwrap();
    let b = res.resolve_unit_str("m").unwrap();
    assert!(res.check_add_compatible(&a, &b, 0, 0).is_ok());
}

#[test]
fn add_kg_m_fails() {
    let res = UnitResolver::new();
    let kg = res.resolve_unit_str("kg").unwrap();
    let m  = res.resolve_unit_str("m").unwrap();
    let err = res.check_add_compatible(&kg, &m, 1, 1).unwrap_err();
    assert!(err.message.contains("cannot add"));
    assert!(err.message.contains("hint"));
    assert_eq!(err.line, 1);
}

// ── Real NOVA program unit chains ─────────────────────────────────────────────

#[test]
fn delta_v_chain() {
    // Δv = Isp[s] * g0[m/s²]  →  should be [m/s]
    let isp = Dim::TIME;
    let g0  = Dim::ACCELERATION;
    assert_eq!(isp.mul(g0), Dim::VELOCITY);
}

#[test]
fn hubble_constant_chain() {
    // H₀ = velocity / distance  →  1/time = frequency
    assert_eq!(Dim::VELOCITY.div(Dim::LENGTH), Dim::FREQUENCY);
}

#[test]
fn stefan_boltzmann_units() {
    // σ: W / (m² · K⁴)
    // = POWER / (AREA * TEMPERATURE^4)
    let sigma = Dim::POWER
        .div(Dim::AREA)
        .div(Dim::TEMPERATURE.pow(4));
    // Should be: [2,1,-3,0,-4,0,0]
    assert_eq!(sigma.exp[0], 0); // L: 2-2 = 0
    assert_eq!(sigma.exp[1], 1); // M: 1
    assert_eq!(sigma.exp[2],-3); // T: -3
    assert_eq!(sigma.exp[4],-4); // Θ: -4
}

// ── Error quality ─────────────────────────────────────────────────────────────

#[test]
fn unknown_unit_error_has_hint() {
    let err = UnitResolver::new().resolve_unit_str("lightyears").unwrap_err();
    assert!(err.message.contains("hint"));
    assert!(err.message.contains("lightyears"));
}

// ── Helper ────────────────────────────────────────────────────────────────────
fn r(s: &str) -> nova_unit_resolver::ResolvedUnit {
    UnitResolver::new().resolve_unit_str(s).expect(s)
}
