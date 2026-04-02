//! SI Unit Table
//!
//! Maps unit symbol strings (as they appear in NOVA source after `[`) to their
//! canonical SI dimension vector and a scale factor relative to the SI base unit.
//!
//! # Scale factor
//!
//! `scale` is the multiplier to convert one unit of this type into the SI base.
//! For example:
//!   km  → scale = 1000.0   (1 km = 1000 m)
//!   AU  → scale = 1.496e11 (1 AU = 1.496×10¹¹ m)
//!   eV  → scale = 1.602e-19 (1 eV = 1.602×10⁻¹⁹ J)
//!
//! Scale factors are used during unit conversion (`v as [km/s]`), but NOT
//! during type checking — only the dimension vector matters for type safety.

use crate::dimension::Dim;
use std::collections::HashMap;

/// One entry in the SI unit table.
#[derive(Debug, Clone, Copy)]
pub struct UnitEntry {
    /// The canonical SI dimension vector.
    pub dim: Dim,
    /// Scale factor: 1 [this unit] = scale [SI base unit].
    pub scale: f64,
    /// Human-readable name (for error messages).
    pub display: &'static str,
}

impl UnitEntry {
    const fn new(dim: Dim, scale: f64, display: &'static str) -> Self {
        UnitEntry { dim, scale, display }
    }
}

