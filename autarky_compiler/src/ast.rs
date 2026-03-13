// src/ast.rs

#[derive(Debug, Clone, PartialEq)]
pub enum Type {
    Int,
    Float,
    Func(Box<Type>, Box<Type>),
    Pair(Box<Type>, Box<Type>),
    Either(Box<Type>, Box<Type>),
}

#[derive(Debug, Clone)]
pub enum Expr {
    IntLiteral(i64),
    FloatLiteral(f64),
    Variable(String),
    Add(Box<Expr>, Box<Expr>),
    Sub(Box<Expr>, Box<Expr>),
    Mul(Box<Expr>, Box<Expr>), // NEW
    Div(Box<Expr>, Box<Expr>), // NEW
    Eq { left: Box<Expr>, right: Box<Expr> },
    Lambda { param: String, param_type: Type, body: Box<Expr> },
    App { func: Box<Expr>, arg: Box<Expr> },
    MkPair(Box<Expr>, Box<Expr>),
    Left(Box<Expr>, Type),
    Right(Box<Expr>, Type),
    Unpack { pair: Box<Expr>, var1: String, var2: String, body: Box<Expr> },
    Match { expr: Box<Expr>, left_var: String, left_body: Box<Expr>, right_var: String, right_body: Box<Expr> },
}