#[derive(Debug, Clone, PartialEq)]
pub enum Permission { Full, Fraction(u32, u32) }

#[derive(Debug, Clone, PartialEq)]
pub enum Type {
    Universe(u32), Int, Unit, Bool, String,
    Pair(Box<Type>, Box<Type>), Either(Box<Type>, Box<Type>), Array(Box<Type>),
    Pi(String, Box<Type>, Box<Type>), Persistent(Box<Type>), Linear(Permission, Box<Type>),
    Rec(String, Box<Type>), TVar(String), // NEW: Recursive Types and Type Variables
}

#[derive(Debug, Clone, PartialEq)]
pub enum Term {
    Var(String), IntVal(u32), UnitVal, BoolVal(bool), StringVal(String),
    MkPair(Box<Term>, Box<Term>), Unpack(Box<Term>, String, String, Box<Term>),
    Left(Box<Term>, Type), Right(Type, Box<Term>), Match(Box<Term>, String, Box<Term>, String, Box<Term>), 
    Alloc(Box<Term>, Box<Term>), Read(Box<Term>, Box<Term>), Write(Box<Term>, Box<Term>, Box<Term>), ReadFile(Box<Term>),
    If(Box<Term>, Box<Term>, Box<Term>), Add(Box<Term>, Box<Term>), Sub(Box<Term>, Box<Term>), Eq(Box<Term>, Box<Term>),  
    Fix(Box<Term>), Abs(String, Type, Box<Term>), App(Box<Term>, Box<Term>), Free(Box<Term>),
    Split(String, String, String, Box<Term>), Merge(String, String, String, Box<Term>),
    Fold(Type, Box<Term>), Unfold(Box<Term>), // NEW: Type-level rolling and unrolling
}

impl Type {
    pub fn substitute(&self, var_name: &str, term: &Term) -> Type {
        match self {
            Type::Universe(n) => Type::Universe(*n), Type::Int => Type::Int, Type::Unit => Type::Unit,
            Type::Bool => Type::Bool, Type::String => Type::String,
            Type::Pair(t1, t2) => Type::Pair(Box::new(t1.substitute(var_name, term)), Box::new(t2.substitute(var_name, term))),
            Type::Either(t1, t2) => Type::Either(Box::new(t1.substitute(var_name, term)), Box::new(t2.substitute(var_name, term))),
            Type::Array(t) => Type::Array(Box::new(t.substitute(var_name, term))),
            Type::Pi(p, t1, t2) => Type::Pi(p.clone(), Box::new(t1.substitute(var_name, term)), if p == var_name { t2.clone() } else { Box::new(t2.substitute(var_name, term)) }),
            Type::Persistent(t) => Type::Persistent(Box::new(t.substitute(var_name, term))),
            Type::Linear(p, t) => Type::Linear(p.clone(), Box::new(t.substitute(var_name, term))),
            Type::Rec(p, t) => Type::Rec(p.clone(), Box::new(t.substitute(var_name, term))),
            Type::TVar(n) => Type::TVar(n.clone()),
        }
    }

    // NEW: Type-in-Type substitution (used for Unfolding Rec Types)
    pub fn substitute_type(&self, var_name: &str, replacement: &Type) -> Type {
        match self {
            Type::Universe(n) => Type::Universe(*n), Type::Int => Type::Int, Type::Unit => Type::Unit,
            Type::Bool => Type::Bool, Type::String => Type::String,
            Type::Pair(t1, t2) => Type::Pair(Box::new(t1.substitute_type(var_name, replacement)), Box::new(t2.substitute_type(var_name, replacement))),
            Type::Either(t1, t2) => Type::Either(Box::new(t1.substitute_type(var_name, replacement)), Box::new(t2.substitute_type(var_name, replacement))),
            Type::Array(t) => Type::Array(Box::new(t.substitute_type(var_name, replacement))),
            Type::Pi(p, t1, t2) => Type::Pi(p.clone(), Box::new(t1.substitute_type(var_name, replacement)), Box::new(t2.substitute_type(var_name, replacement))),
            Type::Persistent(t) => Type::Persistent(Box::new(t.substitute_type(var_name, replacement))),
            Type::Linear(p, t) => Type::Linear(p.clone(), Box::new(t.substitute_type(var_name, replacement))),
            Type::Rec(p, t) => { if p == var_name { Type::Rec(p.clone(), t.clone()) } else { Type::Rec(p.clone(), Box::new(t.substitute_type(var_name, replacement))) } }
            Type::TVar(n) => { if n == var_name { replacement.clone() } else { Type::TVar(n.clone()) } },
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Resource { Persistent(Type), Linear(Type) }