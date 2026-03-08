use crate::ast::Term;

#[derive(Debug, Clone, PartialEq)]
pub enum IRTerm {
    Var(String),
    Abs(String, Box<IRTerm>),
    App(Box<IRTerm>, Box<IRTerm>),
    #[allow(dead_code)]
    Erased,
}

pub fn generate_ir(ast: &Term) -> IRTerm {
    match ast {
        Term::Var(name) => IRTerm::Var(name.clone()),
        Term::Abs(param, _type_annotation, body) => {
            IRTerm::Abs(param.clone(), Box::new(generate_ir(body)))
        }
        Term::App(t1, t2) => {
            IRTerm::App(Box::new(generate_ir(t1)), Box::new(generate_ir(t2)))
        }
        Term::Split(target, alias1, alias2, body) => {
            // ERASURE: A fractional split is erased into a double function application
            // Equivalent to: (\alias1 . \alias2 . body) target target
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
    }
}