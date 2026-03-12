// src/vm.rs

use std::collections::HashMap;
use crate::ir::IrNode;

#[derive(Debug, Clone)]
pub enum Value {
    Int(i64),
    Pair(Box<Value>, Box<Value>),
    Array(Vec<Value>),
    Closure(String, IrNode, HashMap<String, Value>),
    // Special variant to allow recursive function calls in an immutable environment
    RecClosure(String, String, IrNode, HashMap<String, Value>), 
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
        match node {
            IrNode::Int(n) => Ok(Value::Int(*n)),
            
            IrNode::Var(name) => env
                .get(name)
                .cloned()
                .ok_or_else(|| format!("VM Error: Undefined variable '{}'", name)),
            
            IrNode::Add(l, r) => {
                let lv = self.evaluate(env, l)?;
                let rv = self.evaluate(env, r)?;
                match (lv, rv) {
                    (Value::Int(a), Value::Int(b)) => Ok(Value::Int(a + b)),
                    _ => Err("VM Error: Type mismatch in Addition".to_string()),
                }
            }

            IrNode::Sub(l, r) => {
                let lv = self.evaluate(env, l)?;
                let rv = self.evaluate(env, r)?;
                match (lv, rv) {
                    (Value::Int(a), Value::Int(b)) => Ok(Value::Int(a - b)),
                    _ => Err("VM Error: Type mismatch in Subtraction".to_string()),
                }
            }

            IrNode::Eq(l, r) => {
                let lv = self.evaluate(env, l)?;
                let rv = self.evaluate(env, r)?;
                match (lv, rv) {
                    (Value::Int(a), Value::Int(b)) => {
                        Ok(Value::Int(if a == b { 1 } else { 0 }))
                    },
                    _ => Err("VM Error: Type mismatch in Equality".to_string()),
                }
            }

            IrNode::Lam(param, body) => {
                Ok(Value::Closure(param.clone(), *body.clone(), env.clone()))
            }

            IrNode::App(func, arg) => {
                // INTERCEPT: Recursive Let Binding
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
                        return self.evaluate(&local_env, in_body);
                    }
                }

                // Standard Application
                let fv = self.evaluate(env, func)?;
                let av = self.evaluate(env, arg)?;
                match fv {
                    Value::Closure(param, body, mut closure_env) => {
                        closure_env.insert(param, av);
                        self.evaluate(&closure_env, &body)
                    }
                    Value::RecClosure(fn_name, param, body, mut closure_env) => {
                        // Inject itself into the environment right before execution
                        let self_ref = Value::RecClosure(fn_name.clone(), param.clone(), body.clone(), closure_env.clone());
                        closure_env.insert(fn_name, self_ref);
                        closure_env.insert(param, av);
                        self.evaluate(&closure_env, &body)
                    }
                    _ => Err("VM Error: Attempted to call a non-function".to_string())
                }
            }

            IrNode::MkPair(l, r) => {
                let lv = self.evaluate(env, l)?;
                let rv = self.evaluate(env, r)?;
                Ok(Value::Pair(Box::new(lv), Box::new(rv)))
            }

            IrNode::Left(p) => {
                if let Value::Pair(l, _) = self.evaluate(env, p)? {
                    Ok(*l)
                } else {
                    Err("VM Error: Expected a Pair for 'Left'".to_string())
                }
            }

            IrNode::Right(p) => {
                if let Value::Pair(_, r) = self.evaluate(env, p)? {
                    Ok(*r)
                } else {
                    Err("VM Error: Expected a Pair for 'Right'".to_string())
                }
            }

            IrNode::Unpack(v1, v2, p, body) => {
                if let Value::Pair(l, r) = self.evaluate(env, p)? {
                    let mut local_env = env.clone();
                    local_env.insert(v1.clone(), *l);
                    local_env.insert(v2.clone(), *r);
                    self.evaluate(&local_env, body)
                } else {
                    Err("VM Error: Expected a Pair for 'Unpack'".to_string())
                }
            }

            IrNode::ArrayAlloc(sz, init) => {
                let size = if let Value::Int(s) = self.evaluate(env, sz)? {
                    s as usize
                } else {
                    return Err("VM Error: Array size must be an Integer".to_string());
                };
                let init_val = self.evaluate(env, init)?;
                Ok(Value::Array(vec![init_val; size]))
            }

            IrNode::ArraySwap(arr, idx, val) => {
                let mut av = self.evaluate(env, arr)?;
                let i = if let Value::Int(index) = self.evaluate(env, idx)? {
                    index as usize
                } else {
                    return Err("VM Error: Array index must be an Integer".to_string());
                };
                let new_val = self.evaluate(env, val)?;

                if let Value::Array(ref mut elements) = av {
                    let old_val = elements[i].clone();
                    elements[i] = new_val;
                    Ok(Value::Pair(Box::new(old_val), Box::new(av.clone())))
                } else {
                    Err("VM Error: Expected an Array for 'Swap'".to_string())
                }
            }

            IrNode::Match(expr, l_var, l_body, r_var, r_body) => {
                let match_val = self.evaluate(env, expr)?;
                if let Value::Int(v) = match_val {
                    if v != 0 {
                        // Non-zero equates to True (Left branch)
                        let mut local_env = env.clone();
                        local_env.insert(l_var.clone(), Value::Int(v));
                        self.evaluate(&local_env, l_body)
                    } else {
                        // Zero equates to False (Right branch)
                        let mut local_env = env.clone();
                        local_env.insert(r_var.clone(), Value::Int(v));
                        self.evaluate(&local_env, r_body)
                    }
                } else {
                    Err("VM Error: Match expression must evaluate to an Integer".to_string())
                }
            }
        }
    }
}