use crate::codegen::Instruction;
use std::collections::HashMap;

/// The types of values that can exist in our VM's memory at runtime.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub enum Value {
    /// A closure containing its parameter name and executable body.
    Closure(String, Vec<Instruction>),
    /// A raw memory address (to simulate our linear pointer).
    MemoryAddress(usize),
}

pub struct VM {
    /// The execution stack
    stack: Vec<Value>,
    /// The runtime environment (mapping variable names to memory values).
    /// In a production VM, this would be a chain of call frames.
    env: HashMap<String, Value>,
}

impl VM {
    pub fn new() -> Self {
        Self {
            stack: Vec::new(),
            env: HashMap::new(),
        }
    }

    /// Seeds the VM environment with external/global variables
    pub fn insert_global(&mut self, name: String, val: Value) {
        self.env.insert(name, val);
    }

    /// Executes a block of bytecode instructions
    pub fn execute(&mut self, instructions: &[Instruction]) -> Result<Option<Value>, String> {
        let mut pc = 0; // Program Counter
        
        while pc < instructions.len() {
            match &instructions[pc] {
                Instruction::PushVar(name) => {
                    // Look up the variable in the environment and push it to the stack
                    if let Some(val) = self.env.get(name) {
                        self.stack.push(val.clone());
                    } else {
                        return Err(format!("Runtime Error: Unbound variable '{}'", name));
                    }
                }
                Instruction::MakeClosure(param, body) => {
                    self.stack.push(Value::Closure(param.clone(), body.clone()));
                }
                Instruction::Call => {
                    // Pop the function, then pop the argument off the stack
                    let func = self.stack.pop().ok_or("Runtime Error: Stack underflow (func)")?;
                    let arg = self.stack.pop().ok_or("Runtime Error: Stack underflow (arg)")?;

                    match func {
                        Value::Closure(param, body) => {
                            // Create a new VM scope for the function call
                            let mut call_frame = VM::new();
                            call_frame.env = self.env.clone(); // Inherit globals
                            
                            // Bind the argument to the parameter name in the new scope
                            call_frame.env.insert(param, arg);
                            
                            // Execute the closure body
                            if let Some(ret_val) = call_frame.execute(&body)? {
                                self.stack.push(ret_val);
                            }
                        }
                        _ => return Err("Runtime Error: Attempted to call a non-closure".to_string()),
                    }
                }
                Instruction::Return => {
                    // Return the top value of the stack
                    return Ok(self.stack.pop());
                }
            }
            pc += 1;
        }
        
        // If the program ends without an explicit return, pop the final result
        Ok(self.stack.pop())
    }
}