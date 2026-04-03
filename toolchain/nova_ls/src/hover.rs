// NOVA Language Server - Hover Provider
// Provides type/unit information on hover

/// Represents hover content for a symbol
#[derive(Debug, Clone)]
pub struct HoverContent {
    pub symbol: String,
    pub type_info: Option<String>,
    pub unit_info: Option<String>,
    pub documentation: Option<String>,
}

/// The hover provider for NOVA
pub struct HoverProvider {
    // In a full implementation, this would have access to the type checker
    // and semantic analyzer to provide real type information
}

impl HoverProvider {
    pub fn new() -> Self {
        HoverProvider {}
    }

    /// Get hover information for a symbol at a given position
    pub fn get_hover(&self, _line: u32, _col: u32, _source: &str) -> Option<HoverContent> {
        // TODO: Implement proper hover by integrating with type checker
        // For now, return None as placeholder
        None
    }

    /// Create hover content for a variable with unit type
    pub fn create_unit_hover(&self, name: &str, type_name: &str, unit: &str) -> HoverContent {
        HoverContent {
            symbol: name.to_string(),
            type_info: Some(type_name.to_string()),
            unit_info: Some(unit.to_string()),
            documentation: None,
        }
    }

    /// Create hover content for a mission/function
    pub fn create_mission_hover(&self, name: &str, signature: &str, docs: &str) -> HoverContent {
        HoverContent {
            symbol: name.to_string(),
            type_info: Some(signature.to_string()),
            unit_info: None,
            documentation: Some(docs.to_string()),
        }
    }

    /// Create hover content showing tensor shape
    pub fn create_tensor_hover(&self, name: &str, element_type: &str, shape: &[usize]) -> HoverContent {
        let shape_str = shape.iter().map(|d| d.to_string()).collect::<Vec<_>>().join("×");
        HoverContent {
            symbol: name.to_string(),
            type_info: Some(format!("Tensor[{}, {}]", element_type, shape_str)),
            unit_info: None,
            documentation: None,
        }
    }
}

impl Default for HoverProvider {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_unit_hover() {
        let provider = HoverProvider::new();
        let hover = provider.create_unit_hover("velocity", "Float", "m/s");
        assert_eq!(hover.symbol, "velocity");
        assert_eq!(hover.type_info, Some("Float".to_string()));
        assert_eq!(hover.unit_info, Some("m/s".to_string()));
    }

    #[test]
    fn test_create_tensor_hover() {
        let provider = HoverProvider::new();
        let hover = provider.create_tensor_hover("matrix", "Float", &[3, 4]);
        assert!(hover.type_info.unwrap().contains("3×4"));
    }
}