/// The full SI unit table, keyed by symbol string as it appears in NOVA source.
pub fn build_si_table() -> HashMap<&'static str, UnitEntry> {
    let mut t = HashMap::new();
    macro_rules! unit {
        ($sym:expr, $dim:expr, $scale:expr, $name:expr) => {
            t.insert($sym, UnitEntry::new($dim, $scale, $name));
        };
    }

    // ── Length ───────────────────────────────────────────────────────────────
    unit!("m",   Dim::LENGTH, 1.0,       "metre");
    unit!("cm",  Dim::LENGTH, 1e-2,      "centimetre");
    unit!("mm",  Dim::LENGTH, 1e-3,      "millimetre");
    unit!("km",  Dim::LENGTH, 1e3,       "kilometre");
    unit!("nm",  Dim::LENGTH, 1e-9,      "nanometre");
    unit!("um",  Dim::LENGTH, 1e-6,      "micrometre");
    unit!("AU",  Dim::LENGTH, 1.496e11,  "astronomical unit");
    unit!("ly",  Dim::LENGTH, 9.461e15,  "light-year");
    unit!("pc",  Dim::LENGTH, 3.086e16,  "parsec");
    unit!("kpc", Dim::LENGTH, 3.086e19,  "kiloparsec");
    unit!("Mpc", Dim::LENGTH, 3.086e22,  "megaparsec");
    unit!("ft",  Dim::LENGTH, 0.3048,    "foot");
    unit!("in",  Dim::LENGTH, 0.0254,    "inch");
    unit!("mi",  Dim::LENGTH, 1609.344,  "mile");

    // ── Mass ─────────────────────────────────────────────────────────────────
    unit!("kg",    Dim::MASS, 1.0,       "kilogram");
    unit!("g",     Dim::MASS, 1e-3,      "gram");
    unit!("mg",    Dim::MASS, 1e-6,      "milligram");
    unit!("t",     Dim::MASS, 1e3,       "metric tonne");
    unit!("lb",    Dim::MASS, 0.453592,  "pound");
    unit!("oz",    Dim::MASS, 0.028350,  "ounce");
    unit!("M_sun", Dim::MASS, 1.989e30,  "solar mass");
    unit!("M☉",    Dim::MASS, 1.989e30,  "solar mass");
    unit!("u",     Dim::MASS, 1.6605e-27,"atomic mass unit");

    // ── Time ─────────────────────────────────────────────────────────────────
    unit!("s",   Dim::TIME, 1.0,       "second");
    unit!("ms",  Dim::TIME, 1e-3,      "millisecond");
    unit!("us",  Dim::TIME, 1e-6,      "microsecond");
    unit!("ns",  Dim::TIME, 1e-9,      "nanosecond");
    unit!("min", Dim::TIME, 60.0,      "minute");
    unit!("hr",  Dim::TIME, 3600.0,    "hour");
    unit!("day", Dim::TIME, 86400.0,   "day");
    unit!("yr",  Dim::TIME, 3.156e7,   "year");
    unit!("Myr", Dim::TIME, 3.156e13,  "megayear");
    unit!("Gyr", Dim::TIME, 3.156e16,  "gigayear");

    // ── Temperature ──────────────────────────────────────────────────────────
    unit!("K",  Dim::TEMPERATURE, 1.0, "kelvin");
    // °C and °F require offset conversion — scale alone is not sufficient.
    // We record them as temperature-dimensional for type checking;
    // the unit converter handles the offset separately.
    unit!("C",  Dim::TEMPERATURE, 1.0, "degree Celsius");   // offset: +273.15
    unit!("F",  Dim::TEMPERATURE, 1.0, "degree Fahrenheit"); // offset: (F-32)*5/9+273.15

    // ── Electric current ─────────────────────────────────────────────────────
    unit!("A",  Dim::CURRENT, 1.0,  "ampere");
    unit!("mA", Dim::CURRENT, 1e-3, "milliampere");

    // ── Amount of substance ───────────────────────────────────────────────────
    unit!("mol", Dim::AMOUNT, 1.0, "mole");

    // ── Luminous intensity ────────────────────────────────────────────────────
    unit!("cd",  Dim::LUMINOSITY, 1.0, "candela");

    // ── Force ────────────────────────────────────────────────────────────────
    unit!("N",  Dim::FORCE, 1.0,    "newton");
    unit!("kN", Dim::FORCE, 1e3,    "kilonewton");
    unit!("MN", Dim::FORCE, 1e6,    "meganewton");
    unit!("lbf",Dim::FORCE, 4.4482, "pound-force");

    // ── Energy ───────────────────────────────────────────────────────────────
    unit!("J",    Dim::ENERGY, 1.0,      "joule");
    unit!("kJ",   Dim::ENERGY, 1e3,      "kilojoule");
    unit!("MJ",   Dim::ENERGY, 1e6,      "megajoule");
    unit!("GJ",   Dim::ENERGY, 1e9,      "gigajoule");
    unit!("cal",  Dim::ENERGY, 4.184,    "calorie");
    unit!("kcal", Dim::ENERGY, 4184.0,   "kilocalorie");
    unit!("eV",   Dim::ENERGY, 1.602e-19,"electron volt");
    unit!("keV",  Dim::ENERGY, 1.602e-16,"kiloelectron volt");
    unit!("MeV",  Dim::ENERGY, 1.602e-13,"megaelectron volt");
    unit!("GeV",  Dim::ENERGY, 1.602e-10,"gigaelectron volt");
    unit!("erg",  Dim::ENERGY, 1e-7,     "erg");

    // ── Power ────────────────────────────────────────────────────────────────
    unit!("W",  Dim::POWER, 1.0,  "watt");
    unit!("kW", Dim::POWER, 1e3,  "kilowatt");
    unit!("MW", Dim::POWER, 1e6,  "megawatt");
    unit!("GW", Dim::POWER, 1e9,  "gigawatt");
    unit!("L_sun", Dim::POWER, 3.828e26, "solar luminosity");
    unit!("L☉",    Dim::POWER, 3.828e26, "solar luminosity");

    // ── Pressure ─────────────────────────────────────────────────────────────
    unit!("Pa",  Dim::PRESSURE, 1.0,     "pascal");
    unit!("kPa", Dim::PRESSURE, 1e3,     "kilopascal");
    unit!("MPa", Dim::PRESSURE, 1e6,     "megapascal");
    unit!("atm", Dim::PRESSURE, 101325.0,"atmosphere");
    unit!("bar", Dim::PRESSURE, 1e5,     "bar");
    unit!("psi", Dim::PRESSURE, 6894.76, "pounds per square inch");

    // ── Frequency ────────────────────────────────────────────────────────────
    unit!("Hz",  Dim::FREQUENCY, 1.0,  "hertz");
    unit!("kHz", Dim::FREQUENCY, 1e3,  "kilohertz");
    unit!("MHz", Dim::FREQUENCY, 1e6,  "megahertz");
    unit!("GHz", Dim::FREQUENCY, 1e9,  "gigahertz");
    unit!("THz", Dim::FREQUENCY, 1e12, "terahertz");

    // ── Angle ────────────────────────────────────────────────────────────────
    // Angles are dimensionless in SI, but we track them separately by
    // using a special convention: angle uses T⁰ … (all zeros) but distinct
    // scale factors. The type checker treats angle as dimensionless.
    unit!("rad",    Dim::DIMENSIONLESS, 1.0,          "radian");
    unit!("deg",    Dim::DIMENSIONLESS, 0.017453,     "degree");
    unit!("arcmin", Dim::DIMENSIONLESS, 2.909e-4,     "arcminute");
    unit!("arcsec", Dim::DIMENSIONLESS, 4.848e-6,     "arcsecond");
    unit!("mas",    Dim::DIMENSIONLESS, 4.848e-9,     "milliarcsecond");

    // ── Data / information ────────────────────────────────────────────────────
    // Not an SI dimension; we use a custom dimension slot.
    // For now: dimensionless with scale in bytes.
    unit!("B",   Dim::DIMENSIONLESS, 1.0,    "byte");
    unit!("KB",  Dim::DIMENSIONLESS, 1024.0, "kilobyte");
    unit!("MB",  Dim::DIMENSIONLESS, 1.049e6,"megabyte");
    unit!("GB",  Dim::DIMENSIONLESS, 1.074e9,"gigabyte");
    unit!("TB",  Dim::DIMENSIONLESS, 1.1e12, "terabyte");

    // ── Velocity (compound, for convenience) ──────────────────────────────────
    unit!("m/s",   Dim::VELOCITY, 1.0,       "metres per second");
    unit!("km/s",  Dim::VELOCITY, 1e3,       "kilometres per second");
    unit!("km/h",  Dim::VELOCITY, 0.27778,   "kilometres per hour");
    unit!("mph",   Dim::VELOCITY, 0.44704,   "miles per hour");
    unit!("c",     Dim::VELOCITY, 2.998e8,   "speed of light");

    // ── Acceleration ─────────────────────────────────────────────────────────
    unit!("m/s2",  Dim::ACCELERATION, 1.0, "metres per second squared");
    unit!("m/s^2", Dim::ACCELERATION, 1.0, "metres per second squared");

    // ── Specific impulse ─────────────────────────────────────────────────────
    // Isp in seconds is force / (mass_flow * g0) — dimensionally [T]
    // (already covered by TIME table)

    t
}

