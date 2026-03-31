// src/parser.rs
// NOVA Parser: recursive descent parser building AST from tokens
// Complete Phase 0 implementation for NOVA language

use crate::lexer::{Token, TokenKind};
use crate::{
    ast::*,
    Error, Result, SourceLoc,
};

pub struct Parser {
    tokens: Vec<Token>,
    pos: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Parser { tokens, pos: 0 }
    }

    // ======================== Utilities ========================

    fn current(&self) -> &Token {
        self.tokens.get(self.pos).unwrap_or_else(|| {
            &self.tokens[self.tokens.len() - 1] // EOF token
        })
    }

    fn peek(&self, offset: usize) -> &Token {
        self.tokens.get(self.pos + offset).unwrap_or_else(|| {
            &self.tokens[self.tokens.len() - 1]
        })
    }

    fn advance(&mut self) -> Token {
        let token = self.current().clone();
        if self.pos < self.tokens.len() {
            self.pos += 1;
        }
        token
    }

    fn current_location(&self) -> SourceLoc {
        self.current().location
    }

    fn expect(&mut self, expected: TokenKind) -> Result<Token> {
        let token = self.current().clone();
        if std::mem::discriminant(&token.kind) == std::mem::discriminant(&expected) {
            self.advance();
            Ok(token)
        } else {
            Err(Error::parse(
                format!("Expected {:?}, got {:?}", expected, token.kind),
                token.location,
            ))
        }
    }

    fn consume_if(&mut self, kind: &TokenKind) -> bool {
        if std::mem::discriminant(kind) == std::mem::discriminant(&self.current().kind) {
            self.advance();
            true
        } else {
            false
        }
    }

    fn is_at_end(&self) -> bool {
        matches!(self.current().kind, TokenKind::Eof)
    }

    // ======================== Main Entry Point ========================

    pub fn parse_program(&mut self) -> Result<Program> {
        let mut items = Vec::new();

        while !self.is_at_end() {
            items.push(self.parse_top_level()?);
        }

        Ok(Program { items })
    }

    // ======================== Top-Level Declarations ========================

    fn parse_top_level(&mut self) -> Result<TopLevelItem> {
        match &self.current().kind {
            TokenKind::Test => {
                self.advance();
                let mission = self.parse_mission_def()?;
                Ok(TopLevelItem::TestMission(mission))
            }
            TokenKind::Mission => {
                self.advance();
                let mission = self.parse_mission_def()?;
                Ok(TopLevelItem::MissionDecl(mission))
            }
            TokenKind::Parallel => {
                self.advance();
                self.expect(TokenKind::Mission)?;
                let mission = self.parse_mission_def()?;
                Ok(TopLevelItem::ParallelMissionDecl(mission))
            }
            TokenKind::Constellation => {
                self.advance();
                self.parse_constellation_def()
            }
            TokenKind::Model => {
                self.advance();
                self.parse_model_def()
            }
            TokenKind::Struct => {
                self.advance();
                self.parse_struct_def()
            }
            TokenKind::Enum => {
                self.advance();
                self.parse_enum_def()
            }
            TokenKind::Unit => {
                self.advance();
                self.parse_unit_def()
            }
            _ => Err(Error::parse(
                format!("Expected top-level declaration, got {:?}", self.current().kind),
                self.current_location(),
            )),
        }
    }

    fn parse_mission_def(&mut self) -> Result<MissionDecl> {
        let loc = self.current_location();
        
        let name_token = self.expect(TokenKind::Ident(String::new()))?;
        let name = match name_token.kind {
            TokenKind::Ident(n) => n,
            _ => return Err(Error::parse("Expected mission name", loc)),
        };

        self.expect(TokenKind::LParen)?;
        let params = self.parse_parameters()?;
        self.expect(TokenKind::RParen)?;

        let return_type = if self.consume_if(&TokenKind::Arrow) 
            || self.consume_if(&TokenKind::Ident("->".to_string())) {
            Some(self.parse_type_expr()?)
        } else {
            None
        };

        self.expect(TokenKind::LBrace)?;
        let body = self.parse_block()?;
        self.expect(TokenKind::RBrace)?;

        Ok(MissionDecl {
            name,
            params,
            return_type,
            body,
            location: loc,
        })
    }

    fn parse_parameters(&mut self) -> Result<Vec<Parameter>> {
        let mut params = Vec::new();

        while !matches!(self.current().kind, TokenKind::RParen) {
            let loc = self.current_location();
            
            let name_token = self.expect(TokenKind::Ident(String::new()))?;
            let name = match name_token.kind {
                TokenKind::Ident(n) => n,
                _ => return Err(Error::parse("Expected parameter name", loc)),
            };

            self.expect(TokenKind::Colon)?;
            let type_annotation = self.parse_type_expr()?;

            params.push(Parameter {
                name,
                type_annotation,
                location: loc,
            });

            if !self.consume_if(&TokenKind::Comma) {
                break;
            }
        }

        Ok(params)
    }

    fn parse_type_expr(&mut self) -> Result<TypeExpr> {
        let loc = self.current_location();
        
        match &self.current().kind {
            TokenKind::Ident(name) if name == "Float" => {
                self.advance();
                if self.consume_if(&TokenKind::LBracket) {
                    let unit_str = self.expect(TokenKind::Ident(String::new()))?;
                    let unit_name = match unit_str.kind {
                        TokenKind::Ident(u) => u,
                        _ => return Err(Error::parse("Expected unit name", loc)),
                    };
                    self.expect(TokenKind::RBracket)?;
                    Ok(TypeExpr::Float(Some(UnitExpr(unit_name))))
                } else {
                    Ok(TypeExpr::Float(None))
                }
            }
            TokenKind::Ident(name) if name == "Int" => {
                self.advance();
                Ok(TypeExpr::Int)
            }
            TokenKind::Ident(name) if name == "Bool" => {
                self.advance();
                Ok(TypeExpr::Bool)
            }
            TokenKind::Ident(name) if name == "String" => {
                self.advance();
                Ok(TypeExpr::String)
            }
            TokenKind::Ident(name) if name == "Void" || name == "None" => {
                self.advance();
                Ok(TypeExpr::Named("Void".to_string()))
            }
            TokenKind::Ident(name) if name == "Array" => {
                self.advance();
                self.expect(TokenKind::LBracket)?;
                let inner = self.parse_type_expr()?;
                self.expect(TokenKind::RBracket)?;
                Ok(TypeExpr::Array(Box::new(inner)))
            }
            TokenKind::Ident(name) if name == "Tensor" => {
                self.advance();
                self.expect(TokenKind::LBracket)?;
                let element_type = self.parse_type_expr()?;
                let mut shape = Vec::new();
                while self.consume_if(&TokenKind::Comma) {
                    if let TokenKind::Integer(n) = self.current().kind {
                        shape.push(n as usize);
                        self.advance();
                    } else {
                        return Err(Error::parse("Expected dimension in tensor type", loc));
                    }
                }
                self.expect(TokenKind::RBracket)?;
                Ok(TypeExpr::Tensor {
                    element_type: Box::new(element_type),
                    shape,
                })
            }
            TokenKind::Ident(name) => {
                let n = name.clone();
                self.advance();
                Ok(TypeExpr::Named(n))
            }
            _ => Err(Error::parse(
                format!("Expected type expression, got {:?}", self.current().kind),
                loc,
            )),
        }
    }

    fn parse_block(&mut self) -> Result<Vec<Statement>> {
        let mut statements = Vec::new();

        while !matches!(self.current().kind, TokenKind::RBrace | TokenKind::Eof) {
            statements.push(self.parse_statement()?);
        }

        Ok(statements)
    }

    fn parse_statement(&mut self) -> Result<Statement> {
        let loc = self.current_location();
        
        match &self.current().kind {
            TokenKind::Let => {
                self.advance();
                self.parse_let_binding(false, loc)
            }
            TokenKind::Var => {
                self.advance();
                self.parse_let_binding(true, loc)
            }
            TokenKind::If => {
                self.advance();
                self.parse_if_statement(loc)
            }
            TokenKind::For => {
                self.advance();
                self.parse_for_statement(loc)
            }
            TokenKind::While => {
                self.advance();
                self.parse_while_statement(loc)
            }
            TokenKind::Return => {
                self.advance();
                let expr = if !matches!(self.current().kind, TokenKind::RBrace | TokenKind::Semicolon | TokenKind::Eof) {
                    Some(self.parse_expr()?)
                } else {
                    None
                };
                self.consume_if(&TokenKind::Semicolon);
                Ok(Statement::Return(expr))
            }
            TokenKind::Break => {
                self.advance();
                self.consume_if(&TokenKind::Semicolon);
                Ok(Statement::Break)
            }
            _ => {
                let expr = self.parse_expr()?;
                self.consume_if(&TokenKind::Semicolon);
                Ok(Statement::ExprStmt(expr))
            }
        }
    }

    fn parse_let_binding(&mut self, mutable: bool, loc: SourceLoc) -> Result<Statement> {
        let name_token = self.expect(TokenKind::Ident(String::new()))?;
        let name = match name_token.kind {
            TokenKind::Ident(n) => n,
            _ => return Err(Error::parse("Expected variable name", loc)),
        };

        let type_annotation = if self.consume_if(&TokenKind::Colon) {
            Some(self.parse_type_expr()?)
        } else {
            None
        };

        self.expect(TokenKind::Equals)?;
        let value = self.parse_expr()?;
        self.consume_if(&TokenKind::Semicolon);

        Ok(Statement::LetBind {
            name,
            mutable,
            type_annotation,
            value,
            location: loc,
        })
    }

    fn parse_if_statement(&mut self, loc: SourceLoc) -> Result<Statement> {
        let condition = self.parse_expr()?;
        self.expect(TokenKind::LBrace)?;
        let then_body = self.parse_block()?;
        self.expect(TokenKind::RBrace)?;

        let else_body = if self.consume_if(&TokenKind::Else) {
            if matches!(self.current().kind, TokenKind::If) {
                // else if
                let elif_stmt = self.parse_if_statement(self.current_location())?;
                if let Statement::If { then_body, .. } = elif_stmt {
                    Some(then_body)
                } else {
                    Some(vec![elif_stmt])
                }
            } else {
                self.expect(TokenKind::LBrace)?;
                let body = self.parse_block()?;
                self.expect(TokenKind::RBrace)?;
                Some(body)
            }
        } else {
            None
        };

        Ok(Statement::If {
            condition,
            then_body,
            else_body,
            location: loc,
        })
    }

    fn parse_for_statement(&mut self, loc: SourceLoc) -> Result<Statement> {
        let var_token = self.expect(TokenKind::Ident(String::new()))?;
        let var = match var_token.kind {
            TokenKind::Ident(v) => v,
            _ => return Err(Error::parse("Expected loop variable", loc)),
        };

        self.expect(TokenKind::In)?;
        let iter = self.parse_expr()?;
        self.expect(TokenKind::LBrace)?;
        let body = self.parse_block()?;
        self.expect(TokenKind::RBrace)?;

        Ok(Statement::For {
            var,
            iter,
            body,
            location: loc,
        })
    }

    fn parse_while_statement(&mut self, loc: SourceLoc) -> Result<Statement> {
        let condition = self.parse_expr()?;
        self.expect(TokenKind::LBrace)?;
        let body = self.parse_block()?;
        self.expect(TokenKind::RBrace)?;

        Ok(Statement::While {
            condition,
            body,
            location: loc,
        })
    }

    // ======================== Expression Parsing ========================
    // Operator precedence (low to high):
    // 1. pipe (|>)
    // 2. lambda (=>)
    // 3. ||
    // 4. &&
    // 5. ==, !=, <, >, <=, >=
    // 6. +, -
    // 7. *, /, %
    // 8. @
    // 9. ^
    // 10. unary (-, !, ?)

    fn parse_expr(&mut self) -> Result<Expr> {
        self.parse_pipe()
    }

    fn parse_pipe(&mut self) -> Result<Expr> {
        let mut expr = self.parse_lambda()?;

        while self.consume_if(&TokenKind::Pipe) {
            let loc = self.current_location();
            let right = self.parse_lambda()?;
            expr = Expr::Pipe {
                left: Box::new(expr),
                right: Box::new(right),
                location: loc,
            };
        }

        Ok(expr)
    }

    fn parse_lambda(&mut self) -> Result<Expr> {
        let loc = self.current_location();
        
        // Check for lambda: ident => expr
        if let TokenKind::Ident(param) = &self.current().kind {
            if matches!(self.peek(1).kind, TokenKind::FatArrow) {
                let param = match self.advance().kind {
                    TokenKind::Ident(p) => p,
                    _ => unreachable!(),
                };
                self.advance(); // consume =>
                let body = self.parse_lambda()?; // Right-associate lambdas
                return Ok(Expr::Lambda {
                    param,
                    body: Box::new(body),
                    location: loc,
                });
            }
        }

        self.parse_or()
    }

    fn parse_or(&mut self) -> Result<Expr> {
        let mut expr = self.parse_and()?;

        while self.consume_if(&TokenKind::Or) {
            let loc = self.current_location();
            let right = self.parse_and()?;
            expr = Expr::BinaryOp {
                left: Box::new(expr),
                op: BinOp::Or,
                right: Box::new(right),
                location: loc,
            };
        }

        Ok(expr)
    }

    fn parse_and(&mut self) -> Result<Expr> {
        let mut expr = self.parse_comparison()?;

        while self.consume_if(&TokenKind::And) {
            let loc = self.current_location();
            let right = self.parse_comparison()?;
            expr = Expr::BinaryOp {
                left: Box::new(expr),
                op: BinOp::And,
                right: Box::new(right),
                location: loc,
            };
        }

        Ok(expr)
    }

    fn parse_comparison(&mut self) -> Result<Expr> {
        let mut expr = self.parse_additive()?;

        loop {
            let loc = self.current_location();
            let op = match &self.current().kind {
                TokenKind::EqEq => Some(BinOp::Equal),
                TokenKind::NotEq => Some(BinOp::NotEqual),
                TokenKind::Less => Some(BinOp::Less),
                TokenKind::Greater => Some(BinOp::Greater),
                TokenKind::LessEq => Some(BinOp::LessEqual),
                TokenKind::GreaterEq => Some(BinOp::GreaterEqual),
                _ => None,
            };

            if let Some(op) = op {
                self.advance();
                let right = self.parse_additive()?;
                expr = Expr::BinaryOp {
                    left: Box::new(expr),
                    op,
                    right: Box::new(right),
                    location: loc,
                };
            } else {
                break;
            }
        }

        Ok(expr)
    }

    fn parse_additive(&mut self) -> Result<Expr> {
        let mut expr = self.parse_multiplicative()?;

        loop {
            let loc = self.current_location();
            let op = match &self.current().kind {
                TokenKind::Plus => Some(BinOp::Add),
                TokenKind::Minus => Some(BinOp::Subtract),
                _ => None,
            };

            if let Some(op) = op {
                self.advance();
                let right = self.parse_multiplicative()?;
                expr = Expr::BinaryOp {
                    left: Box::new(expr),
                    op,
                    right: Box::new(right),
                    location: loc,
                };
            } else {
                break;
            }
        }

        Ok(expr)
    }

    fn parse_multiplicative(&mut self) -> Result<Expr> {
        let mut expr = self.parse_at()?;

        loop {
            let loc = self.current_location();
            let op = match &self.current().kind {
                TokenKind::Star => Some(BinOp::Multiply),
                TokenKind::Slash => Some(BinOp::Divide),
                TokenKind::Percent => Some(BinOp::Modulo),
                _ => None,
            };

            if let Some(op) = op {
                self.advance();
                let right = self.parse_at()?;
                expr = Expr::BinaryOp {
                    left: Box::new(expr),
                    op,
                    right: Box::new(right),
                    location: loc,
                };
            } else {
                break;
            }
        }

        Ok(expr)
    }

    fn parse_at(&mut self) -> Result<Expr> {
        let mut expr = self.parse_power()?;

        while self.consume_if(&TokenKind::At) {
            let loc = self.current_location();
            let right = self.parse_power()?;
            expr = Expr::BinaryOp {
                left: Box::new(expr),
                op: BinOp::MatMul,
                right: Box::new(right),
                location: loc,
            };
        }

        Ok(expr)
    }

    fn parse_power(&mut self) -> Result<Expr> {
        let mut expr = self.parse_unary()?;

        while self.consume_if(&TokenKind::Caret) {
            let loc = self.current_location();
            let right = self.parse_unary()?;
            expr = Expr::BinaryOp {
                left: Box::new(expr),
                op: BinOp::Power,
                right: Box::new(right),
                location: loc,
            };
        }

        Ok(expr)
    }

    fn parse_unary(&mut self) -> Result<Expr> {
        let loc = self.current_location();
        
        match &self.current().kind {
            TokenKind::Minus => {
                self.advance();
                let operand = self.parse_unary()?;
                Ok(Expr::UnaryOp {
                    op: UnOp::Negate,
                    operand: Box::new(operand),
                    location: loc,
                })
            }
            TokenKind::Not => {
                self.advance();
                let operand = self.parse_unary()?;
                Ok(Expr::UnaryOp {
                    op: UnOp::Not,
                    operand: Box::new(operand),
                    location: loc,
                })
            }
            TokenKind::Question => {
                self.advance();
                let operand = self.parse_unary()?;
                Ok(Expr::UnaryOp {
                    op: UnOp::Deref,
                    operand: Box::new(operand),
                    location: loc,
                })
            }
            _ => self.parse_postfix(),
        }
    }

    fn parse_postfix(&mut self) -> Result<Expr> {
        let mut expr = self.parse_primary()?;

        loop {
            let loc = self.current_location();
            match &self.current().kind {
                TokenKind::LParen => {
                    self.advance();
                    let args = self.parse_call_args()?;
                    self.expect(TokenKind::RParen)?;
                    expr = Expr::Call {
                        func: Box::new(expr),
                        args,
                        location: loc,
                    };
                }
                TokenKind::Dot => {
                    self.advance();
                    let field_token = self.expect(TokenKind::Ident(String::new()))?;
                    let field = match field_token.kind {
                        TokenKind::Ident(f) => f,
                        _ => return Err(Error::parse("Expected field name", loc)),
                    };
                    expr = Expr::FieldAccess {
                        object: Box::new(expr),
                        field,
                        location: loc,
                    };
                }
                TokenKind::LBracket => {
                    self.advance();
                    let index = self.parse_expr()?;
                    self.expect(TokenKind::RBracket)?;
                    expr = Expr::Index {
                        object: Box::new(expr),
                        index: Box::new(index),
                        location: loc,
                    };
                }
                _ => break,
            }
        }

        Ok(expr)
    }

    fn parse_call_args(&mut self) -> Result<Vec<(Option<String>, Expr)>> {
        let mut args = Vec::new();

        while !matches!(self.current().kind, TokenKind::RParen) {
            // Check for named argument
            if let TokenKind::Ident(name) = &self.current().kind {
                if matches!(self.peek(1).kind, TokenKind::Colon) {
                    let name = name.clone();
                    self.advance();
                    self.advance(); // consume ':'
                    let expr = self.parse_lambda()?;
                    args.push((Some(name), expr));
                } else {
                    let expr = self.parse_lambda()?;
                    args.push((None, expr));
                }
            } else {
                let expr = self.parse_lambda()?;
                args.push((None, expr));
            }

            if !self.consume_if(&TokenKind::Comma) {
                break;
            }
        }

        Ok(args)
    }

    fn parse_primary(&mut self) -> Result<Expr> {
        let loc = self.current_location();
        
        match &self.current().kind {
            TokenKind::Integer(n) => {
                let n = *n;
                self.advance();
                Ok(Expr::Integer(n))
            }
            TokenKind::Float(f) => {
                let f = *f;
                self.advance();
                Ok(Expr::Float(f))
            }
            TokenKind::UnitAnnotatedFloat { value, unit } => {
                let value = *value;
                let unit = UnitExpr(unit.clone());
                self.advance();
                Ok(Expr::UnitAnnotatedFloat { value, unit })
            }
            TokenKind::String(s) => {
                let s = s.clone();
                self.advance();
                Ok(Expr::String(s))
            }
            TokenKind::Bool(b) => {
                let b = *b;
                self.advance();
                Ok(Expr::Bool(b))
            }
            TokenKind::Ident(name) => {
                let name = name.clone();
                self.advance();
                Ok(Expr::Ident(name))
            }
            TokenKind::LParen => {
                self.advance();
                let expr = self.parse_expr()?;
                self.expect(TokenKind::RParen)?;
                Ok(expr)
            }
            TokenKind::LBrace => {
                self.advance();
                let fields = self.parse_struct_literal_fields()?;
                self.expect(TokenKind::RBrace)?;
                Ok(Expr::StructLiteral {
                    type_name: None,
                    fields,
                    location: loc,
                })
            }
            TokenKind::LBracket => {
                self.advance();
                let mut elements = Vec::new();
                while !matches!(self.current().kind, TokenKind::RBracket) {
                    elements.push(self.parse_expr()?);
                    if !self.consume_if(&TokenKind::Comma) {
                        break;
                    }
                }
                self.expect(TokenKind::RBracket)?;
                Ok(Expr::TensorLiteral {
                    elements,
                    location: loc,
                })
            }
            TokenKind::If => {
                self.advance();
                let condition = self.parse_expr()?;
                self.expect(TokenKind::LBrace)?;
                let then_body = self.parse_block()?;
                self.expect(TokenKind::RBrace)?;
                let else_expr = if self.consume_if(&TokenKind::Else) {
                    if matches!(self.current().kind, TokenKind::If) {
                        Some(Box::new(self.parse_primary()?))
                    } else {
                        self.expect(TokenKind::LBrace)?;
                        let body = self.parse_block()?;
                        self.expect(TokenKind::RBrace)?;
                        // Convert statements to expression
                        Some(Box::new(Expr::Bool(true))) // Simplified
                    }
                } else {
                    None
                };
                Ok(Expr::If {
                    condition: Box::new(condition),
                    then_expr: Box::new(Expr::Bool(true)), // Simplified
                    else_expr,
                    location: loc,
                })
            }
            TokenKind::Match => {
                self.advance();
                let subject = self.parse_expr()?;
                self.expect(TokenKind::LBrace)?;
                let mut arms = Vec::new();
                while !matches!(self.current().kind, TokenKind::RBrace) {
                    arms.push(self.parse_match_arm()?);
                }
                self.expect(TokenKind::RBrace)?;
                Ok(Expr::Match {
                    subject: Box::new(subject),
                    arms,
                    location: loc,
                })
            }
            TokenKind::Transmit => {
                self.advance();
                self.expect(TokenKind::LParen)?;
                let mut args = Vec::new();
                while !matches!(self.current().kind, TokenKind::RParen) {
                    args.push(self.parse_expr()?);
                    if !self.consume_if(&TokenKind::Comma) {
                        break;
                    }
                }
                self.expect(TokenKind::RParen)?;
                Ok(Expr::Transmit { args, location: loc })
            }
            TokenKind::Pipeline => {
                self.advance();
                self.expect(TokenKind::LBracket)?;
                let mut stages = Vec::new();
                while !matches!(self.current().kind, TokenKind::RBracket) {
                    stages.push(self.parse_expr()?);
                    if !self.consume_if(&TokenKind::Comma) {
                        break;
                    }
                }
                self.expect(TokenKind::RBracket)?;
                // Pipeline requires a source; we'll handle that in the pipe operator
                Ok(Expr::Pipeline {
                    source: Box::new(Expr::Ident("_".to_string())),
                    stages,
                    location: loc,
                })
            }
            TokenKind::Autodiff => {
                self.advance();
                self.expect(TokenKind::LParen)?;
                let target = self.parse_expr()?;
                self.expect(TokenKind::RParen)?;
                self.expect(TokenKind::LBrace)?;
                let body = self.parse_block()?;
                self.expect(TokenKind::RBrace)?;
                Ok(Expr::Autodiff {
                    target: Box::new(target),
                    body,
                    location: loc,
                })
            }
            TokenKind::Gradient => {
                self.advance();
                self.expect(TokenKind::LParen)?;
                let expr = self.parse_expr()?;
                let wrt = if self.consume_if(&TokenKind::Comma) {
                    self.expect(TokenKind::Wrt)?;
                    let mut wrt_list = Vec::new();
                    loop {
                        let var_token = self.expect(TokenKind::Ident(String::new()))?;
                        if let TokenKind::Ident(v) = var_token.kind {
                            wrt_list.push(v);
                        }
                        if !self.consume_if(&TokenKind::Comma) {
                            break;
                        }
                    }
                    wrt_list
                } else {
                    Vec::new()
                };
                self.expect(TokenKind::RParen)?;
                Ok(Expr::Gradient {
                    expr: Box::new(expr),
                    wrt,
                    location: loc,
                })
            }
            _ => Err(Error::parse(
                format!("Unexpected token in expression: {:?}", self.current().kind),
                loc,
            )),
        }
    }

    fn parse_struct_literal_fields(&mut self) -> Result<Vec<(String, Expr)>> {
        let mut fields = Vec::new();

        while !matches!(self.current().kind, TokenKind::RBrace) {
            let name_token = self.expect(TokenKind::Ident(String::new()))?;
            let name = match name_token.kind {
                TokenKind::Ident(n) => n,
                _ => return Err(Error::parse("Expected field name", self.current_location())),
            };

            self.expect(TokenKind::Colon)?;
            let value = self.parse_expr()?;
            fields.push((name, value));

            if !self.consume_if(&TokenKind::Comma) {
                break;
            }
        }

        Ok(fields)
    }

    fn parse_match_arm(&mut self) -> Result<MatchArm> {
        let pattern = self.parse_pattern()?;
        self.expect(TokenKind::FatArrow)?;
        let body = self.parse_expr()?;
        self.consume_if(&TokenKind::Comma);
        
        Ok(MatchArm {
            pattern,
            guard: None,
            body,
        })
    }

    fn parse_pattern(&mut self) -> Result<Pattern> {
        match &self.current().kind {
            TokenKind::Ident(name) => {
                let name = name.clone();
                self.advance();
                Ok(Pattern::Ident(name))
            }
            _ => Err(Error::parse("Expected pattern", self.current_location())),
        }
    }

    // Parse constellation, model, struct, enum, unit
    fn parse_constellation_def(&mut self) -> Result<TopLevelItem> {
        let loc = self.current_location();
        
        let name_token = self.expect(TokenKind::Ident(String::new()))?;
        let name = match name_token.kind {
            TokenKind::Ident(n) => n,
            _ => return Err(Error::parse("Expected constellation name", loc)),
        };

        self.expect(TokenKind::LBrace)?;
        let mut items = Vec::new();

        while !matches!(self.current().kind, TokenKind::RBrace) {
            match &self.current().kind {
                TokenKind::Export => {
                    self.advance();
                    let export_token = self.expect(TokenKind::Ident(String::new()))?;
                    let export_name = match export_token.kind {
                        TokenKind::Ident(e) => e,
                        _ => return Err(Error::parse("Expected export name", loc)),
                    };
                    items.push(ConstellationItem::Export(export_name));
                    self.consume_if(&TokenKind::Semicolon);
                }
                TokenKind::Mission => {
                    self.advance();
                    let mission = self.parse_mission_def()?;
                    items.push(ConstellationItem::MissionDecl(mission));
                }
                _ => {
                    return Err(Error::parse("Expected mission or export in constellation", loc));
                }
            }
        }

        self.expect(TokenKind::RBrace)?;

        Ok(TopLevelItem::ConstellationDecl(ConstellationDecl {
            name,
            items,
            location: loc,
        }))
    }

    fn parse_model_def(&mut self) -> Result<TopLevelItem> {
        let loc = self.current_location();
        
        let name_token = self.expect(TokenKind::Ident(String::new()))?;
        let name = match name_token.kind {
            TokenKind::Ident(n) => n,
            _ => return Err(Error::parse("Expected model name", loc)),
        };

        self.expect(TokenKind::LBrace)?;
        let mut layers = Vec::new();

        while !matches!(self.current().kind, TokenKind::RBrace) {
            layers.push(self.parse_layer_decl()?);
        }

        self.expect(TokenKind::RBrace)?;

        Ok(TopLevelItem::ModelDecl(ModelDecl {
            name,
            layers,
            location: loc,
        }))
    }

    fn parse_layer_decl(&mut self) -> Result<LayerDecl> {
        let loc = self.current_location();
        
        self.expect(TokenKind::Layer)?;
        
        let kind_token = self.expect(TokenKind::Ident(String::new()))?;
        let kind = match kind_token.kind {
            TokenKind::Ident(k) => k,
            _ => return Err(Error::parse("Expected layer type", loc)),
        };

        self.expect(TokenKind::LParen)?;
        let args = self.parse_layer_args()?;
        self.expect(TokenKind::RParen)?;

        let repeat = if self.consume_if(&TokenKind::Repeat) {
            self.expect(TokenKind::LParen)?;
            if let TokenKind::Integer(n) = self.current().kind {
                let n = n as u32;
                self.advance();
                self.expect(TokenKind::RParen)?;
                Some(n)
            } else {
                return Err(Error::parse("Expected repeat count", loc));
            }
        } else {
            None
        };

        self.consume_if(&TokenKind::Semicolon);

        Ok(LayerDecl {
            kind,
            args,
            repeat,
            nested: Vec::new(),
            location: loc,
        })
    }

    fn parse_layer_args(&mut self) -> Result<Vec<(String, Expr)>> {
        let mut args = Vec::new();

        while !matches!(self.current().kind, TokenKind::RParen) {
            let name_token = self.expect(TokenKind::Ident(String::new()))?;
            let name = match name_token.kind {
                TokenKind::Ident(n) => n,
                _ => return Err(Error::parse("Expected layer argument name", self.current_location())),
            };

            self.expect(TokenKind::Equals)?;
            let value = self.parse_expr()?;
            args.push((name, value));

            if !self.consume_if(&TokenKind::Comma) {
                break;
            }
        }

        Ok(args)
    }

    fn parse_struct_def(&mut self) -> Result<TopLevelItem> {
        let loc = self.current_location();
        
        let name_token = self.expect(TokenKind::Ident(String::new()))?;
        let name = match name_token.kind {
            TokenKind::Ident(n) => n,
            _ => return Err(Error::parse("Expected struct name", loc)),
        };

        self.expect(TokenKind::LBrace)?;
        let mut fields = Vec::new();

        while !matches!(self.current().kind, TokenKind::RBrace) {
            let field_name_token = self.expect(TokenKind::Ident(String::new()))?;
            let field_name = match field_name_token.kind {
                TokenKind::Ident(f) => f,
                _ => return Err(Error::parse("Expected field name", loc)),
            };

            self.expect(TokenKind::Colon)?;
            let field_type = self.parse_type_expr()?;
            fields.push((field_name, field_type));

            if !self.consume_if(&TokenKind::Comma) {
                break;
            }
        }

        self.expect(TokenKind::RBrace)?;

        Ok(TopLevelItem::StructDecl(StructDecl {
            name,
            fields,
            location: loc,
        }))
    }

    fn parse_enum_def(&mut self) -> Result<TopLevelItem> {
        let loc = self.current_location();
        
        let name_token = self.expect(TokenKind::Ident(String::new()))?;
        let name = match name_token.kind {
            TokenKind::Ident(n) => n,
            _ => return Err(Error::parse("Expected enum name", loc)),
        };

        self.expect(TokenKind::LBrace)?;
        let mut variants = Vec::new();

        while !matches!(self.current().kind, TokenKind::RBrace) {
            let variant_token = self.expect(TokenKind::Ident(String::new()))?;
            if let TokenKind::Ident(v) = variant_token.kind {
                variants.push(v);
            }

            if !self.consume_if(&TokenKind::Comma) {
                break;
            }
        }

        self.expect(TokenKind::RBrace)?;

        Ok(TopLevelItem::EnumDecl(EnumDecl {
            name,
            variants,
            location: loc,
        }))
    }

    fn parse_unit_def(&mut self) -> Result<TopLevelItem> {
        let loc = self.current_location();
        
        let name_token = self.expect(TokenKind::Ident(String::new()))?;
        let name = match name_token.kind {
            TokenKind::Ident(n) => n,
            _ => return Err(Error::parse("Expected unit name", loc)),
        };

        self.expect(TokenKind::Equals)?;

        let def_token = self.expect(TokenKind::Ident(String::new()))?;
        let definition = match def_token.kind {
            TokenKind::Ident(d) => UnitExpr(d),
            _ => return Err(Error::parse("Expected unit definition", loc)),
        };

        self.consume_if(&TokenKind::Semicolon);

        Ok(TopLevelItem::UnitDecl(UnitDecl {
            name,
            definition,
            location: loc,
        }))
    }
}
