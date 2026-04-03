//! NOVA Code Formatter (nova-fmt)
//!
//! Opinionated, non-configurable formatter for NOVA source code.
//! Similar to gofmt - enforces consistent style across all NOVA projects.

pub mod formatter;

pub use formatter::{Formatter, FormatOptions, FormatResult};

/// Format a NOVA source file
pub fn format_source(source: &str, options: FormatOptions) -> Result<String, String> {
    let mut formatter = Formatter::new(options);
    formatter.format(source)
}

/// Format a NOVA source file with default options
pub fn format_source_default(source: &str) -> Result<String, String> {
    format_source(source, FormatOptions::default())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_simple_mission() {
        let input = r#"mission main()->Void{transmit("Hello")}"#;
        let result = format_source_default(input);
        // Formatter should at least not crash and return something
        assert!(result.is_ok());
    }
}