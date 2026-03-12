// src/ast.rs

#[derive(Debug, Clone, PartialEq)]
pub enum Type {
    Int,
    Pair(Box<Type>, Box<Type>),
    Either(Box<Type>, Box<Type>),
    Func(Box<Type>, Box<Type>),
}

#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    IntLiteral(i64),
    Variable(String),
    Lambda { param: String, param_type: Type, body: Box<Expr> },
    App { func: Box<Expr>, arg: Box<Expr> },
    Add(Box<Expr>, Box<Expr>),
    Sub(Box<Expr>, Box<Expr>),
    Eq { left: Box<Expr>, right: Box<Expr> },
    MkPair(Box<Expr>, Box<Expr>),
    Unpack { pair: Box<Expr>, var1: String, var2: String, body: Box<Expr> },
    Left(Box<Expr>, Type),
    Right(Box<Expr>, Type),
    Match { 
        expr: Box<Expr>, 
        left_var: String, left_body: Box<Expr>, 
        right_var: String, right_body: Box<Expr> 
    },
    ArrayAlloc { size: Box<Expr>, init_val: Box<Expr> },
    ArraySwap { array: Box<Expr>, index: Box<Expr>, new_val: Box<Expr> },
}