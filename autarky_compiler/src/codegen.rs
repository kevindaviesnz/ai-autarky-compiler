use crate::ir::IRTerm;

#[derive(Debug, Clone, PartialEq)]
pub enum Instruction {
    PushVar(String),
    PushInt(u32),
    PushUnit, 
    PushBool(bool), // NEW
    JumpIfFalse(usize), // NEW: PC relative jump
    Jump(usize), // NEW: PC relative jump
    Add,      
    MakeClosure(String, Vec<Instruction>),
    Call,
    Return,
    Free, 
}

pub fn generate_bytecode(ir: &IRTerm) -> Vec<Instruction> {
    match ir {
        IRTerm::IntVal(n) => vec![Instruction::PushInt(*n)],
        IRTerm::UnitVal => vec![Instruction::PushUnit], 
        IRTerm::BoolVal(b) => vec![Instruction::PushBool(*b)], // NEW
        IRTerm::If(cond, t_branch, f_branch) => { // NEW: Emitting Jump offsets
            let mut code = generate_bytecode(cond);
            let mut t_code = generate_bytecode(t_branch);
            let mut f_code = generate_bytecode(f_branch);
            
            // If condition is false, jump over the True branch AND the unconditional jump
            code.push(Instruction::JumpIfFalse(t_code.len() + 2));
            code.append(&mut t_code);
            // After executing True branch, jump over the False branch
            code.push(Instruction::Jump(f_code.len() + 1));
            code.append(&mut f_code);
            
            code
        }
        IRTerm::Add(t1, t2) => {
            let mut code = Vec::new();
            code.extend(generate_bytecode(t1)); 
            code.extend(generate_bytecode(t2)); 
            code.push(Instruction::Add);        
            code
        }
        IRTerm::Var(name) => vec![Instruction::PushVar(name.clone())],
        IRTerm::Free(target) => { 
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