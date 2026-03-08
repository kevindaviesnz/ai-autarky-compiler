use crate::ast::Term;

#[derive(Debug, Clone, PartialEq)]
pub enum IRTerm {
    Var(String),
    IntVal(u32), 
    UnitVal, 
    BoolVal(bool), 
    MkPair(Box<IRTerm>, Box<IRTerm>), 
    Unpack(Box<IRTerm>, String, String, Box<IRTerm>), 
    If(Box<IRTerm>, Box<IRTerm>, Box<IRTerm>), 
    Add(Box<IRTerm>, Box<IRTerm>), 
    Sub(Box<IRTerm>, Box<IRTerm>), // NEW
    Eq(Box<IRTerm>, Box<IRTerm>),  // NEW
    Fix(Box<IRTerm>),              // NEW
    Abs(String, Box<IRTerm>),
    App(Box<IRTerm>, Box<IRTerm>),
    Free(Box<IRTerm>), 
    #[allow(dead_code)]
    Erased,
}

pub fn generate_ir(ast: &Term) -> IRTerm {
    match ast {
        Term::IntVal(n) => IRTerm::IntVal(*n),
        Term::UnitVal => IRTerm::UnitVal, 
        Term::BoolVal(b) => IRTerm::BoolVal(*b), 
        Term::MkPair(t1, t2) => IRTerm::MkPair(Box::new(generate_ir(t1)), Box::new(generate_ir(t2))), 
        Term::Unpack(target, alias1, alias2, body) => { 
            IRTerm::Unpack(Box::new(generate_ir(target)), alias1.clone(), alias2.clone(), Box::new(generate_ir(body)))
        }
        Term::If(c, t, f) => IRTerm::If(Box::new(generate_ir(c)), Box::new(generate_ir(t)), Box::new(generate_ir(f))), 
        Term::Add(t1, t2) => IRTerm::Add(Box::new(generate_ir(t1)), Box::new(generate_ir(t2))),
        Term::Sub(t1, t2) => IRTerm::Sub(Box::new(generate_ir(t1)), Box::new(generate_ir(t2))), // NEW
        Term::Eq(t1, t2) => IRTerm::Eq(Box::new(generate_ir(t1)), Box::new(generate_ir(t2))),    // NEW
        Term::Fix(inner) => IRTerm::Fix(Box::new(generate_ir(inner))),                           // NEW
        Term::Var(name) => IRTerm::Var(name.clone()),
        Term::Free(target) => IRTerm::Free(Box::new(generate_ir(target))), 
        Term::Abs(param, _type_annotation, body) => {
            IRTerm::Abs(param.clone(), Box::new(generate_ir(body)))
        }
        Term::App(t1, t2) => {
            IRTerm::App(Box::new(generate_ir(t1)), Box::new(generate_ir(t2)))
        }
        Term::Split(target, alias1, alias2, body) => {
            IRTerm::App(
                Box::new(IRTerm::App(
                    Box::new(IRTerm::Abs(
                        alias1.clone(), 
                        Box::new(IRTerm::Abs(alias2.clone(), Box::new(generate_ir(body))))
                    )),
                    Box::new(IRTerm::Var(target.clone()))
                )),
                Box::new(IRTerm::Var(target.clone()))
            )
        }
        Term::Merge(alias1, _alias2, target, body) => {
            IRTerm::App(
                Box::new(IRTerm::Abs(target.clone(), Box::new(generate_ir(body)))),
                Box::new(IRTerm::Var(alias1.clone()))
            )
        }
    }
}