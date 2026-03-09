use crate::codegen::Instruction;
use std::collections::HashMap;

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub enum Value {
    Closure(String, Vec<Instruction>, HashMap<String, Value>),
    RecursiveClosure(String, Vec<Instruction>, HashMap<String, Value>),
    MemoryAddress(usize), ArrayAddress(usize),
    Int(u32), Unit, Bool(bool), String(String), // NEW
    Pair(Box<Value>, Box<Value>), Left(Box<Value>), Right(Box<Value>),
}

pub struct VM { stack: Vec<Value>, env: HashMap<String, Value>, heap: HashMap<usize, Vec<Value>>, next_alloc_ptr: usize }

impl VM {
    pub fn new() -> Self { Self { stack: Vec::new(), env: HashMap::new(), heap: HashMap::new(), next_alloc_ptr: 0x1000 } }
    pub fn insert_global(&mut self, name: String, val: Value) { self.env.insert(name, val); }
    pub fn execute(&mut self, instructions: &[Instruction]) -> Result<Option<Value>, String> {
        let mut pc = 0;
        while pc < instructions.len() {
            match &instructions[pc] {
                Instruction::PushInt(n) => self.stack.push(Value::Int(*n)), Instruction::PushUnit => self.stack.push(Value::Unit),
                Instruction::PushBool(b) => self.stack.push(Value::Bool(*b)), Instruction::PushString(s) => self.stack.push(Value::String(s.clone())),
                Instruction::AllocArray => {
                    let init = self.stack.pop().unwrap(); let size = self.stack.pop().unwrap();
                    if let Value::Int(s) = size {
                        let id = self.next_alloc_ptr; self.next_alloc_ptr += 0x10;
                        self.heap.insert(id, vec![init; s as usize]);
                        self.stack.push(Value::ArrayAddress(id));
                    }
                }
                Instruction::ReadFileOS => { // NEW: Interacting with the real host system
                    if let Value::String(path) = self.stack.pop().unwrap() {
                        let bytes = std::fs::read(&path).map_err(|e| format!("IO Error: {}", e))?;
                        let id = self.next_alloc_ptr; self.next_alloc_ptr += 0x10;
                        let mut arr = Vec::new();
                        for b in bytes { arr.push(Value::Int(b as u32)); }
                        self.heap.insert(id, arr.clone());
                        println!("💾 [VM] File System Read: Loaded '{}' ({} bytes) directly into linear heap address: {:#X}", path, arr.len(), id);
                        self.stack.push(Value::ArrayAddress(id));
                    } else { return Err("ReadFileOS requires String".to_string()); }
                }
                Instruction::ReadArray => {
                    let idx = self.stack.pop().unwrap(); let arr = self.stack.pop().unwrap();
                    if let (Value::ArrayAddress(id), Value::Int(i)) = (arr, idx) {
                        let vec = self.heap.get(&id).unwrap();
                        let val = vec[i as usize].clone();
                        self.stack.push(Value::Pair(Box::new(val), Box::new(Value::ArrayAddress(id))));
                    }
                }
                Instruction::WriteArray => {
                    let val = self.stack.pop().unwrap(); let idx = self.stack.pop().unwrap(); let arr = self.stack.pop().unwrap();
                    if let (Value::ArrayAddress(id), Value::Int(i)) = (arr, idx) {
                        let vec = self.heap.get_mut(&id).unwrap(); vec[i as usize] = val;
                        self.stack.push(Value::ArrayAddress(id));
                    }
                }
                Instruction::MakeLeft => { let val = self.stack.pop().unwrap(); self.stack.push(Value::Left(Box::new(val))); }
                Instruction::MakeRight => { let val = self.stack.pop().unwrap(); self.stack.push(Value::Right(Box::new(val))); }
                Instruction::BranchMatch(offset) => {
                    match self.stack.pop().unwrap() { Value::Left(val) => self.stack.push(*val), Value::Right(val) => { self.stack.push(*val); pc += offset; continue; }, _ => return Err("Err".to_string()) }
                }
                Instruction::Bind(name) => { let val = self.stack.pop().unwrap(); self.env.insert(name.clone(), val); }
                Instruction::MakePair => { let right = self.stack.pop().unwrap(); let left = self.stack.pop().unwrap(); self.stack.push(Value::Pair(Box::new(left), Box::new(right))); }
                Instruction::UnpackAndBind(a1, a2) => { if let Value::Pair(l, r) = self.stack.pop().unwrap() { self.env.insert(a1.clone(), *l); self.env.insert(a2.clone(), *r); } }
                Instruction::JumpIfFalse(offset) => { if let Value::Bool(b) = self.stack.pop().unwrap() { if !b { pc += offset; continue; } } }
                Instruction::Jump(offset) => { pc += offset; continue; }
                Instruction::Add => { let r = self.stack.pop().unwrap(); let l = self.stack.pop().unwrap(); if let (Value::Int(l), Value::Int(r)) = (l, r) { self.stack.push(Value::Int(l + r)); } }
                Instruction::Sub => { let r = self.stack.pop().unwrap(); let l = self.stack.pop().unwrap(); if let (Value::Int(l), Value::Int(r)) = (l, r) { self.stack.push(Value::Int(if r > l { 0 } else { l - r })); } }
                Instruction::Eq => { let r = self.stack.pop().unwrap(); let l = self.stack.pop().unwrap(); if let (Value::Int(l), Value::Int(r)) = (l, r) { self.stack.push(Value::Bool(l == r)); } }
                Instruction::Fix => {
                    if let Value::Closure(param, body, env) = self.stack.pop().unwrap() {
                        let rec_ref = Value::RecursiveClosure(param.clone(), body.clone(), env.clone());
                        let old_env = std::mem::replace(&mut self.env, env); self.env.insert(param, rec_ref);
                        if let Some(ret_val) = self.execute(&body)? { self.stack.push(ret_val); } self.env = old_env;
                    }
                }
                Instruction::PushVar(name) => { if let Some(val) = self.env.get(name) { self.stack.push(val.clone()); } else { return Err("Err".to_string()); } }
                Instruction::MakeClosure(param, body) => { self.stack.push(Value::Closure(param.clone(), body.clone(), self.env.clone())); }
                Instruction::Call => {
                    let func = self.stack.pop().unwrap(); let arg = self.stack.pop().unwrap();
                    match func {
                        Value::Closure(param, body, captured_env) => {
                            let old_env = std::mem::replace(&mut self.env, captured_env); self.env.insert(param, arg);
                            if let Some(ret_val) = self.execute(&body)? { self.stack.push(ret_val); } self.env = old_env;
                        }
                        Value::RecursiveClosure(param, body, captured_env) => {
                            let old_env = std::mem::replace(&mut self.env, captured_env); let rec_ref = Value::RecursiveClosure(param.clone(), body.clone(), self.env.clone()); self.env.insert(param.clone(), rec_ref);
                            let unrolled_func = self.execute(&body)?.unwrap(); self.env = old_env;
                            if let Value::Closure(u_param, u_body, u_env) = unrolled_func { let old_env2 = std::mem::replace(&mut self.env, u_env); self.env.insert(u_param, arg); if let Some(ret_val) = self.execute(&u_body)? { self.stack.push(ret_val); } self.env = old_env2; }
                        }
                        _ => return Err("Err".to_string()),
                    }
                }
                Instruction::Free => {
                    match self.stack.pop().unwrap() {
                        Value::MemoryAddress(_) => { self.stack.push(Value::Unit); }
                        Value::ArrayAddress(addr) => { if self.heap.remove(&addr).is_some() { self.stack.push(Value::Unit); } }
                        _ => return Err("Err".to_string()),
                    }
                }
                Instruction::Return => { return Ok(self.stack.pop()); }
            }
            pc += 1;
        }
        Ok(self.stack.pop())
    }
}