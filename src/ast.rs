//! Abstract syntax tree for Fenneth.

use crate::diagnostic::Pos;

/// Fenneth's static types. `Void` is the type of functions with no `->`
/// (and the placeholder type of an expression before type checking).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Type {
    I32,
    F64,
    Bool,
    Str,
    Void,
    /// Dynamic array `[T]` (reference semantics).
    Array(Box<Type>),
    /// User-defined struct (value semantics).
    Struct(String),
}

impl Type {
    pub fn name(&self) -> String {
        match self {
            Type::I32 => "i32".into(),
            Type::F64 => "f64".into(),
            Type::Bool => "bool".into(),
            Type::Str => "String".into(),
            Type::Void => "void".into(),
            Type::Array(t) => format!("[{}]", t.name()),
            Type::Struct(n) => n.clone(),
        }
    }

    /// Looks up a built-in scalar type name.
    pub fn from_name(name: &str) -> Option<Type> {
        match name {
            "i32" => Some(Type::I32),
            "f64" => Some(Type::F64),
            "bool" => Some(Type::Bool),
            "String" => Some(Type::Str),
            _ => None,
        }
    }

    pub fn is_numeric(&self) -> bool {
        matches!(self, Type::I32 | Type::F64)
    }

    pub fn is_printable(&self) -> bool {
        matches!(self, Type::I32 | Type::F64 | Type::Bool | Type::Str)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnaryOp {
    Neg,
    Not,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BinOp {
    Add,
    Sub,
    Mul,
    Div,
    Rem,
    Eq,
    Ne,
    Lt,
    Le,
    Gt,
    Ge,
    And,
    Or,
}

impl BinOp {
    pub fn symbol(self) -> &'static str {
        match self {
            BinOp::Add => "+",
            BinOp::Sub => "-",
            BinOp::Mul => "*",
            BinOp::Div => "/",
            BinOp::Rem => "%",
            BinOp::Eq => "==",
            BinOp::Ne => "!=",
            BinOp::Lt => "<",
            BinOp::Le => "<=",
            BinOp::Gt => ">",
            BinOp::Ge => ">=",
            BinOp::And => "&&",
            BinOp::Or => "||",
        }
    }
}

#[derive(Debug, Clone)]
pub enum ExprKind {
    Int(i64),
    Float(f64),
    Bool(bool),
    Str(String),
    Ident(String),
    Unary(UnaryOp, Box<Expr>),
    Binary(BinOp, Box<Expr>, Box<Expr>),
    Call(String, Vec<Expr>),
    /// `base[index]`
    Index(Box<Expr>, Box<Expr>),
    /// `base.field`
    Field(Box<Expr>, String),
    /// `[a, b, c]`
    ArrayLit(Vec<Expr>),
    /// `Name { field: value, ... }`
    StructLit(String, Vec<(String, Expr)>),
}

/// An expression. `ty` is filled in by the type checker (it is `Void`
/// until then) and read by the code generator.
#[derive(Debug, Clone)]
pub struct Expr {
    pub kind: ExprKind,
    pub pos: Pos,
    pub ty: Type,
}

impl Expr {
    pub fn new(kind: ExprKind, pos: Pos) -> Self {
        Expr {
            kind,
            pos,
            ty: Type::Void,
        }
    }
}

pub type Block = Vec<Stmt>;

#[derive(Debug, Clone)]
pub struct Stmt {
    pub kind: StmtKind,
    pub pos: Pos,
}

#[derive(Debug, Clone)]
pub enum StmtKind {
    Let {
        name: String,
        ty: Type,
        init: Expr,
        is_const: bool,
    },
    /// `target = value` (op is None) or `target op= value`.
    Assign {
        target: Expr,
        op: Option<BinOp>,
        value: Expr,
    },
    Expr(Expr),
    Return(Option<Expr>),
    If {
        cond: Expr,
        then_block: Block,
        else_block: Option<Block>,
    },
    While {
        cond: Expr,
        body: Block,
    },
    /// `for i in start..end { }` (end is exclusive)
    ForRange {
        var: String,
        start: Expr,
        end: Expr,
        body: Block,
    },
    /// `for x in array { }`
    ForEach {
        var: String,
        iter: Expr,
        body: Block,
    },
    Break,
    Continue,
    Block(Block),
}

#[derive(Debug, Clone)]
pub struct Param {
    pub name: String,
    pub ty: Type,
    pub pos: Pos,
}

#[derive(Debug, Clone)]
pub struct Function {
    pub name: String,
    pub params: Vec<Param>,
    pub ret: Type,
    pub body: Block,
    pub pos: Pos,
    /// `extern fn` declarations have no body and link to C/Rust symbols.
    pub is_extern: bool,
    /// True for functions that come from a standard library module
    /// (`import "std/..."`). Unused ones are dropped before code generation.
    pub from_std: bool,
}

#[derive(Debug, Clone)]
pub struct Field {
    pub name: String,
    pub ty: Type,
    pub pos: Pos,
}

#[derive(Debug, Clone)]
pub struct StructDef {
    pub name: String,
    pub fields: Vec<Field>,
    pub pos: Pos,
}

/// A top-level `let`/`const` with a literal initializer.
#[derive(Debug, Clone)]
pub struct Global {
    pub name: String,
    pub ty: Type,
    pub init: Expr,
    pub is_const: bool,
    pub pos: Pos,
}

#[derive(Debug, Clone)]
pub struct Import {
    pub path: String,
    pub pos: Pos,
}

#[derive(Debug, Clone, Default)]
pub struct Program {
    pub imports: Vec<Import>,
    pub structs: Vec<StructDef>,
    pub globals: Vec<Global>,
    pub functions: Vec<Function>,
}
