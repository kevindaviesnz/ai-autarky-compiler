use crate::ast::Term;

#[derive(Debug, Clone, PartialEq)]
pub enum IRTerm {
    Var(String), IntVal(u32), UnitVal, BoolVal(bool), StringVal(String), // NEW
    Left(Box<IRTerm>), Right(Box<IRTerm>), Match(Box<IRTerm>, String, Box<IRTerm>, String, Box<IRTerm>),
    Alloc(Box<IRTerm>, Box<IRTerm>), Read(Box<IRTerm>, Box<IRTerm>), Write(Box<IRTerm>, Box<IRTerm>, Box<IRTerm>), ReadFile(Box<IRTerm>), // NEW
    MkPair(Box<IRTerm>, Box<IRTerm>), Unpack(Box<IRTerm>, String, String, Box<IRTerm>),
    If(Box<IRTerm>, Box<IRTerm>, Box<IRTerm>), Add(Box<IRTerm>, Box<IRTerm>), Sub(Box<IRTerm>, Box<IRTerm>), Eq(Box<IRTerm>, Box<IRTerm>),
    Fix(Box<IRTerm>), Abs(String, Box<IRTerm>), App(Box<IRTerm>, Box<IRTerm>), Free(Box<IRTerm>),
    #[allow(dead_code)] Erased,
}

pub fn generate_ir(ast: &Term) -> IRTerm {
    match ast {
        Term::IntVal(n) => IRTerm::IntVal(*n), Term::UnitVal => IRTerm::UnitVal, Term::BoolVal(b) => IRTerm::BoolVal(*b), 
        Term::StringVal(s) => IRTerm::StringVal(s.clone()), // NEW
        Term::Left(t, _) => IRTerm::Left(Box::new(generate_ir(t))), Term::Right(_, t) => IRTerm::Right(Box::new(generate_ir(t))),
        Term::Match(tgt, id_l, b_l, id_r, b_r) => IRTerm::Match(Box::new(generate_ir(tgt)), id_l.clone(), Box::new(generate_ir(b_l)), id_r.clone(), Box::new(generate_ir(b_r))),
        Term::Alloc(s, i) => IRTerm::Alloc(Box::new(generate_ir(s)), Box::new(generate_ir(i))),
        Term::Read(a, i) => IRTerm::Read(Box::new(generate_ir(a)), Box::new(generate_ir(i))),
        Term::Write(a, i, v) => IRTerm::Write(Box::new(generate_ir(a)), Box::new(generate_ir(i)), Box::new(generate_ir(v))),
        Term::ReadFile(p) => IRTerm::ReadFile(Box::new(generate_ir(p))), // NEW
        Term::MkPair(t1, t2) => IRTerm::MkPair(Box::new(generate_ir(t1)), Box::new(generate_ir(t2))),
        Term::Unpack(tgt, a1, a2, b) => IRTerm::Unpack(Box::new(generate_ir(tgt)), a1.clone(), a2.clone(), Box::new(generate_ir(b))),
        Term::If(c, t, f) => IRTerm::If(Box::new(generate_ir(c)), Box::new(generate_ir(t)), Box::new(generate_ir(f))),
        Term::Add(t1, t2) => IRTerm::Add(Box::new(generate_ir(t1)), Box::new(generate_ir(t2))), Term::Sub(t1, t2) => IRTerm::Sub(Box::new(generate_ir(t1)), Box::new(generate_ir(t2))), Term::Eq(t1, t2) => IRTerm::Eq(Box::new(generate_ir(t1)), Box::new(generate_ir(t2))),
        Term::Fix(i) => IRTerm::Fix(Box::new(generate_ir(i))), Term::Var(name) => IRTerm::Var(name.clone()), Term::Free(tgt) => IRTerm::Free(Box::new(generate_ir(tgt))),
        Term::Abs(p, _, b) => IRTerm::Abs(p.clone(), Box::new(generate_ir(b))), Term::App(t1, t2) => IRTerm::App(Box::new(generate_ir(t1)), Box::new(generate_ir(t2))),
        Term::Split(tgt, a1, a2, b) => IRTerm::App(Box::new(IRTerm::App(Box::new(IRTerm::Abs(a1.clone(), Box::new(IRTerm::Abs(a2.clone(), Box::new(generate_ir(b)))))), Box::new(IRTerm::Var(tgt.clone())))), Box::new(IRTerm::Var(tgt.clone()))),
        Term::Merge(a1, _, tgt, b) => IRTerm::App(Box::new(IRTerm::Abs(tgt.clone(), Box::new(generate_ir(b)))), Box::new(IRTerm::Var(a1.clone()))),
    }
}