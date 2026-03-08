use crate::codegen::Instruction;
use std::collections::HashMap;

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub enum Value {
    Closure(String, Vec<Instruction>, HashMap<String, Value>),
    MemoryAddress(usize),
    Int(u32),
    Unit, // NEW
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
                Instruction::PushInt(n) => {
                    self.stack.push(Value::Int(*n));
                }
                Instruction::PushUnit => {
                    self.stack.push(Value::Unit);
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
                Instruction::Free => { // NEW: The native VM destruction call
                    let val = self.stack.pop().ok_or("Runtime Error: Stack underflow on Free")?;
                    match val {
                        Value::MemoryAddress(addr) => {
                            // In a real system, this is where we call drop() or free() on the heap allocator
                            println!("💀 [VM] Safely deallocating memory at address: {:#X}", addr);
                            self.stack.push(Value::Unit); // Replaces the pointer with void
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