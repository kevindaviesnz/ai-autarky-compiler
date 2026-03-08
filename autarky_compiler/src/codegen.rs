use crate::ir::IRTerm;

#[derive(Debug, Clone, PartialEq)]
pub enum Instruction {
    PushVar(String),
    PushInt(u32),
    PushUnit, 
    PushBool(bool), 
    MakePair, 
    UnpackAndBind(String, String), 
    JumpIfFalse(usize), 
    Jump(usize), 
    Add,      
    Sub,      // NEW
    Eq,       // NEW
    Fix,      // NEW
    MakeClosure(String, Vec<Instruction>),
    Call,
    Return,
    Free, 
}

pub fn generate_bytecode(ir: &IRTerm) -> Vec<Instruction> {
    match ir {
        IRTerm::IntVal(n) => vec![Instruction::PushInt(*n)],
        IRTerm::UnitVal => vec![Instruction::PushUnit], 
        IRTerm::BoolVal(b) => vec![Instruction::PushBool(*b)], 
        IRTerm::MkPair(t1, t2) => { 
            let mut code = Vec::new();
            code.extend(generate_bytecode(t1));
            code.extend(generate_bytecode(t2));
            code.push(Instruction::MakePair);
            code
        }
        IRTerm::Unpack(target, alias1, alias2, body) => { 
            let mut code = generate_bytecode(target);
            code.push(Instruction::UnpackAndBind(alias1.clone(), alias2.clone()));
            code.extend(generate_bytecode(body));
            code
        }
        IRTerm::If(cond, t_branch, f_branch) => { 
            let mut code = generate_bytecode(cond);
            let mut t_code = generate_bytecode(t_branch);
            let mut f_code = generate_bytecode(f_branch);
            
            code.push(Instruction::JumpIfFalse(t_code.len() + 2));
            code.append(&mut t_code);
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
        IRTerm::Sub(t1, t2) => { // NEW
            let mut code = Vec::new();
            code.extend(generate_bytecode(t1)); 
            code.extend(generate_bytecode(t2)); 
            code.push(Instruction::Sub);        
            code
        }
        IRTerm::Eq(t1, t2) => { // NEW
            let mut code = Vec::new();
            code.extend(generate_bytecode(t1)); 
            code.extend(generate_bytecode(t2)); 
            code.push(Instruction::Eq);        
            code
        }
        IRTerm::Fix(inner) => { // NEW
            let mut code = generate_bytecode(inner);
            code.push(Instruction::Fix);
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