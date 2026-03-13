// src/typecheck.rs

use std::collections::HashMap;
use crate::ast::{Expr, Type};

pub struct TypeChecker {}

impl TypeChecker {
    pub fn new() -> Self {
        Self {}
    }

    pub fn check(
        &self,
        env: HashMap<String, Type>,
        expr: &Expr,
    ) -> Result<(Type, HashMap<String, Type>), String> {
        match expr {
            Expr::IntLiteral(_) => Ok((Type::Int, env)),
            Expr::FloatLiteral(_) => Ok((Type::Float, env)),
            
            Expr::Variable(name) => {
                let t = env.get(name).cloned().ok_or_else(|| format!("Type Error: Undefined '{}'", name))?;
                Ok((t, env))
            }

            // NEW: All math operations share the same strict typing rules
            Expr::Add(l, r) | Expr::Sub(l, r) | Expr::Mul(l, r) | Expr::Div(l, r) => {
                let (lt, env1) = self.check(env, l)?;
                let (rt, env2) = self.check(env1, r)?;
                match (lt, rt) {
                    (Type::Int, Type::Int) => Ok((Type::Int, env2)),
                    (Type::Float, Type::Float) => Ok((Type::Float, env2)),
                    (l_type, r_type) => Err(format!("Type Error: Arithmetic requires matching types. Got {:?} and {:?}", l_type, r_type)),
                }
            }

            Expr::Eq { left, right } => {
                let (lt, env1) = self.check(env, left)?;
                let (rt, env2) = self.check(env1, right)?;
                match (lt.clone(), rt.clone()) {
                    (Type::Int, Type::Int) | (Type::Float, Type::Float) => {
                        Ok((Type::Either(Box::new(lt), Box::new(rt)), env2))
                    },
                    _ => Err("Type Error: eq requires matching types".to_string())
                }
            }

            Expr::Left(expr, right_type) => {
                let (lt, env1) = self.check(env, expr)?;
                Ok((Type::Either(Box::new(lt), Box::new(right_type.clone())), env1))
            }

            Expr::Right(expr, left_type) => {
                let (rt, env1) = self.check(env, expr)?;
                Ok((Type::Either(Box::new(left_type.clone()), Box::new(rt)), env1))
            }

            Expr::Match { expr, left_var, left_body, right_var, right_body } => {
                let (match_type, env1) = self.check(env, expr)?;
                if let Type::Either(left_t, right_t) = match_type {
                    let mut left_env = env1.clone();
                    left_env.insert(left_var.clone(), *left_t);
                    let (left_res_type, _) = self.check(left_env, left_body)?;

                    let mut right_env = env1.clone();
                    right_env.insert(right_var.clone(), *right_t);
                    let (right_res_type, _) = self.check(right_env, right_body)?;

                    if left_res_type == right_res_type {
                        Ok((left_res_type, env1))
                    } else {
                        Err(format!("Type Error: Match branches differ. Left: {:?}, Right: {:?}", left_res_type, right_res_type))
                    }
                } else {
                    Err("Type Error: Can only match on Either".to_string())
                }
            }

            Expr::Lambda { param, param_type, body } => {
                let mut local_env = env.clone();
                local_env.insert(param.clone(), param_type.clone());
                let (ret_type, _) = self.check(local_env, body)?;
                Ok((Type::Func(Box::new(param_type.clone()), Box::new(ret_type)), env))
            }

            Expr::App { func, arg } => {
                if let Expr::Lambda { param: func_name, body, .. } = &**func {
                    let assumed_func_type = Type::Func(
                        Box::new(Type::Pair(Box::new(Type::Int), Box::new(Type::Int))),
                        Box::new(Type::Int) 
                    );

                    let mut arg_env = env.clone();
                    arg_env.insert(func_name.clone(), assumed_func_type.clone());
                    let (actual_arg_type, _) = self.check(arg_env, arg)?;

                    let mut body_env = env.clone();
                    body_env.insert(func_name.clone(), actual_arg_type);
                    let (ret_type, final_env) = self.check(body_env, body)?;
                    return Ok((ret_type, final_env));
                }

                let (ft, env1) = self.check(env, func)?;
                let (at, env2) = self.check(env1, arg)?;
                
                match ft {
                    Type::Func(p, r) => {
                        if *p == at { Ok((*r, env2)) }
                        else { Err(format!("Type Error: Expected {:?}, got {:?}", p, at)) }
                    }
                    _ => Err(format!("Type Error: Non-function. Got {:?}", ft)),
                }
            }

            Expr::MkPair(l, r) => {
                let (lt, env1) = self.check(env, l)?;
                let (rt, env2) = self.check(env1, r)?;
                Ok((Type::Pair(Box::new(lt), Box::new(rt)), env2))
            }

            Expr::Unpack { pair, var1, var2, body } => {
                let (pt, env1) = self.check(env, pair)?;
                if let Type::Pair(t1, t2) = pt {
                    let mut local_env = env1;
                    local_env.insert(var1.clone(), *t1);
                    local_env.insert(var2.clone(), *t2);
                    self.check(local_env, body)
                } else { 
                    Err("Type Error: Unpack requires Pair".to_string()) 
                }
            }
        }
    }
}