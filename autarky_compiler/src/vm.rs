use crate::codegen::Instruction;
use std::collections::HashMap;

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub enum Value {
    Closure(String, Vec<Instruction>, HashMap<String, Value>),
    MemoryAddress(usize),
    Int(u32),
    Unit,
    Bool(bool), // NEW
}

pub struct VM {
    stack: Vec<Value>,
    env: HashMap<String, Value>,
}

impl VM {
    pub fn new() -> Self {
        Self {
            stack: Vec::new(),
            env: HashMap::new(),
        }
    }

    pub fn insert_global(&mut self, name: String, val: Value) {
        self.env.insert(name, val);
    }

    pub fn execute(&mut self, instructions: &[Instruction]) -> Result<Option<Value>, String> {
        let mut pc = 0; 
        
        while pc < instructions.len() {
            match &instructions[pc] {
                Instruction::PushInt(n) => self.stack.push(Value::Int(*n)),
                Instruction::PushUnit => self.stack.push(Value::Unit),
                Instruction::PushBool(b) => self.stack.push(Value::Bool(*b)), // NEW
                
                Instruction::JumpIfFalse(offset) => { // NEW
                    let condition = self.stack.pop().ok_or("Runtime Error: Stack underflow on JumpIfFalse")?;
                    match condition {
                        Value::Bool(b) => {
                            if !b {
                                pc += offset;
                                continue; // Skip the standard pc += 1 at the end of the loop
                            }
                        }
                        _ => return Err("Runtime Error: Expected Bool for conditional branch".to_string()),
                    }
                }
                Instruction::Jump(offset) => { // NEW
                    pc += offset;
                    continue; 
                }

                Instruction::Add => {
                    let right = self.stack.pop().ok_or("Runtime Error: Stack underflow on Add (right)")?;
                    let left = self.stack.pop().ok_or("Runtime Error: Stack underflow on Add (left)")?;
                    
                    match (left, right) {
                        (Value::Int(l), Value::Int(r)) => {
                            self.stack.push(Value::Int(l + r));
                        }
                        _ => return Err("Runtime Error: Attempted to add non-integers".to_string()),
                    }
                }
                Instruction::PushVar(name) => {
                    if let Some(val) = self.env.get(name) {
                        self.stack.push(val.clone());
                    } else {
                        return Err(format!("Runtime Error: Unbound variable '{}'", name));
                    }
                }
                Instruction::MakeClosure(param, body) => {
                    self.stack.push(Value::Closure(param.clone(), body.clone(), self.env.clone()));
                }
                Instruction::Call => {
                    let func = self.stack.pop().ok_or("Runtime Error: Stack underflow (func)")?;
                    let arg = self.stack.pop().ok_or("Runtime Error: Stack underflow (arg)")?;

                    match func {
                        Value::Closure(param, body, captured_env) => {
                            let mut call_frame = VM::new();
                            call_frame.env = captured_env; 
                            call_frame.env.insert(param, arg);
                            
                            if let Some(ret_val) = call_frame.execute(&body)? {
                                self.stack.push(ret_val);
                            }
                        }
                        _ => return Err("Runtime Error: Attempted to call a non-closure".to_string()),
                    }
                }
                Instruction::Free => { 
                    let val = self.stack.pop().ok_or("Runtime Error: Stack underflow on Free")?;
                    match val {
                        Value::MemoryAddress(addr) => {
                            println!("💀 [VM] Safely deallocating memory at address: {:#X}", addr);
                            self.stack.push(Value::Unit); 
                        }
                        _ => return Err("Runtime Error: Attempted to free a non-memory resource".to_string()),
                    }
                }
                Instruction::Return => {
                    return Ok(self.stack.pop());
                }
            }
            pc += 1;
        }
        
        Ok(self.stack.pop())
    }
}