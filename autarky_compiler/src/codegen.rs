use crate::ir::IRTerm;

#[derive(Debug, Clone, PartialEq)]
pub enum Instruction {
    PushInt(u32), PushUnit, PushBool(bool), PushString(String),
    AllocArray, ReadArray, WriteArray, ReadFileOS,
    MakeLeft, MakeRight, BranchMatch(usize),
    Bind(String), MakePair, UnpackAndBind(String, String),
    JumpIfFalse(usize), Jump(usize), Add, Sub, Eq, Fix,
    PushVar(String), MakeClosure(String, Vec<Instruction>), Call, Free, Return
}

pub fn generate_bytecode(ir: &IRTerm) -> Vec<Instruction> {
    let mut code = Vec::new();
    match ir {
        IRTerm::IntVal(n) => code.push(Instruction::PushInt(*n)),
        IRTerm::UnitVal => code.push(Instruction::PushUnit),
        IRTerm::BoolVal(b) => code.push(Instruction::PushBool(*b)),
        IRTerm::StringVal(s) => code.push(Instruction::PushString(s.clone())),
        IRTerm::Left(t) => { 
            code.extend(generate_bytecode(t)); 
            code.push(Instruction::MakeLeft); 
        }
        IRTerm::Right(t) => { 
            code.extend(generate_bytecode(t)); 
            code.push(Instruction::MakeRight); 
        }
        IRTerm::Match(tgt, id_l, b_l, id_r, b_r) => {
            code.extend(generate_bytecode(tgt));
            let mut code_l = Vec::new(); 
            code_l.push(Instruction::Bind(id_l.clone())); 
            code_l.extend(generate_bytecode(b_l));
            
            let mut code_r = Vec::new(); 
            code_r.push(Instruction::Bind(id_r.clone())); 
            code_r.extend(generate_bytecode(b_r));
            
            code_l.push(Instruction::Jump(code_r.len() + 1));
            code.push(Instruction::BranchMatch(code_l.len() + 1));
            code.extend(code_l); 
            code.extend(code_r);
        }
        IRTerm::Alloc(s, i) => { 
            code.extend(generate_bytecode(s)); 
            code.extend(generate_bytecode(i)); 
            code.push(Instruction::AllocArray); 
        }
        IRTerm::Read(a, i) => { 
            code.extend(generate_bytecode(a)); 
            code.extend(generate_bytecode(i)); 
            code.push(Instruction::ReadArray); 
        }
        IRTerm::Write(a, i, v) => { 
            code.extend(generate_bytecode(a)); 
            code.extend(generate_bytecode(i)); 
            code.extend(generate_bytecode(v)); 
            code.push(Instruction::WriteArray); 
        }
        IRTerm::ReadFile(p) => { 
            code.extend(generate_bytecode(p)); 
            code.push(Instruction::ReadFileOS); 
        }
        IRTerm::MkPair(t1, t2) => { 
            code.extend(generate_bytecode(t1)); 
            code.extend(generate_bytecode(t2)); 
            code.push(Instruction::MakePair); 
        }
        IRTerm::Unpack(tgt, a1, a2, b) => {
            code.extend(generate_bytecode(tgt));
            code.push(Instruction::UnpackAndBind(a1.clone(), a2.clone()));
            code.extend(generate_bytecode(b));
        }
        IRTerm::If(c, t, f) => {
            code.extend(generate_bytecode(c));
            let mut t_code = generate_bytecode(t); 
            let f_code = generate_bytecode(f);
            t_code.push(Instruction::Jump(f_code.len() + 1));
            code.push(Instruction::JumpIfFalse(t_code.len() + 1));
            code.extend(t_code); 
            code.extend(f_code);
        }
        IRTerm::Add(t1, t2) => { 
            code.extend(generate_bytecode(t1)); 
            code.extend(generate_bytecode(t2)); 
            code.push(Instruction::Add); 
        }
        IRTerm::Sub(t1, t2) => { 
            code.extend(generate_bytecode(t1)); 
            code.extend(generate_bytecode(t2)); 
            code.push(Instruction::Sub); 
        }
        IRTerm::Eq(t1, t2) => { 
            code.extend(generate_bytecode(t1)); 
            code.extend(generate_bytecode(t2)); 
            code.push(Instruction::Eq); 
        }
        IRTerm::Fix(i) => { 
            code.extend(generate_bytecode(i)); 
            code.push(Instruction::Fix); 
        }
        IRTerm::Var(name) => {
            code.push(Instruction::PushVar(name.clone()));
        }
        IRTerm::Free(tgt) => { 
            code.extend(generate_bytecode(tgt)); 
            code.push(Instruction::Free); 
        }
        IRTerm::Abs(p, b) => {
            code.push(Instruction::MakeClosure(p.clone(), generate_bytecode(b)));
        }
        IRTerm::App(t1, t2) => { 
            // The argument must be pushed to the stack first, then the closure
            code.extend(generate_bytecode(t2)); 
            code.extend(generate_bytecode(t1)); 
            code.push(Instruction::Call); 
        }
        IRTerm::Erased => {}
    }
    code
}