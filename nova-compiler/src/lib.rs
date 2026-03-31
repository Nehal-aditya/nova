// src/lib.rs
// NOVA Compiler - Phase 0: Lexer + Parser + AST

pub mod lexer;
pub mod parser;
pub mod ast;
pub mod error;

pub use lexer::Lexer;
pub use parser::Parser;
pub use ast::*;
pub use error::{Error, Result};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SourceLoc {
    pub file: usize,       // file ID
    pub line: usize,       // 1-indexed
    pub column: usize,     // 1-indexed
}

impl SourceLoc {
    pub fn new(file: usize, line: usize, column: usize) -> Self {
        SourceLoc { file, line, column }
    }

    pub fn builtin() -> Self {
        SourceLoc { file: 0, line: 0, column: 0 }
    }
}

impl std::fmt::Display for SourceLoc {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{}:{}", self.file, self.line, self.column)
    }
}
