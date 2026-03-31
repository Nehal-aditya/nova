// src/ast.rs
// Abstract Syntax Tree for NOVA

use crate::SourceLoc;

/// A complete NOVA program
#[derive(Debug, Clone)]
pub struct Program {
    pub items: Vec<TopLevelItem>,
}

/// Top-level declarations
#[derive(Debug, Clone)]
pub enum TopLevelItem {
    MissionDecl(MissionDecl),
    ParallelMissionDecl(MissionDecl),
    ConstellationDecl(ConstellationDecl),
    ModelDecl(ModelDecl),
    StructDecl(StructDecl),
    EnumDecl(EnumDecl),
    UnitDecl(UnitDecl),
    TestMission(MissionDecl),
}

#[derive(Debug, Clone)]
pub struct MissionDecl {
    pub name: String,
    pub params: Vec<Parameter>,
    pub return_type: Option<TypeExpr>,
    pub body: Vec<Statement>,
    pub location: SourceLoc,
}

#[derive(Debug, Clone)]
pub struct Parameter {
    pub name: String,
    pub type_annotation: TypeExpr,
    pub location: SourceLoc,
}

#[derive(Debug, Clone)]
pub struct ConstellationDecl {
    pub name: String,
    pub items: Vec<ConstellationItem>,
    pub location: SourceLoc,
}

#[derive(Debug, Clone)]
pub enum ConstellationItem {
    MissionDecl(MissionDecl),
    Export(String),
}

#[derive(Debug, Clone)]
pub struct ModelDecl {
    pub name: String,
    pub layers: Vec<LayerDecl>,
    pub location: SourceLoc,
}

#[derive(Debug, Clone)]
pub struct LayerDecl {
    pub kind: String,
    pub args: Vec<(String, Expr)>,
    pub repeat: Option<u32>,
    pub nested: Vec<LayerDecl>,
    pub location: SourceLoc,
}

#[derive(Debug, Clone)]
pub struct StructDecl {
    pub name: String,
    pub fields: Vec<(String, TypeExpr)>,
    pub location: SourceLoc,
}

#[derive(Debug, Clone)]
pub struct EnumDecl {
    pub name: String,
    pub variants: Vec<String>,
    pub location: SourceLoc,
}

#[derive(Debug, Clone)]
pub struct UnitDecl {
    pub name: String,
    pub definition: UnitExpr,
    pub location: SourceLoc,
}

/// Statements in a mission body
#[derive(Debug, Clone)]
pub enum Statement {
    LetBind {
        name: String,
        mutable: bool,
        type_annotation: Option<TypeExpr>,
        value: Expr,
        location: SourceLoc,
    },
    ExprStmt(Expr),
    Return(Option<Expr>),
    If {
        condition: Expr,
        then_body: Vec<Statement>,
        else_body: Option<Vec<Statement>>,
        location: SourceLoc,
    },
    For {
        var: String,
        iter: Expr,
        body: Vec<Statement>,
        location: SourceLoc,
    },
    While {
        condition: Expr,
        body: Vec<Statement>,
        location: SourceLoc,
    },
    Break,
}

/// Type expressions
#[derive(Debug, Clone, PartialEq)]
pub enum TypeExpr {
    Named(String),
    Float(Option<UnitExpr>),
    Int,
    Bool,
    String,
    Tensor {
        element_type: Box<TypeExpr>,
        shape: Vec<usize>,
    },
    Array(Box<TypeExpr>),
    Option(Box<TypeExpr>),
    Result(Box<TypeExpr>, Box<TypeExpr>),
    Wave(Box<TypeExpr>),
    Function {
        params: Vec<TypeExpr>,
        return_type: Box<TypeExpr>,
    },
}

impl std::fmt::Display for TypeExpr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TypeExpr::Named(n) => write!(f, "{}", n),
            TypeExpr::Float(None) => write!(f, "Float"),
            TypeExpr::Float(Some(unit)) => write!(f, "Float[{}]", unit),
            TypeExpr::Int => write!(f, "Int"),
            TypeExpr::Bool => write!(f, "Bool"),
            TypeExpr::String => write!(f, "String"),
            TypeExpr::Tensor { element_type, shape } => {
                write!(f, "Tensor[{}", element_type)?;
                for dim in shape {
                    write!(f, ",{}", dim)?;
                }
                write!(f, "]")
            }
            TypeExpr::Array(inner) => write!(f, "Array[{}]", inner),
            TypeExpr::Option(inner) => write!(f, "Option[{}]", inner),
            TypeExpr::Result(ok, err) => write!(f, "Result[{},{}]", ok, err),
            TypeExpr::Wave(inner) => write!(f, "Wave[{}]", inner),
            TypeExpr::Function { params, return_type } => {
                write!(f, "(")?;
                for (i, p) in params.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", p)?;
                }
                write!(f, ") → {}", return_type)
            }
        }
    }
}

