// src/typecheck.rs

use std::collections::HashMap;
use crate::ast::{Expr, Type};

/// The Scope Janitor. 
/// It mathematically proves that every variable is used exactly once.
pub struct TypeChecker;

impl TypeChecker {
    pub fn new() -> Self {
        TypeChecker
    }

    /// Type-checks an expression against an environment of available linear resources.
    /// It returns the calculated Type, and the *remaining* environment (resources not yet consumed).
    pub fn check(&self, env: HashMap<String, Type>, expr: &Expr) -> Result<(Type, HashMap<String, Type>), String> {
        match expr {
            // 1. Core Language
            Expr::IntLiteral(_) => Ok((Type::Int, env)),
            
            Expr::Variable(name) => {
                let mut new_env = env.clone();
                // CONSUME THE VARIABLE: If it exists, we take it out of the environment forever.
                match new_env.remove(name) {
                    Some(ty) => Ok((ty, new_env)),
                    None => Err(format!("Scope Janitor Panic: Variable '{}' is unbound, already consumed, or leaked!", name)),
                }
            }

            Expr::Lambda { param, param_type, body } => {
                let mut body_env = env.clone();
                body_env.insert(param.clone(), param_type.clone());
                
                let (body_type, mut out_env) = self.check(body_env, body)?;
                
                // Ensure the parameter was actually consumed inside the body
                if out_env.contains_key(param) {
                    return Err(format!("Scope Janitor Panic: Parameter '{}' was never used!", param));
                }
                out_env.remove(param); // Clean up just in case
                
                Ok((Type::Func(Box::new(param_type.clone()), Box::new(body_type)), out_env))
            }

            Expr::App { func, arg } => {
                let (func_ty, env_after_func) = self.check(env, func)?;
                let (arg_ty, env_after_arg) = self.check(env_after_func, arg)?;
                
                match func_ty {
                    Type::Func(param_ty, ret_ty) => {
                        if *param_ty == arg_ty {
                            Ok((*ret_ty, env_after_arg))
                        } else {
                            Err("Type Error: Argument type does not match function parameter type.".to_string())
                        }
                    },
                    _ => Err("Type Error: Tried to apply an argument to a non-function.".to_string()),
                }
            }

            // 2. Math
            Expr::Add(left, right) | Expr::Sub(left, right) => {
                let (l_ty, env_after_l) = self.check(env, left)?;
                let (r_ty, env_after_r) = self.check(env_after_l, right)?;
                
                if l_ty == Type::Int && r_ty == Type::Int {
                    Ok((Type::Int, env_after_r))
                } else {
                    Err("Type Error: Math operations require Ints.".to_string())
                }
            }

            // 3. Pairs
            Expr::MkPair(left, right) => {
                let (l_ty, env_after_l) = self.check(env, left)?;
                let (r_ty, env_after_r) = self.check(env_after_l, right)?;
                Ok((Type::Pair(Box::new(l_ty), Box::new(r_ty)), env_after_r))
            }

            Expr::Unpack { pair, var1, var2, body } => {
                let (pair_ty, mut env_after_pair) = self.check(env, pair)?;
                
                match pair_ty {
                    Type::Pair(t1, t2) => {
                        env_after_pair.insert(var1.clone(), *t1);
                        env_after_pair.insert(var2.clone(), *t2);
                        
                        let (body_ty, mut out_env) = self.check(env_after_pair, body)?;
                        
                        if out_env.contains_key(var1) || out_env.contains_key(var2) {
                            return Err(format!("Scope Janitor Panic: Unpacked variables '{}' and '{}' must both be consumed!", var1, var2));
                        }
                        out_env.remove(var1);
                        out_env.remove(var2);
                        
                        Ok((body_ty, out_env))
                    },
                    _ => Err("Type Error: Cannot unpack a non-Pair.".to_string()),
                }
            }

            // 4. Branching (Sum Types)
            Expr::Left(expr, right_ty) => {
                let (l_ty, out_env) = self.check(env, expr)?;
                Ok((Type::Either(Box::new(l_ty), Box::new(right_ty.clone())), out_env))
            }

            Expr::Right(expr, left_ty) => {
                let (r_ty, out_env) = self.check(env, expr)?;
                Ok((Type::Either(Box::new(left_ty.clone()), Box::new(r_ty)), out_env))
            }

            Expr::Match { expr, left_var, left_body, right_var, right_body } => {
                let (expr_ty, env_after_expr) = self.check(env, expr)?;
                
                match expr_ty {
                    Type::Either(l_ty, r_ty) => {
                        let mut l_env = env_after_expr.clone();
                        l_env.insert(left_var.clone(), *l_ty);
                        let (l_body_ty, mut l_out_env) = self.check(l_env, left_body)?;
                        l_out_env.remove(left_var);

                        let mut r_env = env_after_expr.clone();
                        r_env.insert(right_var.clone(), *r_ty);
                        let (r_body_ty, mut r_out_env) = self.check(r_env, right_body)?;
                        r_out_env.remove(right_var);

                        if l_body_ty != r_body_ty {
                            return Err("Type Error: Match branches must return the exact same type.".to_string());
                        }

                        // BRANCH EQUIVALENCE RULE: Both timelines must have consumed the exact same outer variables!
                        if l_out_env != r_out_env {
                            return Err("Scope Janitor Panic: Match branches consumed different linear variables! Timelines are unbalanced.".to_string());
                        }

                        Ok((l_body_ty, l_out_env))
                    },
                    _ => Err("Type Error: Can only match on an Either type.".to_string()),
                }
            }

            // 5. System Primitives: Arrays
            Expr::ArrayAlloc { size, init_val } => {
                let (size_ty, env_after_size) = self.check(env, size)?;
                let (init_ty, env_after_init) = self.check(env_after_size, init_val)?;
                
                if size_ty != Type::Int { return Err("Type Error: Array size must be an Int.".to_string()); }
                if init_ty != Type::Int { return Err("Scope Janitor Panic: Arrays can only be initialized with inert Ints to prevent duplicating linear assets!".to_string()); }
                
                Ok((Type::Array(Box::new(Type::Int)), env_after_init))
            }

            Expr::ArraySwap { array, index, new_val } => {
                let (arr_ty, env_after_arr) = self.check(env, array)?;
                let (idx_ty, env_after_idx) = self.check(env_after_arr, index)?;
                let (val_ty, env_after_val) = self.check(env_after_idx, new_val)?;

                if idx_ty != Type::Int { return Err("Type Error: Array index must be an Int.".to_string()); }

                match arr_ty {
                    Type::Array(inner_ty) => {
                        if val_ty != *inner_ty {
                            return Err("Type Error: Swapped value does not match array type.".to_string());
                        }
                        // Returns: Pair(OldValue, NewArray)
                        let return_ty = Type::Pair(inner_ty.clone(), Box::new(Type::Array(inner_ty)));
                        Ok((return_ty, env_after_val))
                    },
                    _ => Err("Type Error: Cannot swap on a non-Array.".to_string()),
                }
            }
        }
    }
}