// src/vm.rs

use std::collections::HashMap;
use crate::ir::IrNode;

/// The runtime values that the Virtual Machine actually computes and passes around.
#[derive(Debug, Clone)]
pub enum VmValue {
    Int(i64),
    Pair(Box<VmValue>, Box<VmValue>),
    Left(Box<VmValue>),
    Right(Box<VmValue>),
    Array(Vec<VmValue>),
    // A function that remembers the environment it was created in
    Closure {
        param: String,
        body: IrNode,
        env: HashMap<String, VmValue>,
    },
}

pub struct VirtualMachine;

impl VirtualMachine {
    pub fn new() -> Self {
        VirtualMachine
    }

    /// Evaluates an IR Node down to a final VmValue
    pub fn evaluate(&mut self, env: &HashMap<String, VmValue>, node: &IrNode) -> Result<VmValue, String> {
        match node {
            IrNode::Int(n) => Ok(VmValue::Int(*n)),
            
            IrNode::Var(name) => {
                env.get(name)
                    .cloned()
                    .ok_or_else(|| format!("VM Error: Variable '{}' not found in environment at runtime!", name))
            }

            IrNode::Lam(param, body) => {
                Ok(VmValue::Closure {
                    param: param.clone(),
                    body: *body.clone(),
                    env: env.clone(),
                })
            }

            IrNode::App(func, arg) => {
                let func_val = self.evaluate(env, func)?;
                let arg_val = self.evaluate(env, arg)?;

                match func_val {
                    VmValue::Closure { param, body, mut env } => {
                        env.insert(param, arg_val);
                        self.evaluate(&env, &body)
                    }
                    _ => Err("VM Error: Tried to apply arguments to a non-function.".to_string()),
                }
            }

            IrNode::Add(l, r) => {
                let left = self.evaluate(env, l)?;
                let right = self.evaluate(env, r)?;
                match (left, right) {
                    (VmValue::Int(a), VmValue::Int(b)) => Ok(VmValue::Int(a + b)),
                    _ => Err("VM Error: Math requires Ints.".to_string()),
                }
            }

            IrNode::Sub(l, r) => {
                let left = self.evaluate(env, l)?;
                let right = self.evaluate(env, r)?;
                match (left, right) {
                    (VmValue::Int(a), VmValue::Int(b)) => Ok(VmValue::Int(a - b)),
                    _ => Err("VM Error: Math requires Ints.".to_string()),
                }
            }

            IrNode::MkPair(l, r) => {
                let left = self.evaluate(env, l)?;
                let right = self.evaluate(env, r)?;
                Ok(VmValue::Pair(Box::new(left), Box::new(right)))
            }

            IrNode::Unpack(v1, v2, pair_node, body) => {
                let pair_val = self.evaluate(env, pair_node)?;
                match pair_val {
                    VmValue::Pair(l, r) => {
                        let mut new_env = env.clone();
                        new_env.insert(v1.clone(), *l);
                        new_env.insert(v2.clone(), *r);
                        self.evaluate(&new_env, body)
                    }
                    _ => Err("VM Error: Cannot unpack a non-pair.".to_string()),
                }
            }

            IrNode::Left(val) => {
                let inner = self.evaluate(env, val)?;
                Ok(VmValue::Left(Box::new(inner)))
            }

            IrNode::Right(val) => {
                let inner = self.evaluate(env, val)?;
                Ok(VmValue::Right(Box::new(inner)))
            }

            IrNode::Match(expr, l_var, l_body, r_var, r_body) => {
                let match_val = self.evaluate(env, expr)?;
                match match_val {
                    VmValue::Left(inner) => {
                        let mut new_env = env.clone();
                        new_env.insert(l_var.clone(), *inner);
                        self.evaluate(&new_env, l_body)
                    }
                    VmValue::Right(inner) => {
                        let mut new_env = env.clone();
                        new_env.insert(r_var.clone(), *inner);
                        self.evaluate(&new_env, r_body)
                    }
                    _ => Err("VM Error: Cannot match on a non-Either value.".to_string()),
                }
            }

            IrNode::ArrayAlloc(size_node, init_node) => {
                let size_val = self.evaluate(env, size_node)?;
                let init_val = self.evaluate(env, init_node)?;

                match (size_val, init_val) {
                    (VmValue::Int(size), VmValue::Int(init)) => {
                        if size < 0 {
                            return Err("VM Error: Array size cannot be negative.".to_string());
                        }
                        // Create a contiguous block of memory
                        let vec = vec![VmValue::Int(init); size as usize];
                        Ok(VmValue::Array(vec))
                    }
                    _ => Err("VM Error: ArrayAlloc requires Ints.".to_string()),
                }
            }

            IrNode::ArraySwap(arr_node, idx_node, val_node) => {
                let arr_val = self.evaluate(env, arr_node)?;
                let idx_val = self.evaluate(env, idx_node)?;
                let new_val = self.evaluate(env, val_node)?;

                match (arr_val, idx_val) {
                    (VmValue::Array(mut vec), VmValue::Int(idx)) => {
                        let idx_usize = idx as usize;
                        if idx_usize >= vec.len() {
                            return Err(format!("VM Error: Index {} out of bounds for array of size {}.", idx, vec.len()));
                        }
                        
                        // Perform the hardware-level swap
                        let old_val = std::mem::replace(&mut vec[idx_usize], new_val);
                        
                        // Return the mathematical Pair(OldValue, NewArray)
                        Ok(VmValue::Pair(Box::new(old_val), Box::new(VmValue::Array(vec))))
                    }
                    _ => Err("VM Error: ArraySwap requires an Array and an Int index.".to_string()),
                }
            }
        }
    }
}