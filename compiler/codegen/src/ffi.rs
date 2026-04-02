//! Foreign Function Interface code generation
//!
//! Handles C FFI interop for NOVA missions calling C functions.

use crate::ir_emitter::{NovaType, CodegenError};

/// FFI binding for external C functions.
#[derive(Debug, Clone)]
pub struct ExternFunction {
    pub name: String,
    pub return_type: NovaType,
    pub parameters: Vec<NovaType>,
}

impl ExternFunction {
    pub fn new(name: String, return_type: NovaType) -> Self {
        ExternFunction {
            name,
            return_type,
            parameters: Vec::new(),
        }
    }

    pub fn add_parameter(&mut self, ty: NovaType) {
        self.parameters.push(ty);
    }

    pub fn emit_declaration(&self) -> String {
        let params = self
            .parameters
            .iter()
            .map(|ty| ty.to_string())
            .collect::<Vec<_>>()
            .join(", ");
        format!(
            "declare {} @{}({})",
            self.return_type.to_string(),
            self.name,
            params
        )
    }
}

/// FFI code generator.
pub struct FFICodegen;

impl FFICodegen {
    pub fn new() -> Self {
        FFICodegen
    }

    pub fn generate_wrapper(
        &self,
        extern_func: &ExternFunction,
    ) -> Result<String, CodegenError> {
        let wrapper_name = format!("{}_wrapper", extern_func.name);
        let mut wrapper = format!(
            "define {} @{}() {{\n  call {} @{}(",
            extern_func.return_type.to_string(),
            wrapper_name,
            extern_func.return_type.to_string(),
            extern_func.name
        );
        wrapper.push_str(")\n");
        wrapper.push_str("  ret void\n}");
        Ok(wrapper)
    }
}

impl Default for FFICodegen {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extern_function_creation() {
        let func = ExternFunction::new("malloc".to_string(), NovaType::Float);
        assert_eq!(func.name, "malloc");
        assert_eq!(func.return_type, NovaType::Float);
    }

    #[test]
    fn extern_function_declaration() {
        let func = ExternFunction::new("sin".to_string(), NovaType::Float);
        let decl = func.emit_declaration();
        assert!(decl.contains("declare f64 @sin"));
    }

    #[test]
    fn ffi_codegen_create() {
        let codegen = FFICodegen::new();
        let _ = codegen;
    }
}
