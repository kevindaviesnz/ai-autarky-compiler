use crate::ir::IRTerm;

#[derive(Debug, Clone, PartialEq)]
pub enum Instruction {
    PushVar(String),
    PushInt(u32), // NEW
    Add,          // NEW
    MakeClosure(String, Vec<Instruction>),
    Call,
    Return,
}

pub fn generate_bytecode(ir: &IRTerm) -> Vec<Instruction> {
    match ir {
        IRTerm::IntVal(n) => {
            vec![Instruction::PushInt(*n)]
        }
        IRTerm::Add(t1, t2) => {
            let mut code = Vec::new();
            code.extend(generate_bytecode(t1)); // Evaluate left operand
            code.extend(generate_bytecode(t2)); // Evaluate right operand
            code.push(Instruction::Add);        // Add them together
            code
        }
        IRTerm::Var(name) => {
            vec![Instruction::PushVar(name.clone())]
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