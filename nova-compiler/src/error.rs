// src/error.rs
// Error type and Result for NOVA compiler

use crate::SourceLoc;
use std::fmt;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Clone)]
pub struct Error {
    pub message: String,
    pub location: SourceLoc,
    pub kind: ErrorKind,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ErrorKind {
    LexError,
    ParseError,
    TypeError,
    UnitError,
}

impl Error {
    pub fn new(kind: ErrorKind, message: impl Into<String>, location: SourceLoc) -> Self {
        Error {
            message: message.into(),
            location,
            kind,
        }
    }

    pub fn lex(message: impl Into<String>, location: SourceLoc) -> Self {
        Error::new(ErrorKind::LexError, message, location)
    }

    pub fn parse(message: impl Into<String>, location: SourceLoc) -> Self {
        Error::new(ErrorKind::ParseError, message, location)
    }

    pub fn type_error(message: impl Into<String>, location: SourceLoc) -> Self {
        Error::new(ErrorKind::TypeError, message, location)
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}  [{}:{}:{}]\n  {}",
            match self.kind {
                ErrorKind::LexError => "LexError",
                ErrorKind::ParseError => "ParseError",
                ErrorKind::TypeError => "TypeError",
                ErrorKind::UnitError => "UnitError",
            },
            self.location.file,
            self.location.line,
            self.location.column,
            self.message
        )
    }
}

impl std::error::Error for Error {}
