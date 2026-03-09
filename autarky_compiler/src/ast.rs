#[derive(Debug, Clone, PartialEq)]
pub enum Permission { Full, Fraction(u32, u32) }

#[derive(Debug, Clone, PartialEq)]
pub enum Type {
    Universe(u32), Int, Unit, Bool, String, // NEW
    Pair(Box<Type>, Box<Type>), Either(Box<Type>, Box<Type>), Array(Box<Type>),
    Pi(String, Box<Type>, Box<Type>), Persistent(Box<Type>), Linear(Permission, Box<Type>),
}

#[derive(Debug, Clone, PartialEq)]
pub enum Term {
    Var(String), IntVal(u32), UnitVal, BoolVal(bool), StringVal(String), // NEW
    MkPair(Box<Term>, Box<Term>), Unpack(Box<Term>, String, String, Box<Term>),
    Left(Box<Term>, Type), Right(Type, Box<Term>), Match(Box<Term>, String, Box<Term>, String, Box<Term>), 
    Alloc(Box<Term>, Box<Term>), Read(Box<Term>, Box<Term>), Write(Box<Term>, Box<Term>, Box<Term>), 
    ReadFile(Box<Term>), // NEW: read_file "path"
    If(Box<Term>, Box<Term>, Box<Term>), Add(Box<Term>, Box<Term>), Sub(Box<Term>, Box<Term>), Eq(Box<Term>, Box<Term>),  
    Fix(Box<Term>), Abs(String, Type, Box<Term>), App(Box<Term>, Box<Term>), Free(Box<Term>),
    Split(String, String, String, Box<Term>), Merge(String, String, String, Box<Term>),
}

impl Type {
    pub fn substitute(&self, var_name: &str, term: &Term) -> Type {
        match self {
            Type::Universe(n) => Type::Universe(*n), Type::Int => Type::Int, Type::Unit => Type::Unit,
            Type::Bool => Type::Bool, Type::String => Type::String, // NEW
            Type::Pair(t1, t2) => Type::Pair(Box::new(t1.substitute(var_name, term)), Box::new(t2.substitute(var_name, term))),
            Type::Either(t1, t2) => Type::Either(Box::new(t1.substitute(var_name, term)), Box::new(t2.substitute(var_name, term))),
            Type::Array(t) => Type::Array(Box::new(t.substitute(var_name, term))),
            Type::Pi(p, t1, t2) => Type::Pi(p.clone(), Box::new(t1.substitute(var_name, term)), if p == var_name { t2.clone() } else { Box::new(t2.substitute(var_name, term)) }),
            Type::Persistent(t) => Type::Persistent(Box::new(t.substitute(var_name, term))),
            Type::Linear(p, t) => Type::Linear(p.clone(), Box::new(t.substitute(var_name, term))),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Resource { Persistent(Type), Linear(Type) }