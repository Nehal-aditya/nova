// src/lexer.rs
// NOVA Lexer: converts source text into tokens

use crate::{Error, Result, SourceLoc};

#[derive(Debug, Clone, PartialEq)]
pub enum TokenKind {
    // Literals
    Integer(i64),
    Float(f64),
    UnitAnnotatedFloat { value: f64, unit: String }, // e.g. 3.0[m/s²]
    String(String),
    Bool(bool),

    // Keywords
    Mission,
    Parallel,
    Constellation,
    Absorb,
    Let,
    Var,
    Model,
    Layer,
    Repeat,
    Autodiff,
    Gradient,
    Wrt,
    For,
    In,
    While,
    Match,
    Return,
    Export,
    Transmit,
    If,
    Else,
    Break,
    Pipeline,
    Struct,
    Enum,
    Unit,
    Wave,
    On,
    Device,
    From,
    Test,

    // Operators
    Arrow,       // → (or -> as ASCII)
    Pipe,        // |>
    At,          // @ (matmul)
    Caret,       // ^ (power/xor)
    FatArrow,    // =>
    Dotdot,      // ..
    Question,    // ?
    Plus,        // +
    Minus,       // -
    Star,        // *
    Slash,       // /
    Percent,     // %
    Equals,      // =
    EqEq,        // ==
    NotEq,       // !=
    Less,        // <
    Greater,     // >
    LessEq,      // <=
    GreaterEq,   // >=
    And,         // &&
    Or,          // ||
    Not,         // !
    Ampersand,   // &
    Dot,         // .
    Comma,       // ,
    Colon,       // :
    Semicolon,   // ;

    // Punctuation
    LParen,      // (
    RParen,      // )
    LBrace,      // {
    RBrace,      // }
    LBracket,    // [
    RBracket,    // ]

    // Special
    Ident(String),
    Eof,
}

#[derive(Debug, Clone)]
pub struct Token {
    pub kind: TokenKind,
    pub location: SourceLoc,
}

impl Token {
    pub fn new(kind: TokenKind, location: SourceLoc) -> Self {
        Token { kind, location }
    }
}

pub struct Lexer {
    input: Vec<char>,
    pos: usize,
    line: usize,
    column: usize,
    file_id: usize,
}

impl Lexer {
    pub fn new(input: &str, file_id: usize) -> Self {
        Lexer {
            input: input.chars().collect(),
            pos: 0,
            line: 1,
            column: 1,
            file_id,
        }
    }

    fn current_loc(&self) -> SourceLoc {
        SourceLoc::new(self.file_id, self.line, self.column)
    }

    fn current(&self) -> Option<char> {
        if self.pos < self.input.len() {
            Some(self.input[self.pos])
        } else {
            None
        }
    }

    fn peek(&self, offset: usize) -> Option<char> {
        let pos = self.pos + offset;
        if pos < self.input.len() {
            Some(self.input[pos])
        } else {
            None
        }
    }

    fn advance(&mut self) -> Option<char> {
        if let Some(ch) = self.current() {
            self.pos += 1;
            if ch == '\n' {
                self.line += 1;
                self.column = 1;
            } else {
                self.column += 1;
            }
            Some(ch)
        } else {
            None
        }
    }

    fn skip_whitespace(&mut self) {
        while let Some(ch) = self.current() {
            if ch.is_whitespace() {
                self.advance();
            } else {
                break;
            }
        }
    }

    fn skip_comment(&mut self) {
        // NOVA comments: -- to end of line
        if self.current() == Some('-') && self.peek(1) == Some('-') {
            while let Some(ch) = self.current() {
                if ch == '\n' {
                    break;
                }
                self.advance();
            }
        }
    }