/// Unit expressions: m/s², kg·m/s², etc.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UnitExpr(pub String);

impl std::fmt::Display for UnitExpr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[{}]", self.0)
    }
}

/// Expressions
#[derive(Debug, Clone)]
pub enum Expr {
    Integer(i64),
    Float(f64),
    UnitAnnotatedFloat {
        value: f64,
        unit: UnitExpr,
    },
    String(String),
    Bool(bool),
    Ident(String),
    
    BinaryOp {
        left: Box<Expr>,
        op: BinOp,
        right: Box<Expr>,
        location: SourceLoc,
    },
    UnaryOp {
        op: UnOp,
        operand: Box<Expr>,
        location: SourceLoc,
    },
    Call {
        func: Box<Expr>,
        args: Vec<(Option<String>, Expr)>, // (name, value) for named args
        location: SourceLoc,
    },
    FieldAccess {
        object: Box<Expr>,
        field: String,
        location: SourceLoc,
    },
    Index {
        object: Box<Expr>,
        index: Box<Expr>,
        location: SourceLoc,
    },
    Pipe {
        left: Box<Expr>,
        right: Box<Expr>,
        location: SourceLoc,
    },
    Pipeline {
        source: Box<Expr>,
        stages: Vec<Expr>,
        location: SourceLoc,
    },
    Lambda {
        param: String,
        body: Box<Expr>,
        location: SourceLoc,
    },
    Match {
        subject: Box<Expr>,
        arms: Vec<MatchArm>,
        location: SourceLoc,
    },
    If {
        condition: Box<Expr>,
        then_expr: Box<Expr>,
        else_expr: Option<Box<Expr>>,
        location: SourceLoc,
    },
    Transmit {
        args: Vec<Expr>,
        location: SourceLoc,
    },
    Autodiff {
        target: Box<Expr>,
        body: Vec<Statement>,
        location: SourceLoc,
    },
    Gradient {
        expr: Box<Expr>,
        wrt: Vec<String>,
        location: SourceLoc,
    },
    TensorLiteral {
        elements: Vec<Expr>,
        location: SourceLoc,
    },
    StructLiteral {
        type_name: Option<String>,
        fields: Vec<(String, Expr)>,
        location: SourceLoc,
    },
    Range {
        start: Box<Expr>,
        end: Box<Expr>,
        inclusive: bool,
        location: SourceLoc,
    },
    StringInterpolation {
        parts: Vec<InterpolationPart>,
        location: SourceLoc,
    },
}

#[derive(Debug, Clone)]
pub enum InterpolationPart {
    Literal(String),
    Expr { expr: Expr, format: Option<String> },
}

#[derive(Debug, Clone)]
pub struct MatchArm {
    pub pattern: Pattern,
    pub guard: Option<Expr>,
    pub body: Expr,
}

#[derive(Debug, Clone)]
pub enum Pattern {
    Ident(String),
    Literal(Literal),
    Some(Box<Pattern>),
    None,
    Constructor(String, Vec<Pattern>),
}

#[derive(Debug, Clone)]
pub enum Literal {
    Int(i64),
    Float(f64),
    Bool(bool),
    String(String),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BinOp {
    Add,
    Subtract,
    Multiply,
    Divide,
    Modulo,
    Power,
    Equal,
    NotEqual,
    Less,
    Greater,
    LessEqual,
    GreaterEqual,
    And,
    Or,
    MatMul, // @
    Range,  // .. for range expressions
}

impl std::fmt::Display for BinOp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BinOp::Add => write!(f, "+"),
            BinOp::Subtract => write!(f, "-"),
            BinOp::Multiply => write!(f, "*"),
            BinOp::Divide => write!(f, "/"),
            BinOp::Modulo => write!(f, "%"),
            BinOp::Power => write!(f, "^"),
            BinOp::Equal => write!(f, "=="),
            BinOp::NotEqual => write!(f, "!="),
            BinOp::Less => write!(f, "<"),
            BinOp::Greater => write!(f, ">"),
            BinOp::LessEqual => write!(f, "<="),
            BinOp::GreaterEqual => write!(f, ">="),
            BinOp::And => write!(f, "&&"),
            BinOp::Or => write!(f, "||"),
            BinOp::MatMul => write!(f, "@"),
            BinOp::Range => write!(f, ".."),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnOp {
    Negate,
    Not,
    Deref,
}

impl std::fmt::Display for UnOp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            UnOp::Negate => write!(f, "-"),
            UnOp::Not => write!(f, "!"),
            UnOp::Deref => write!(f, "*"),
        }
    }
}
