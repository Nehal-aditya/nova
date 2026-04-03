//! NOVA Language Server (nova-ls)
//!
//! LSP implementation for NOVA providing:
//! - Hover with type/unit annotations
//! - Go-to-definition
//! - Unit error highlighting
//! - Mission signature help
//! - Tensor shape tooltips
//! - Auto-import suggestions

pub mod diagnostics;
pub mod hover;

pub use diagnostics::DiagnosticsEngine;
pub use hover::HoverProvider;

/// Start the language server
pub fn start_server() -> Result<(), String> {
    // TODO: Implement full LSP server using tower-lsp
    println!("NOVA Language Server starting...");
    println!("LSP integration pending - basic functionality only");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_server_starts() {
        let result = start_server();
        assert!(result.is_ok());
    }
}