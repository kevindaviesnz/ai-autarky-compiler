// src/ir.rs

use crate::ast::Expr;

#[derive(Debug, Clone)]
pub enum IrNode {
    Int(i64),
    Float(f64),
    Var(String),
    Add(Box<IrNode>, Box<IrNode>),
    Sub(Box<IrNode>, Box<IrNode>),
    Mul(Box<IrNode>, Box<IrNode>), // NEW
    Div(Box<IrNode>, Box<IrNode>), // NEW
    Eq(Box<IrNode>, Box<IrNode>),
    Lam(String, Box<IrNode>),
    App(Box<IrNode>, Box<IrNode>),
    MkPair(Box<IrNode>, Box<IrNode>),
    Left(Box<IrNode>),
    Right(Box<IrNode>),
    Unpack(String, String, Box<IrNode>, Box<IrNode>),
    Match(Box<IrNode>, String, Box<IrNode>, String, Box<IrNode>),
}

pub fn erase_proofs(expr: &Expr) -> IrNode {
    match expr {
        Expr::IntLiteral(n) => IrNode::Int(*n),
        Expr::FloatLiteral(f) => IrNode::Float(*f),
        Expr::Variable(name) => IrNode::Var(name.clone()),
        Expr::Add(l, r) => IrNode::Add(Box::new(erase_proofs(l)), Box::new(erase_proofs(r))),
        Expr::Sub(l, r) => IrNode::Sub(Box::new(erase_proofs(l)), Box::new(erase_proofs(r))),
        Expr::Mul(l, r) => IrNode::Mul(Box::new(erase_proofs(l)), Box::new(erase_proofs(r))), // NEW
        Expr::Div(l, r) => IrNode::Div(Box::new(erase_proofs(l)), Box::new(erase_proofs(r))), // NEW
        Expr::Eq { left, right } => IrNode::Eq(Box::new(erase_proofs(left)), Box::new(erase_proofs(right))),
        Expr::Lambda { param, body, .. } => IrNode::Lam(param.clone(), Box::new(erase_proofs(body))),
        Expr::App { func, arg } => IrNode::App(Box::new(erase_proofs(func)), Box::new(erase_proofs(arg))),
        Expr::MkPair(l, r) => IrNode::MkPair(Box::new(erase_proofs(l)), Box::new(erase_proofs(r))),
        Expr::Left(expr, _) => IrNode::Left(Box::new(erase_proofs(expr))),
        Expr::Right(expr, _) => IrNode::Right(Box::new(erase_proofs(expr))),
        Expr::Unpack { pair, var1, var2, body } => IrNode::Unpack(
            var1.clone(), 
            var2.clone(), 
            Box::new(erase_proofs(pair)), 
            Box::new(erase_proofs(body))
        ),
        Expr::Match { expr, left_var, left_body, right_var, right_body } => IrNode::Match(
            Box::new(erase_proofs(expr)),
            left_var.clone(),
            Box::new(erase_proofs(left_body)),
            right_var.clone(),
            Box::new(erase_proofs(right_body)),
        ),
    }
}