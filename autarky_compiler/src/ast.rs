// src/ast.rs

#[derive(Debug, Clone, PartialEq)]
pub enum Type {
    Int,
    Pair(Box<Type>, Box<Type>),
    Either(Box<Type>, Box<Type>),
    Func(Box<Type>, Box<Type>),
    
    // NEW: The Contiguous Array Type. 
    // It wraps another Type, e.g., Array(Int) or Array(Pair Int Int)
    Array(Box<Type>),
}

#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    // 1. Core Lambda Calculus
    IntLiteral(i64),
    Variable(String),
    Lambda { 
        param: String, 
        param_type: Type, 
        body: Box<Expr> 
    },
    App { 
        func: Box<Expr>, 
        arg: Box<Expr> 
    },
    
    // 2. Math
    Add(Box<Expr>, Box<Expr>),
    Sub(Box<Expr>, Box<Expr>),
    
    // 3. Pairs (Tuples)
    MkPair(Box<Expr>, Box<Expr>),
    Unpack { 
        pair: Box<Expr>, 
        var1: String, 
        var2: String, 
        body: Box<Expr> 
    },
    
    // 4. Sum Types (Branching)
    Left(Box<Expr>, Type),
    Right(Box<Expr>, Type),
    Match { 
        expr: Box<Expr>, 
        left_var: String, 
        left_body: Box<Expr>, 
        right_var: String, 
        right_body: Box<Expr> 
    },

    // ==========================================
    // NATIVE SYSTEM PRIMITIVES
    // ==========================================

    // 5. Array Allocation
    // size: Must evaluate to an Int.
    // init_val: Must evaluate to an Int (to prevent duplicating linear assets).
    ArrayAlloc { 
        size: Box<Expr>, 
        init_val: Box<Expr> 
    },

    // 6. The Linear Array Swap
    // array: The contiguous block we are operating on.
    // index: The exact slot to target.
    // new_val: The linear asset we are inserting into the array.
    // RETURNS: Pair(OldVal, NewArray)
    ArraySwap { 
        array: Box<Expr>, 
        index: Box<Expr>, 
        new_val: Box<Expr> 
    },
}