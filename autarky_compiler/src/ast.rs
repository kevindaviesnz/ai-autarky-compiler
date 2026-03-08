#[derive(Debug, Clone, PartialEq)]
pub enum Permission {
    Full,
    Fraction(u32, u32), 
}

#[derive(Debug, Clone, PartialEq)]
pub enum Type {
    Universe(u32),
    Pi(String, Box<Type>, Box<Type>),
    Persistent(Box<Type>),
    Linear(Permission, Box<Type>),
}

#[derive(Debug, Clone, PartialEq)]
pub enum Term {
    Var(String),
    Abs(String, Type, Box<Term>),
    App(Box<Term>, Box<Term>),
    Split(String, String, String, Box<Term>),
    /// merge alias1 and alias2 into target in body
    Merge(String, String, String, Box<Term>),
}

impl Type {
    pub fn substitute(&self, var_name: &str, term: &Term) -> Type {
        match self {
            Type::Universe(n) => Type::Universe(*n),
            Type::Pi(param, t1, t2) => {
                let sub_t1 = t1.substitute(var_name, term);
                if param == var_name {
                    Type::Pi(param.clone(), Box::new(sub_t1), t2.clone())
                } else {
                    Type::Pi(param.clone(), Box::new(sub_t1), Box::new(t2.substitute(var_name, term)))
                }
            },
            Type::Persistent(t) => Type::Persistent(Box::new(t.substitute(var_name, term))),
            Type::Linear(perm, t) => Type::Linear(perm.clone(), Box::new(t.substitute(var_name, term))),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Resource {
    Persistent(Type),
    Linear(Type),
}