// ── Tests ─────────────────────────────────────────────────────────────────────
#[cfg(test)]
mod tests {
    use super::*;

    fn table() -> HashMap<&'static str, UnitEntry> { build_si_table() }

    #[test]
    fn lookup_kg() {
        let t = table();
        let e = t.get("kg").expect("kg in table");
        assert_eq!(e.dim, Dim::MASS);
        assert!((e.scale - 1.0).abs() < 1e-10);
    }

    #[test]
    fn lookup_km_scale() {
        let t = table();
        let e = t.get("km").expect("km in table");
        assert_eq!(e.dim, Dim::LENGTH);
        assert!((e.scale - 1000.0).abs() < 0.1);
    }

    #[test]
    fn lookup_ev() {
        let t = table();
        let e = t.get("eV").expect("eV in table");
        assert_eq!(e.dim, Dim::ENERGY);
        assert!((e.scale - 1.602e-19).abs() < 1e-22);
    }

    #[test]
    fn lookup_newton() {
        let t = table();
        let e = t.get("N").expect("N in table");
        assert_eq!(e.dim, Dim::FORCE);
    }

    #[test]
    fn lookup_solar_mass() {
        let t = table();
        assert!(t.contains_key("M_sun"));
        assert!(t.contains_key("M☉"));
        assert_eq!(t["M_sun"].dim, t["M☉"].dim);
    }

    #[test]
    fn lookup_velocity_shorthand() {
        let t = table();
        assert_eq!(t["m/s"].dim, Dim::VELOCITY);
        assert_eq!(t["km/s"].dim, Dim::VELOCITY);
    }

    #[test]
    fn lookup_acceleration_both_keys() {
        let t = table();
        assert_eq!(t["m/s2"].dim,  Dim::ACCELERATION);
        assert_eq!(t["m/s^2"].dim, Dim::ACCELERATION);
    }

    #[test]
    fn all_entries_have_nonzero_scale() {
        for (sym, entry) in build_si_table() {
            assert!(entry.scale > 0.0, "scale for {} must be positive", sym);
        }
    }
}
