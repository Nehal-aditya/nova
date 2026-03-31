// Complete NOVA Parser Implementation
// This file contains the full recursive descent parser for NOVA

use crate::lexer::{Token, TokenKind};
use crate::{ast::*, Error, Result, SourceLoc};

pub struct Parser {
    tokens: Vec<Token>,
    pos: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Parser { tokens, pos: 0 }
    }

    fn current(&self) -> &Token {
        self.tokens.get(self.pos).unwrap_or_else(|| &self.tokens[self.tokens.len() - 1])
    }

    fn peek(&self, offset: usize) -> &Token {
        self.tokens.get(self.pos + offset).unwrap_or_else(|| &self.tokens[self.tokens.len() - 1])
    }

    fn advance(&mut self) -> Token {
        let token = self.current().clone();
        if self.pos < self.tokens.len() {
            self.pos += 1;
        }
        token
    }

    fn consume_if(&mut self, kind: &TokenKind) -> bool {
        if std::mem::discriminant(kind) == std::mem::discriminant(&self.current().kind) {
            self.advance();
            true
        } else {
            false
        }
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

    fn is_at_end(&self) -> bool {
        matches!(self.current().kind, TokenKind::Eof)
    }

    pub fn parse_program(&mut self) -> Result<Program> {
        let mut items = Vec::new();
        while !self.is_at_end() {
            items.push(self.parse_top_level()?);
        }
        Ok(Program { items })
    }

    fn parse_top_level(&mut self) -> Result<TopLevelItem> {
        match &self.current().kind {
            TokenKind::Test => {
                self.advance();
                self.expect(TokenKind::Mission)?;
                let mission = self.parse_mission_def()?;
                Ok(TopLevelItem::TestMission(mission))
            }
            TokenKind::Mission => {
                self.advance();
                self.parse_mission_def().map(TopLevelItem::MissionDecl)
            }
            TokenKind::Parallel => {
                self.advance();
                self.expect(TokenKind::Mission)?;
                self.parse_mission_def().map(TopLevelItem::ParallelMissionDecl)
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
            _ => Err(Error::parse(format!("Unexpected top-level: {:?}", self.current().kind), self.current().location)),
        }
    }

    fn parse_mission_def(&mut self) -> Result<MissionDecl> {
        let loc = self.current().location;
        let name = self.lex_ident()?;
        self.expect(TokenKind::LParen)?;
        let params = self.parse_parameters()?;
        self.expect(TokenKind::RParen)?;

        let return_type = if self.current().kind == TokenKind::Arrow {
            self.advance();
            Some(self.parse_type()?)
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

        while self.current().kind != TokenKind::RParen {
            let loc = self.current().location;
            let name = self.lex_ident()?;
            self.expect(TokenKind::Colon)?;
            let type_annotation = self.parse_type()?;
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

    fn parse_type(&mut self) -> Result<TypeExpr> {
        let name = self.lex_ident_or_keyword()?;
        match name.as_str() {
            "Float" => {
                if self.current().kind == TokenKind::LBracket {
                    self.advance();
                    let unit = self.parse_unit_expr()?;
                    self.expect(TokenKind::RBracket)?;
                    Ok(TypeExpr::Float(Some(unit)))
                } else {
                    Ok(TypeExpr::Float(None))
                }
            }
            "Int" => Ok(TypeExpr::Int),
            "Bool" => Ok(TypeExpr::Bool),
            "String" => Ok(TypeExpr::String),
            "Void" => Ok(TypeExpr::Named("Void".to_string())),
            "Array" => {
                self.expect(TokenKind::LBracket)?;
                let elem = self.parse_type()?;
                self.expect(TokenKind::RBracket)?;
                Ok(TypeExpr::Array(Box::new(elem)))
            }
            _ => Ok(TypeExpr::Named(name)),
        }
    }

    fn parse_unit_expr(&mut self) -> Result<UnitExpr> {
        let mut unit = self.lex_ident()?;
        
        // Handle unit expressions like "m/s", "kg*m/s^2", etc.
        while matches!(self.current().kind, TokenKind::Slash | TokenKind::Star) {
            if self.current().kind == TokenKind::Slash {
                unit.push('/');
                self.advance();
                unit.push_str(&self.lex_ident()?);
            } else if self.current().kind == TokenKind::Star {
                unit.push('*');
                self.advance();
                unit.push_str(&self.lex_ident()?);
            }
        }
        
        Ok(UnitExpr(unit))
    }

    fn parse_block(&mut self) -> Result<Vec<Statement>> {
        let mut statements = Vec::new();

        while self.current().kind != TokenKind::RBrace && !self.is_at_end() {
            statements.push(self.parse_statement()?);
        }

        Ok(statements)
    }

    fn parse_statement(&mut self) -> Result<Statement> {
        let loc = self.current().location;

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
            TokenKind::Autodiff => {
                self.advance();
                self.expect(TokenKind::LBrace)?;
                let _body = self.parse_block()?;
                self.expect(TokenKind::RBrace)?;
                // For now, treat autodiff as an expr statement
                Ok(Statement::ExprStmt(Expr::Ident("autodiff".to_string())))
            }
            TokenKind::Return => {
                self.advance();
                let expr = if self.current().kind != TokenKind::RBrace && self.current().kind != TokenKind::Semicolon {
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
                
                // Check for assignment
                if self.current().kind == TokenKind::Equals {
                    self.advance();
                    let value = self.parse_expr()?;
                    self.consume_if(&TokenKind::Semicolon);
                    
                    // Convert expr to assignment target
                    if let Expr::Ident(name) = expr {
                        Ok(Statement::LetBind {
                            name,
                            mutable: true,
                            type_annotation: None,
                            value,
                            location: loc,
                        })
                    } else {
                        Err(Error::parse("Invalid assignment target".to_string(), loc))
                    }
                } else {
                    self.consume_if(&TokenKind::Semicolon);
                    Ok(Statement::ExprStmt(expr))
                }
            }
        }
    }

    fn parse_let_binding(&mut self, mutable: bool, loc: SourceLoc) -> Result<Statement> {
        let name = self.lex_ident()?;
        let type_annotation = if self.current().kind == TokenKind::Colon {
            self.advance();
            Some(self.parse_type()?)
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

        let else_body = if self.current().kind == TokenKind::Else {
            self.advance();
            if self.current().kind == TokenKind::If {
                vec![self.parse_if_statement(self.current().location)?]
            } else {
                self.expect(TokenKind::LBrace)?;
                let body = self.parse_block()?;
                self.expect(TokenKind::RBrace)?;
                body
            }
        } else {
            vec![]
        };

        let else_body = if else_body.is_empty() { None } else { Some(else_body) };

        Ok(Statement::If {
            condition,
            then_body,
            else_body,
            location: loc,
        })
    }

    fn parse_for_statement(&mut self, loc: SourceLoc) -> Result<Statement> {
        let var = self.lex_ident()?;
        self.expect(TokenKind::In)?;
        
        // Parse range expression (e.g., 0..10 or start..end)
        let start = self.parse_additive()?;
        self.expect(TokenKind::Dotdot)?;
        let end = self.parse_additive()?;
        
        // Create a range expression
        let iter = Expr::BinaryOp {
            left: Box::new(start),
            op: BinOp::Range,
            right: Box::new(end),
            location: loc,
        };
        
        self.expect(TokenKind::LBrace)?;
        let body = self.parse_block()?;
        self.expect(TokenKind::RBrace)?;

        Ok(Statement::For { var, iter, body, location: loc })
    }

    fn parse_while_statement(&mut self, loc: SourceLoc) -> Result<Statement> {
        let condition = self.parse_expr()?;
        self.expect(TokenKind::LBrace)?;
        let body = self.parse_block()?;
        self.expect(TokenKind::RBrace)?;

        Ok(Statement::While { condition, body, location: loc })
    }

    // Expression parsing with operator precedence
    fn parse_expr(&mut self) -> Result<Expr> {
        self.parse_pipe()
    }

    fn parse_pipe(&mut self) -> Result<Expr> {
        let mut expr = self.parse_or()?;

        while self.consume_if(&TokenKind::Pipe) {
            let loc = self.current().location;
            let right = self.parse_or()?;
            expr = Expr::Pipe {
                left: Box::new(expr),
                right: Box::new(right),
                location: loc,
            };
        }

        Ok(expr)
    }

    fn parse_or(&mut self) -> Result<Expr> {
        let mut expr = self.parse_and()?;

        while self.consume_if(&TokenKind::Or) {
            let loc = self.current().location;
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
            let loc = self.current().location;
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
            let loc = self.current().location;
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
            let loc = self.current().location;
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
        let mut expr = self.parse_power()?;

        loop {
            let loc = self.current().location;
            let op = match &self.current().kind {
                TokenKind::Star => Some(BinOp::Multiply),
                TokenKind::Slash => Some(BinOp::Divide),
                TokenKind::Percent => Some(BinOp::Modulo),
                TokenKind::At => Some(BinOp::MatMul),
                _ => None,
            };

            if let Some(op) = op {
                self.advance();
                let right = self.parse_power()?;
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

    fn parse_power(&mut self) -> Result<Expr> {
        let mut expr = self.parse_unary()?;

        while self.consume_if(&TokenKind::Caret) {
            let loc = self.current().location;
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
        let loc = self.current().location;

        match &self.current().kind {
            TokenKind::Minus => {
                self.advance();
                let expr = self.parse_unary()?;
                Ok(Expr::UnaryOp {
                    op: UnOp::Negate,
                    operand: Box::new(expr),
                    location: loc,
                })
            }
            TokenKind::Not => {
                self.advance();
                let expr = self.parse_unary()?;
                Ok(Expr::UnaryOp {
                    op: UnOp::Not,
                    operand: Box::new(expr),
                    location: loc,
                })
            }
            _ => self.parse_postfix(),
        }
    }

    fn parse_postfix(&mut self) -> Result<Expr> {
        let mut expr = self.parse_primary()?;

        loop {
            let loc = self.current().location;
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
                    let field = self.lex_ident()?;
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

        while self.current().kind != TokenKind::RParen {
            let expr = self.parse_expr()?;
            args.push((None, expr));

            if !self.consume_if(&TokenKind::Comma) {
                break;
            }
        }

        Ok(args)
    }

    fn parse_primary(&mut self) -> Result<Expr> {
        let loc = self.current().location;

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
            TokenKind::Transmit => {
                self.advance();
                self.expect(TokenKind::LParen)?;
                let mut args = Vec::new();
                while self.current().kind != TokenKind::RParen {
                    args.push(self.parse_expr()?);
                    if !self.consume_if(&TokenKind::Comma) {
                        break;
                    }
                }
                self.expect(TokenKind::RParen)?;
                Ok(Expr::Transmit { args, location: loc })
            }
            _ => Err(Error::parse(
                format!("Unexpected token in expression: {:?}", self.current().kind),
                loc,
            )),
        }
    }

    fn parse_constellation_def(&mut self) -> Result<TopLevelItem> {
        let loc = self.current().location;
        let name = self.lex_ident()?;
        self.expect(TokenKind::LBrace)?;

        let mut items = Vec::new();
        while self.current().kind != TokenKind::RBrace {
            match &self.current().kind {
                TokenKind::Export => {
                    self.advance();
                    let export_name = self.lex_ident()?;
                    items.push(ConstellationItem::Export(export_name));
                    self.consume_if(&TokenKind::Semicolon);
                }
                TokenKind::Mission => {
                    self.advance();
                    let mission = self.parse_mission_def()?;
                    items.push(ConstellationItem::MissionDecl(mission));
                }
                _ => {
                    return Err(Error::parse("Expected mission or export", loc));
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
        let loc = self.current().location;
        let name = self.lex_ident()?;
        self.expect(TokenKind::LBrace)?;

        let mut layers = Vec::new();
        while self.current().kind != TokenKind::RBrace && !self.is_at_end() {
            self.expect(TokenKind::Layer)?;
            let kind = self.lex_ident()?;
            self.expect(TokenKind::LParen)?;

            let mut args = Vec::new();
            while self.current().kind != TokenKind::RParen && !self.is_at_end() {
                // Try to parse as either named (key=value) or positional arguments
                let expr = self.parse_expr()?;
                
                // Check if this is a named argument (ident = expr pattern already parsed)
                // For now, just accept positional args
                args.push(("arg".to_string(), expr));

                if !self.consume_if(&TokenKind::Comma) {
                    break;
                }
            }

            self.expect(TokenKind::RParen)?;
            self.consume_if(&TokenKind::Semicolon);

            layers.push(LayerDecl {
                kind,
                args,
                repeat: None,
                nested: Vec::new(),
                location: loc,
            });
        }

        self.expect(TokenKind::RBrace)?;
        Ok(TopLevelItem::ModelDecl(ModelDecl {
            name,
            layers,
            location: loc,
        }))
    }

    fn parse_struct_def(&mut self) -> Result<TopLevelItem> {
        let loc = self.current().location;
        let name = self.lex_ident()?;
        self.expect(TokenKind::LBrace)?;

        let mut fields = Vec::new();
        while self.current().kind != TokenKind::RBrace && !self.is_at_end() {
            let field_name = self.lex_ident()?;
            self.expect(TokenKind::Colon)?;
            let field_type = self.parse_type()?;
            fields.push((field_name, field_type));

            // Allow both comma-separated and newline-separated fields
            if self.current().kind == TokenKind::RBrace {
                break;
            }
            self.consume_if(&TokenKind::Comma);
        }

        self.expect(TokenKind::RBrace)?;
        Ok(TopLevelItem::StructDecl(StructDecl {
            name,
            fields,
            location: loc,
        }))
    }

    fn parse_enum_def(&mut self) -> Result<TopLevelItem> {
        let loc = self.current().location;
        let name = self.lex_ident()?;
        self.expect(TokenKind::LBrace)?;

        let mut variants = Vec::new();
        while self.current().kind != TokenKind::RBrace && !self.is_at_end() {
            variants.push(self.lex_ident()?);
            if self.current().kind == TokenKind::RBrace {
                break;
            }
            self.consume_if(&TokenKind::Comma);
        }

        self.expect(TokenKind::RBrace)?;
        Ok(TopLevelItem::EnumDecl(EnumDecl {
            name,
            variants,
            location: loc,
        }))
    }

    fn parse_unit_def(&mut self) -> Result<TopLevelItem> {
        let loc = self.current().location;
        let name = self.lex_ident()?;
        self.expect(TokenKind::Equals)?;
        let definition = UnitExpr(self.lex_ident()?);
        self.consume_if(&TokenKind::Semicolon);

        Ok(TopLevelItem::UnitDecl(UnitDecl {
            name,
            definition,
            location: loc,
        }))
    }

    // Helper methods
    fn lex_ident(&mut self) -> Result<String> {
        if let TokenKind::Ident(name) = &self.current().kind {
            let name = name.clone();
            self.advance();
            Ok(name)
        } else {
            Err(Error::parse(
                format!("Expected identifier, got {:?}", self.current().kind),
                self.current().location,
            ))
        }
    }

    fn lex_ident_or_keyword(&mut self) -> Result<String> {
        match &self.current().kind {
            TokenKind::Ident(name) => {
                let name = name.clone();
                self.advance();
                Ok(name)
            }
            _ => Err(Error::parse(
                format!("Expected type, got {:?}", self.current().kind),
                self.current().location,
            )),
        }
    }
}
