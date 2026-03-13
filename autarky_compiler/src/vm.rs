// src/vm.rs

use std::collections::HashMap;
use crate::ir::IrNode;

#[derive(Debug, Clone)]
pub enum Value {
    Int(i64),
    Float(f64),
    Pair(Box<Value>, Box<Value>),
    Closure(String, IrNode, HashMap<String, Value>),
    RecClosure(String, String, IrNode, HashMap<String, Value>), 
}

pub enum Trampoline {
    Done(Value),
    TailCall(HashMap<String, Value>, IrNode),
}

pub struct VirtualMachine {}

impl VirtualMachine {
    pub fn new() -> Self {
        Self {}
    }

    pub fn evaluate(
        &mut self,
        env: &HashMap<String, Value>,
        node: &IrNode,
    ) -> Result<Value, String> {
        let mut current_env = env.clone();
        let mut current_node = node.clone();

        loop {
            match self.step(&current_env, &current_node)? {
                Trampoline::Done(val) => return Ok(val),
                Trampoline::TailCall(next_env, next_node) => {
                    current_env = next_env;
                    current_node = next_node;
                }
            }
        }
    }

    fn step(
        &mut self,
        env: &HashMap<String, Value>,
        node: &IrNode,
    ) -> Result<Trampoline, String> {
        match node {
            IrNode::Int(n) => Ok(Trampoline::Done(Value::Int(*n))),
            IrNode::Float(f) => Ok(Trampoline::Done(Value::Float(*f))),
            
            IrNode::Var(name) => {
                let val = env.get(name).cloned().ok_or_else(|| format!("VM Error: Undefined variable '{}'", name))?;
                Ok(Trampoline::Done(val))
            }
            
            IrNode::Add(l, r) => {
                let lv = self.evaluate(env, l)?;
                let rv = self.evaluate(env, r)?;
                match (lv, rv) {
                    (Value::Int(a), Value::Int(b)) => Ok(Trampoline::Done(Value::Int(a + b))),
                    (Value::Float(a), Value::Float(b)) => Ok(Trampoline::Done(Value::Float(a + b))),
                    _ => Err("VM Error: Type mismatch in Addition".to_string()),
                }
            }

            IrNode::Sub(l, r) => {
                let lv = self.evaluate(env, l)?;
                let rv = self.evaluate(env, r)?;
                match (lv, rv) {
                    (Value::Int(a), Value::Int(b)) => Ok(Trampoline::Done(Value::Int(a - b))),
                    (Value::Float(a), Value::Float(b)) => Ok(Trampoline::Done(Value::Float(a - b))),
                    _ => Err("VM Error: Type mismatch in Subtraction".to_string()),
                }
            }

            // NEW: Multiplication
            IrNode::Mul(l, r) => {
                let lv = self.evaluate(env, l)?;
                let rv = self.evaluate(env, r)?;
                match (lv, rv) {
                    (Value::Int(a), Value::Int(b)) => Ok(Trampoline::Done(Value::Int(a * b))),
                    (Value::Float(a), Value::Float(b)) => Ok(Trampoline::Done(Value::Float(a * b))),
                    _ => Err("VM Error: Type mismatch in Multiplication".to_string()),
                }
            }

            // NEW: Division (with zero-checking)
            IrNode::Div(l, r) => {
                let lv = self.evaluate(env, l)?;
                let rv = self.evaluate(env, r)?;
                match (lv, rv) {
                    (Value::Int(a), Value::Int(b)) => {
                        if b == 0 { return Err("VM Error: Division by zero".to_string()); }
                        Ok(Trampoline::Done(Value::Int(a / b)))
                    },
                    (Value::Float(a), Value::Float(b)) => {
                        if b == 0.0 { return Err("VM Error: Division by zero".to_string()); }
                        Ok(Trampoline::Done(Value::Float(a / b)))
                    },
                    _ => Err("VM Error: Type mismatch in Division".to_string()),
                }
            }

            IrNode::Eq(l, r) => {
                let lv = self.evaluate(env, l)?;
                let rv = self.evaluate(env, r)?;
                match (lv, rv) {
                    (Value::Int(a), Value::Int(b)) => {
                        Ok(Trampoline::Done(Value::Int(if a == b { 1 } else { 0 })))
                    },
                    (Value::Float(a), Value::Float(b)) => {
                        Ok(Trampoline::Done(Value::Int(if a == b { 1 } else { 0 })))
                    },
                    _ => Err("VM Error: Type mismatch in Equality".to_string()),
                }
            }

            IrNode::Lam(param, body) => {
                Ok(Trampoline::Done(Value::Closure(param.clone(), *body.clone(), env.clone())))
            }

            IrNode::App(func, arg) => {
                if let IrNode::Lam(bind_name, in_body) = &**func {
                    if let IrNode::Lam(fn_param, fn_body) = &**arg {
                        let rec_closure = Value::RecClosure(
                            bind_name.clone(), 
                            fn_param.clone(), 
                            *fn_body.clone(), 
                            env.clone()
                        );
                        let mut local_env = env.clone();
                        local_env.insert(bind_name.clone(), rec_closure);
                        
                        return Ok(Trampoline::TailCall(local_env, *in_body.clone()));
                    }
                }

                let fv = self.evaluate(env, func)?;
                let av = self.evaluate(env, arg)?;
                match fv {
                    Value::Closure(param, body, mut closure_env) => {
                        closure_env.insert(param, av);
                        Ok(Trampoline::TailCall(closure_env, body))
                    }
                    Value::RecClosure(fn_name, param, body, mut closure_env) => {
                        let self_ref = Value::RecClosure(fn_name.clone(), param.clone(), body.clone(), closure_env.clone());
                        closure_env.insert(fn_name, self_ref);
                        closure_env.insert(param, av);
                        Ok(Trampoline::TailCall(closure_env, body))
                    }
                    _ => Err("VM Error: Attempted to call a non-function".to_string())
                }
            }

            IrNode::MkPair(l, r) => {
                let lv = self.evaluate(env, l)?;
                let rv = self.evaluate(env, r)?;
                Ok(Trampoline::Done(Value::Pair(Box::new(lv), Box::new(rv))))
            }

            IrNode::Left(p) => {
                if let Value::Pair(l, _) = self.evaluate(env, p)? {
                    Ok(Trampoline::Done(*l))
                } else {
                    Err("VM Error: Expected a Pair for 'Left'".to_string())
                }
            }

            IrNode::Right(p) => {
                if let Value::Pair(_, r) = self.evaluate(env, p)? {
                    Ok(Trampoline::Done(*r))
                } else {
                    Err("VM Error: Expected a Pair for 'Right'".to_string())
                }
            }

            IrNode::Unpack(v1, v2, p, body) => {
                if let Value::Pair(l, r) = self.evaluate(env, p)? {
                    let mut local_env = env.clone();
                    local_env.insert(v1.clone(), *l);
                    local_env.insert(v2.clone(), *r);
                    
                    Ok(Trampoline::TailCall(local_env, *body.clone()))
                } else {
                    Err("VM Error: Expected a Pair for 'Unpack'".to_string())
                }
            }

            IrNode::Match(expr, l_var, l_body, r_var, r_body) => {
                let match_val = self.evaluate(env, expr)?;
                if let Value::Int(v) = match_val {
                    if v != 0 {
                        let mut local_env = env.clone();
                        local_env.insert(l_var.clone(), Value::Int(v));
                        Ok(Trampoline::TailCall(local_env, *l_body.clone()))
                    } else {
                        let mut local_env = env.clone();
                        local_env.insert(r_var.clone(), Value::Int(v));
                        Ok(Trampoline::TailCall(local_env, *r_body.clone()))
                    }
                } else {
                    Err("VM Error: Match expression must evaluate to an Integer".to_string())
                }
            }
        }
    }
}