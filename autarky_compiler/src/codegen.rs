use crate::ir::IRTerm;

#[derive(Debug, Clone, PartialEq)]
pub enum Instruction {
    /// Pushes a local variable onto the execution stack
    PushVar(String),
    /// Creates a closure in memory, containing the parameter and its compiled body
    MakeClosure(String, Vec<Instruction>),
    /// Pops an argument and a closure from the stack, applies them, and pushes the result
    Call,
    /// Explicitly halts execution or returns from a scope
    Return,
}

/// Lowers the Intermediate Representation into linear bytecode instructions.
pub fn generate_bytecode(ir: &IRTerm) -> Vec<Instruction> {
    match ir {
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
            // In a stack machine, we evaluate arguments first
            code.extend(generate_bytecode(t2));
            // Then evaluate the function
            code.extend(generate_bytecode(t1));
            // Then execute the application
            code.push(Instruction::Call);
            code
        }
        IRTerm::Erased => {
            // Erased proofs generate zero runtime bytecode!
            vec![]
        }
    }
}