// NOVA Language Server - Diagnostics Engine
// Provides error highlighting, especially for unit mismatches

use std::collections::HashMap;

/// Represents a diagnostic (error/warning) in source code
#[derive(Debug, Clone)]
pub struct Diagnostic {
    pub line: u32,
    pub column: u32,
    pub message: String,
    pub severity: DiagnosticSeverity,
    pub code: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DiagnosticSeverity {
    Error,
    Warning,
    Info,
    Hint,
}

/// The diagnostics engine for NOVA
pub struct DiagnosticsEngine {
    diagnostics: Vec<Diagnostic>,
}

impl DiagnosticsEngine {
    pub fn new() -> Self {
        DiagnosticsEngine {
            diagnostics: Vec::new(),
        }
    }

    /// Add a diagnostic
    pub fn add(&mut self, diag: Diagnostic) {
        self.diagnostics.push(diag);
    }

    /// Add a unit mismatch error
    pub fn add_unit_mismatch(&mut self, line: u32, col: u32, expected: &str, got: &str) {
        let message = format!(
            "cannot use {} where {} is expected - dimension mismatch",
            got, expected
        );
        self.add(Diagnostic {
            line,
            column: col,
            message,
            severity: DiagnosticSeverity::Error,
            code: Some("UNIT_MISMATCH".to_string()),
        });
    }

    /// Add a type error
    pub fn add_type_error(&mut self, line: u32, col: u32, message: String) {
        self.add(Diagnostic {
            line,
            column: col,
            message,
            severity: DiagnosticSeverity::Error,
            code: Some("TYPE_ERROR".to_string()),
        });
    }

    /// Add a warning
    pub fn add_warning(&mut self, line: u32, col: u32, message: String) {
        self.add(Diagnostic {
            line,
            column: col,
            message,
            severity: DiagnosticSeverity::Warning,
            code: None,
        });
    }

    /// Get all diagnostics
    pub fn get_all(&self) -> &[Diagnostic] {
        &self.diagnostics
    }

    /// Clear all diagnostics
    pub fn clear(&mut self) {
        self.diagnostics.clear();
    }

    /// Check if there are any errors
    pub fn has_errors(&self) -> bool {
        self.diagnostics.iter().any(|d| d.severity == DiagnosticSeverity::Error)
    }

    /// Get error count
    pub fn error_count(&self) -> usize {
        self.diagnostics.iter().filter(|d| d.severity == DiagnosticSeverity::Error).count()
    }
}

impl Default for DiagnosticsEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_diagnostic() {
        let mut engine = DiagnosticsEngine::new();
        engine.add_unit_mismatch(10, 5, "Float[kg]", "Float[m]");
        assert_eq!(engine.diagnostics.len(), 1);
        assert!(engine.has_errors());
    }

    #[test]
    fn test_clear_diagnostics() {
        let mut engine = DiagnosticsEngine::new();
        engine.add_warning(1, 0, "test".to_string());
        assert!(!engine.diagnostics.is_empty());
        engine.clear();
        assert!(engine.diagnostics.is_empty());
    }
}