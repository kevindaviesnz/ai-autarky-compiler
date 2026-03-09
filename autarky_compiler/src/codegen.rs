use crate::ir::IRTerm;

#[derive(Debug, Clone, PartialEq)]
pub enum Instruction {
    PushVar(String), PushInt(u32), PushUnit, PushBool(bool), PushString(String), // NEW
    MakeLeft, MakeRight, BranchMatch(usize), Bind(String),
    AllocArray, ReadArray, WriteArray, ReadFileOS, // NEW
    MakePair, UnpackAndBind(String, String), JumpIfFalse(usize), Jump(usize),
    Add, Sub, Eq, Fix, MakeClosure(String, Vec<Instruction>), Call, Return, Free,
}

pub fn generate_bytecode(ir: &IRTerm) -> Vec<Instruction> {
    match ir {
        IRTerm::IntVal(n) => vec![Instruction::PushInt(*n)], IRTerm::UnitVal => vec![Instruction::PushUnit], IRTerm::BoolVal(b) => vec![Instruction::PushBool(*b)], 
        IRTerm::StringVal(s) => vec![Instruction::PushString(s.clone())], // NEW
        IRTerm::Left(t) => { let mut c = generate_bytecode(t); c.push(Instruction::MakeLeft); c }
        IRTerm::Right(t) => { let mut c = generate_bytecode(t); c.push(Instruction::MakeRight); c }
        IRTerm::Match(tgt, id_l, b_l, id_r, b_r) => { 
            let mut c = generate_bytecode(tgt); let mut lc = generate_bytecode(b_l); let mut rc = generate_bytecode(b_r);
            c.push(Instruction::BranchMatch(lc.len() + 2)); c.push(Instruction::Bind(id_l.clone())); c.append(&mut lc); c.push(Instruction::Jump(rc.len() + 1)); c.push(Instruction::Bind(id_r.clone())); c.append(&mut rc); c
        }
        IRTerm::Alloc(s, i) => { let mut c = generate_bytecode(s); c.extend(generate_bytecode(i)); c.push(Instruction::AllocArray); c }
        IRTerm::Read(a, i) => { let mut c = generate_bytecode(a); c.extend(generate_bytecode(i)); c.push(Instruction::ReadArray); c }
        IRTerm::Write(a, i, v) => { let mut c = generate_bytecode(a); c.extend(generate_bytecode(i)); c.extend(generate_bytecode(v)); c.push(Instruction::WriteArray); c }
        IRTerm::ReadFile(p) => { let mut c = generate_bytecode(p); c.push(Instruction::ReadFileOS); c } // NEW
        IRTerm::MkPair(t1, t2) => { let mut c = Vec::new(); c.extend(generate_bytecode(t1)); c.extend(generate_bytecode(t2)); c.push(Instruction::MakePair); c }
        IRTerm::Unpack(tgt, a1, a2, b) => { let mut c = generate_bytecode(tgt); c.push(Instruction::UnpackAndBind(a1.clone(), a2.clone())); c.extend(generate_bytecode(b)); c }
        IRTerm::If(c, t, f) => { let mut code = generate_bytecode(c); let mut tc = generate_bytecode(t); let mut fc = generate_bytecode(f); code.push(Instruction::JumpIfFalse(tc.len() + 2)); code.append(&mut tc); code.push(Instruction::Jump(fc.len() + 1)); code.append(&mut fc); code }
        IRTerm::Add(t1, t2) => { let mut c = generate_bytecode(t1); c.extend(generate_bytecode(t2)); c.push(Instruction::Add); c }
        IRTerm::Sub(t1, t2) => { let mut c = generate_bytecode(t1); c.extend(generate_bytecode(t2)); c.push(Instruction::Sub); c }
        IRTerm::Eq(t1, t2) => { let mut c = generate_bytecode(t1); c.extend(generate_bytecode(t2)); c.push(Instruction::Eq); c }
        IRTerm::Fix(i) => { let mut c = generate_bytecode(i); c.push(Instruction::Fix); c }
        IRTerm::Var(n) => vec![Instruction::PushVar(n.clone())], IRTerm::Free(tgt) => { let mut c = generate_bytecode(tgt); c.push(Instruction::Free); c }
        IRTerm::Abs(p, b) => { let mut bc = generate_bytecode(b); bc.push(Instruction::Return); vec![Instruction::MakeClosure(p.clone(), bc)] }
        IRTerm::App(t1, t2) => { let mut c = generate_bytecode(t2); c.extend(generate_bytecode(t1)); c.push(Instruction::Call); c }
        IRTerm::Erased => vec![],
    }
}