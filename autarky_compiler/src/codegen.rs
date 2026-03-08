use crate::ir::IRTerm;

#[derive(Debug, Clone, PartialEq)]
pub enum Instruction {
    PushVar(String),
    PushInt(u32),
    PushUnit, // NEW
    Add,      
    MakeClosure(String, Vec<Instruction>),
    Call,
    Return,
    Free, // NEW
}

pub fn generate_bytecode(ir: &IRTerm) -> Vec<Instruction> {
    match ir {
        IRTerm::IntVal(n) => vec![Instruction::PushInt(*n)],
        IRTerm::UnitVal => vec![Instruction::PushUnit], // NEW
        IRTerm::Add(t1, t2) => {
            let mut code = Vec::new();
            code.extend(generate_bytecode(t1)); 
            code.extend(generate_bytecode(t2)); 
            code.push(Instruction::Add);        
            code
        }
        IRTerm::Var(name) => vec![Instruction::PushVar(name.clone())],
        IRTerm::Free(target) => { // NEW
            let mut code = generate_bytecode(target);
            code.push(Instruction::Free);
            code
        }
        IRTerm::Abs(param, body) => {
            let mut body_code = generate_bytecode(body);
            body_code.push(Instruction::Return);
            vec![Instruction::MakeClosure(param.clone(), body_code)]
        }
        IRTerm::App(t1, t2) => {
            let mut code = Vec::new();
            code.extend(generate_bytecode(t2));
            code.extend(generate_bytecode(t1));
            code.push(Instruction::Call);
            code
        }
        IRTerm::Erased => vec![],
    }
}