    fn lex_string(&mut self, quote: char) -> Result<String> {
        let start_loc = self.current_loc();
        self.advance(); // consume opening quote

        let mut result = String::new();
        loop {
            match self.current() {
                None => return Err(Error::lex("Unterminated string", start_loc)),
                Some('"') | Some('\'') if self.current() == Some(quote) => {
                    self.advance(); // consume closing quote
                    break;
                }
                Some('\\') => {
                    self.advance();
                    match self.current() {
                        Some('n') => {
                            result.push('\n');
                            self.advance();
                        }
                        Some('t') => {
                            result.push('\t');
                            self.advance();
                        }
                        Some('r') => {
                            result.push('\r');
                            self.advance();
                        }
                        Some('\\') => {
                            result.push('\\');
                            self.advance();
                        }
                        Some('"') => {
                            result.push('"');
                            self.advance();
                        }
                        Some('\'') => {
                            result.push('\'');
                            self.advance();
                        }
                        Some(ch) => {
                            result.push(ch);
                            self.advance();
                        }
                        None => return Err(Error::lex("Unterminated string escape", start_loc)),
                    }
                }
                Some(ch) => {
                    result.push(ch);
                    self.advance();
                }
            }
        }
        Ok(result)
    }

    fn lex_number(&mut self) -> Result<TokenKind> {
        let start_loc = self.current_loc();
        let mut num_str = String::new();
        let mut is_float = false;

        // Collect digits and optional decimal point
        while let Some(ch) = self.current() {
            if ch.is_ascii_digit() {
                num_str.push(ch);
                self.advance();
            } else if ch == '.' && !is_float && self.peek(1).map_or(false, |c| c.is_ascii_digit()) {
                is_float = true;
                num_str.push(ch);
                self.advance();
            } else if ch == 'e' || ch == 'E' {
                is_float = true;
                num_str.push(ch);
                self.advance();
                if self.current() == Some('+') || self.current() == Some('-') {
                    num_str.push(self.current().unwrap());
                    self.advance();
                }
            } else {
                break;
            }
        }

        // Check for unit annotation: [m/s²]
        if self.current() == Some('[') {
            let unit = self.lex_unit()?;
            let value = if is_float {
                num_str
                    .parse::<f64>()
                    .map_err(|_| Error::lex("Invalid float literal", start_loc))?
            } else {
                num_str
                    .parse::<i64>()
                    .map_err(|_| Error::lex("Invalid integer literal", start_loc))?
                    as f64
            };
            return Ok(TokenKind::UnitAnnotatedFloat {
                value,
                unit,
            });
        }

        if is_float {
            let value = num_str
                .parse::<f64>()
                .map_err(|_| Error::lex("Invalid float literal", start_loc))?;
            Ok(TokenKind::Float(value))
        } else {
            let value = num_str
                .parse::<i64>()
                .map_err(|_| Error::lex("Invalid integer literal", start_loc))?;
            Ok(TokenKind::Integer(value))
        }
    }

    fn lex_unit(&mut self) -> Result<String> {
        let start_loc = self.current_loc();
        if self.current() != Some('[') {
            return Err(Error::lex("Expected '[' for unit annotation", start_loc));
        }
        self.advance(); // consume '['

        let mut unit = String::new();
        let mut bracket_count = 1;

        while bracket_count > 0 {
            match self.current() {
                None => return Err(Error::lex("Unterminated unit annotation", start_loc)),
                Some('[') => {
                    unit.push('[');
                    bracket_count += 1;
                    self.advance();
                }
                Some(']') => {
                    bracket_count -= 1;
                    if bracket_count > 0 {
                        unit.push(']');
                    }
                    self.advance();
                }
                Some(ch) => {
                    unit.push(ch);
                    self.advance();
                }
            }
        }

        Ok(unit)
    }

    fn lex_ident(&mut self) -> String {
        let mut ident = String::new();
        while let Some(ch) = self.current() {
            if ch.is_alphanumeric() || ch == '_' {
                ident.push(ch);
                self.advance();
            } else {
                break;
            }
        }
        ident
    }

