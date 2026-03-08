use crate::codegen::Instruction;
use std::collections::HashMap;

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub enum Value {
    Closure(String, Vec<Instruction>, HashMap<String, Value>),
    RecursiveClosure(String, Vec<Instruction>, HashMap<String, Value>), // NEW: The lazy unrolling fixed-point
    MemoryAddress(usize),
    Int(u32),
    Unit,
    Bool(bool),
    Pair(Box<Value>, Box<Value>), 
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
                Instruction::PushBool(b) => self.stack.push(Value::Bool(*b)), 
                
                Instruction::MakePair => { 
                    let right = self.stack.pop().ok_or("Runtime Error: Stack underflow on MakePair (right)")?;
                    let left = self.stack.pop().ok_or("Runtime Error: Stack underflow on MakePair (left)")?;
                    self.stack.push(Value::Pair(Box::new(left), Box::new(right)));
                }
                Instruction::UnpackAndBind(alias1, alias2) => { 
                    let target = self.stack.pop().ok_or("Runtime Error: Stack underflow on Unpack")?;
                    match target {
                        Value::Pair(left, right) => {
                            self.env.insert(alias1.clone(), *left);
                            self.env.insert(alias2.clone(), *right);
                        }
                        _ => return Err("Runtime Error: Attempted to unpack a non-pair value".to_string()),
                    }
                }
                Instruction::JumpIfFalse(offset) => { 
                    let condition = self.stack.pop().ok_or("Runtime Error: Stack underflow on JumpIfFalse")?;
                    match condition {
                        Value::Bool(b) => {
                            if !b {
                                pc += offset;
                                continue; 
                            }
                        }
                        _ => return Err("Runtime Error: Expected Bool for conditional branch".to_string()),
                    }
                }
                Instruction::Jump(offset) => { 
                    pc += offset;
                    continue; 
                }

                Instruction::Add => {
                    let right = self.stack.pop().ok_or("Runtime Error: Stack underflow on Add (right)")?;
                    let left = self.stack.pop().ok_or("Runtime Error: Stack underflow on Add (left)")?;
                    match (left, right) {
                        (Value::Int(l), Value::Int(r)) => self.stack.push(Value::Int(l + r)),
                        _ => return Err("Runtime Error: Attempted to add non-integers".to_string()),
                    }
                }
                Instruction::Sub => { // NEW
                    let right = self.stack.pop().ok_or("Runtime Error: Stack underflow on Sub (right)")?;
                    let left = self.stack.pop().ok_or("Runtime Error: Stack underflow on Sub (left)")?;
                    match (left, right) {
                        (Value::Int(l), Value::Int(r)) => {
                            let result = if r > l { 0 } else { l - r }; // Prevent underflow crash for simple demo
                            self.stack.push(Value::Int(result));
                        }
                        _ => return Err("Runtime Error: Attempted to subtract non-integers".to_string()),
                    }
                }
                Instruction::Eq => { // NEW
                    let right = self.stack.pop().ok_or("Runtime Error: Stack underflow on Eq (right)")?;
                    let left = self.stack.pop().ok_or("Runtime Error: Stack underflow on Eq (left)")?;
                    match (left, right) {
                        (Value::Int(l), Value::Int(r)) => self.stack.push(Value::Bool(l == r)),
                        _ => return Err("Runtime Error: Attempted to equate non-integers".to_string()),
                    }
                }
                Instruction::Fix => { // NEW: Convert standard closure into RecursiveClosure
                    let func = self.stack.pop().ok_or("Runtime Error: Stack underflow on Fix")?;
                    match func {
                        Value::Closure(param, body, env) => {
                            let rec_ref = Value::RecursiveClosure(param.clone(), body.clone(), env.clone());
                            
                            let mut call_frame = VM::new();
                            call_frame.env = env;
                            call_frame.env.insert(param, rec_ref);
                            
                            if let Some(ret_val) = call_frame.execute(&body)? {
                                self.stack.push(ret_val);
                            }
                        }
                        _ => return Err("Runtime Error: 'fix' must be applied to a closure".to_string()),
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
                        Value::RecursiveClosure(param, body, captured_env) => { // NEW: Safely unroll and call
                            let rec_ref = Value::RecursiveClosure(param.clone(), body.clone(), captured_env.clone());
                            
                            let mut unroll_frame = VM::new();
                            unroll_frame.env = captured_env;
                            unroll_frame.env.insert(param, rec_ref);
                            
                            let unrolled_func = unroll_frame.execute(&body)?.ok_or("Runtime Error: Fix unroll failed")?;
                            
                            match unrolled_func {
                                Value::Closure(u_param, u_body, u_env) => {
                                    let mut call_frame = VM::new();
                                    call_frame.env = u_env;
                                    call_frame.env.insert(u_param, arg);
                                    if let Some(ret_val) = call_frame.execute(&u_body)? {
                                        self.stack.push(ret_val);
                                    }
                                }
                                _ => return Err("Runtime Error: Recursive unroll did not yield a closure".to_string()),
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