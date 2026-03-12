// src/ir.rs

use crate::ast::Expr;

/// The untyped Intermediate Representation (IR).
/// All memory safety has been proven, so we can drop the type metadata to run at maximum speed.
#[derive(Debug, Clone)]
pub enum IrNode {
    Int(i64),
    Var(String),
    Lam(String, Box<IrNode>),
    App(Box<IrNode>, Box<IrNode>),
    Add(Box<IrNode>, Box<IrNode>),
    Sub(Box<IrNode>, Box<IrNode>),
    MkPair(Box<IrNode>, Box<IrNode>),
    Unpack(String, String, Box<IrNode>, Box<IrNode>),
    Left(Box<IrNode>),
    Right(Box<IrNode>),
    Match(Box<IrNode>, String, Box<IrNode>, String, Box<IrNode>),
    ArrayAlloc(Box<IrNode>, Box<IrNode>),
    ArraySwap(Box<IrNode>, Box<IrNode>, Box<IrNode>),
}

/// Recursively traverses the safe AST and strips all type information.
pub fn erase_proofs(expr: &Expr) -> IrNode {
    match expr {
        Expr::IntLiteral(n) => IrNode::Int(*n),
        Expr::Variable(name) => IrNode::Var(name.clone()),
        Expr::Lambda { param, body, .. } => {
            IrNode::Lam(param.clone(), Box::new(erase_proofs(body)))
        }
        Expr::App { func, arg } => {
            IrNode::App(Box::new(erase_proofs(func)), Box::new(erase_proofs(arg)))
        }
        Expr::Add(l, r) => IrNode::Add(Box::new(erase_proofs(l)), Box::new(erase_proofs(r))),
        Expr::Sub(l, r) => IrNode::Sub(Box::new(erase_proofs(l)), Box::new(erase_proofs(r))),
        Expr::MkPair(l, r) => IrNode::MkPair(Box::new(erase_proofs(l)), Box::new(erase_proofs(r))),
        Expr::Unpack { pair, var1, var2, body } => {
            IrNode::Unpack(var1.clone(), var2.clone(), Box::new(erase_proofs(pair)), Box::new(erase_proofs(body)))
        }
        Expr::Left(e, _) => IrNode::Left(Box::new(erase_proofs(e))),
        Expr::Right(e, _) => IrNode::Right(Box::new(erase_proofs(e))),
        Expr::Match { expr, left_var, left_body, right_var, right_body } => {
            IrNode::Match(
                Box::new(erase_proofs(expr)), 
                left_var.clone(), Box::new(erase_proofs(left_body)), 
                right_var.clone(), Box::new(erase_proofs(right_body))
            )
        }
        Expr::ArrayAlloc { size, init_val } => {
            IrNode::ArrayAlloc(Box::new(erase_proofs(size)), Box::new(erase_proofs(init_val)))
        }
        Expr::ArraySwap { array, index, new_val } => {
            IrNode::ArraySwap(
                Box::new(erase_proofs(array)), 
                Box::new(erase_proofs(index)), 
                Box::new(erase_proofs(new_val))
            )
        }
    }
}