    fn lex_one(&mut self) -> Result<Option<Token>> {
        self.skip_whitespace();

        // Skip comments
        while self.current() == Some('-') && self.peek(1) == Some('-') {
            self.skip_comment();
            self.skip_whitespace();
        }

        let loc = self.current_loc();

        match self.current() {
            None => Ok(Some(Token::new(TokenKind::Eof, loc))),

            // Single character tokens
            Some('(') => {
                self.advance();
                Ok(Some(Token::new(TokenKind::LParen, loc)))
            }
            Some(')') => {
                self.advance();
                Ok(Some(Token::new(TokenKind::RParen, loc)))
            }
            Some('{') => {
                self.advance();
                Ok(Some(Token::new(TokenKind::LBrace, loc)))
            }
            Some('}') => {
                self.advance();
                Ok(Some(Token::new(TokenKind::RBrace, loc)))
            }
            Some('[') => {
                self.advance();
                Ok(Some(Token::new(TokenKind::LBracket, loc)))
            }
            Some(']') => {
                self.advance();
                Ok(Some(Token::new(TokenKind::RBracket, loc)))
            }
            Some('.') => {
                self.advance();
                if self.current() == Some('.') {
                    self.advance();
                    Ok(Some(Token::new(TokenKind::Dotdot, loc)))
                } else if self.current().map_or(false, |c| c.is_ascii_digit()) {
                    // Backtrack: this is a float like .5
                    self.pos -= 1;
                    self.column -= 1;
                    let kind = self.lex_number()?;
                    Ok(Some(Token::new(kind, loc)))
                } else {
                    Ok(Some(Token::new(TokenKind::Dot, loc)))
                }
            }
            Some(',') => {
                self.advance();
                Ok(Some(Token::new(TokenKind::Comma, loc)))
            }
            Some(':') => {
                self.advance();
                Ok(Some(Token::new(TokenKind::Colon, loc)))
            }
            Some(';') => {
                self.advance();
                Ok(Some(Token::new(TokenKind::Semicolon, loc)))
            }
            Some('?') => {
                self.advance();
                Ok(Some(Token::new(TokenKind::Question, loc)))
            }
            Some('^') => {
                self.advance();
                Ok(Some(Token::new(TokenKind::Caret, loc)))
            }
            Some('@') => {
                self.advance();
                Ok(Some(Token::new(TokenKind::At, loc)))
            }
            Some('%') => {
                self.advance();
                Ok(Some(Token::new(TokenKind::Percent, loc)))
            }

            // Multi-character operators
            Some('+') => {
                self.advance();
                Ok(Some(Token::new(TokenKind::Plus, loc)))
            }
            Some('-') => {
                self.advance();
                if self.current() == Some('>') {
                    self.advance();
                    Ok(Some(Token::new(TokenKind::Arrow, loc)))
                } else if self.current().map_or(false, |c| c.is_ascii_digit()) {
                    // Negative number
                    let kind = self.lex_number()?;
                    match kind {
                        TokenKind::Integer(n) => {
                            Ok(Some(Token::new(TokenKind::Integer(-n), loc)))
                        }
                        TokenKind::Float(f) => {
                            Ok(Some(Token::new(TokenKind::Float(-f), loc)))
                        }
                        TokenKind::UnitAnnotatedFloat { value, unit } => {
                            Ok(Some(Token::new(
                                TokenKind::UnitAnnotatedFloat {
                                    value: -value,
                                    unit,
                                },
                                loc,
                            )))
                        }
                        _ => unreachable!(),
                    }
                } else {
                    Ok(Some(Token::new(TokenKind::Minus, loc)))
                }
            }
            Some('*') => {
                self.advance();
                Ok(Some(Token::new(TokenKind::Star, loc)))
            }
            Some('/') => {
                self.advance();
                Ok(Some(Token::new(TokenKind::Slash, loc)))
            }
            Some('=') => {
                self.advance();
                if self.current() == Some('=') {
                    self.advance();
                    Ok(Some(Token::new(TokenKind::EqEq, loc)))
                } else if self.current() == Some('>') {
                    self.advance();
                    Ok(Some(Token::new(TokenKind::FatArrow, loc)))
                } else {
                    Ok(Some(Token::new(TokenKind::Equals, loc)))
                }
            }
            Some('!') => {
                self.advance();
                if self.current() == Some('=') {
                    self.advance();
                    Ok(Some(Token::new(TokenKind::NotEq, loc)))
                } else {
                    Ok(Some(Token::new(TokenKind::Not, loc)))
                }
            }
            Some('<') => {
                self.advance();
                if self.current() == Some('=') {
                    self.advance();
                    Ok(Some(Token::new(TokenKind::LessEq, loc)))
                } else {
                    Ok(Some(Token::new(TokenKind::Less, loc)))
                }
            }
            Some('>') => {
                self.advance();
                if self.current() == Some('=') {
                    self.advance();
                    Ok(Some(Token::new(TokenKind::GreaterEq, loc)))
                } else {
                    Ok(Some(Token::new(TokenKind::Greater, loc)))
                }
            }
            Some('&') => {
                self.advance();
                if self.current() == Some('&') {
                    self.advance();
                    Ok(Some(Token::new(TokenKind::And, loc)))
                } else {
                    Ok(Some(Token::new(TokenKind::Ampersand, loc)))
                }
            }
            Some('|') => {
                self.advance();
                if self.current() == Some('>') {
                    self.advance();
                    Ok(Some(Token::new(TokenKind::Pipe, loc)))
                } else if self.current() == Some('|') {
                    self.advance();
                    Ok(Some(Token::new(TokenKind::Or, loc)))
                } else {
                    Err(Error::lex(
                        "Unexpected '|' — did you mean '|>' (pipe)?",
                        loc,
                    ))
                }
            }
            Some('→') => {
                self.advance();
                Ok(Some(Token::new(TokenKind::Arrow, loc)))
            }

            Some('"') | Some('\'') => {
                let quote = self.current().unwrap();
                let string = self.lex_string(quote)?;
                Ok(Some(Token::new(TokenKind::String(string), loc)))
            }

            Some(ch) if ch.is_ascii_digit() => {
                let kind = self.lex_number()?;
                Ok(Some(Token::new(kind, loc)))
            }

            Some(ch) if ch.is_alphabetic() || ch == '_' => {
                let ident = self.lex_ident();
                let kind = match ident.as_str() {
                    "mission" => TokenKind::Mission,
                    "parallel" => TokenKind::Parallel,
                    "constellation" => TokenKind::Constellation,
                    "absorb" => TokenKind::Absorb,
                    "let" => TokenKind::Let,
                    "var" => TokenKind::Var,
                    "model" => TokenKind::Model,
                    "layer" => TokenKind::Layer,
                    "repeat" => TokenKind::Repeat,
                    "autodiff" => TokenKind::Autodiff,
                    "gradient" => TokenKind::Gradient,
                    "wrt" => TokenKind::Wrt,
                    "for" => TokenKind::For,
                    "in" => TokenKind::In,
                    "while" => TokenKind::While,
                    "match" => TokenKind::Match,
                    "return" => TokenKind::Return,
                    "export" => TokenKind::Export,
                    "transmit" => TokenKind::Transmit,
                    "if" => TokenKind::If,
                    "else" => TokenKind::Else,
                    "break" => TokenKind::Break,
                    "pipeline" => TokenKind::Pipeline,
                    "struct" => TokenKind::Struct,
                    "enum" => TokenKind::Enum,
                    "unit" => TokenKind::Unit,
                    "wave" => TokenKind::Wave,
                    "on" => TokenKind::On,
                    "device" => TokenKind::Device,
                    "from" => TokenKind::From,
                    "test" => TokenKind::Test,
                    "true" => TokenKind::Bool(true),
                    "false" => TokenKind::Bool(false),
                    _ => TokenKind::Ident(ident),
                };
                Ok(Some(Token::new(kind, loc)))
            }

            Some(ch) => Err(Error::lex(format!("Unexpected character: '{}'", ch), loc)),
        }
    }

    pub fn tokenize(&mut self) -> Result<Vec<Token>> {
        let mut tokens = Vec::new();
        loop {
            if let Some(token) = self.lex_one()? {
                let is_eof = matches!(token.kind, TokenKind::Eof);
                tokens.push(token);
                if is_eof {
                    break;
                }
            }
        }
        Ok(tokens)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_mission() {
        let mut lexer = Lexer::new("mission foo() → Void {}", 0);
        let tokens = lexer.tokenize().unwrap();
        assert!(matches!(tokens[0].kind, TokenKind::Mission));
        assert!(matches!(tokens[1].kind, TokenKind::Ident(ref s) if s == "foo"));
        assert!(matches!(tokens[2].kind, TokenKind::LParen));
    }

    #[test]
    fn test_unit_annotation() {
        let mut lexer = Lexer::new("let x = 3.0[m/s²]", 0);
        let tokens = lexer.tokenize().unwrap();
        assert!(matches!(tokens[0].kind, TokenKind::Let));
        match &tokens[4].kind {
            TokenKind::UnitAnnotatedFloat { value, unit } => {
                assert_eq!(*value, 3.0);
                assert!(unit.contains("m/s"));
            }
            _ => panic!("Expected unit annotated float"),
        }
    }
}
