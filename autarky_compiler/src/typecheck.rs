use std::collections::HashMap;
use crate::ast::{Permission, Resource, Term, Type};

#[derive(Debug, Clone, PartialEq)] 
pub struct Context {
    resources: HashMap<String, Resource>,
}

impl Context {
    pub fn new() -> Self {
        Self { resources: HashMap::new() }
    }

    pub fn insert(&mut self, name: String, resource: Resource) {
        self.resources.insert(name, resource);
    }

    pub fn clone_persistent(&self) -> Self {
        let mut new_ctx = Context::new();
        for (name, resource) in &self.resources {
            if let Resource::Persistent(_) = resource {
                new_ctx.insert(name.clone(), resource.clone());
            }
        }
        new_ctx
    }

    pub fn check(&mut self, term: &Term) -> Result<Type, String> {
        match term {
            Term::IntVal(_) => Ok(Type::Int),
            Term::UnitVal => Ok(Type::Unit),
            Term::BoolVal(_) => Ok(Type::Bool), 
            Term::MkPair(t1, t2) => {
                let type1 = self.check(t1)?;
                let type2 = self.check(t2)?;
                Ok(Type::Pair(Box::new(type1), Box::new(type2)))
            }
            Term::Unpack(target, alias1, alias2, body) => {
                let target_type = self.check(target)?;
                
                let (t_x, t_y) = match target_type {
                    Type::Pair(t1, t2) => (*t1, *t2),
                    Type::Linear(Permission::Full, inner) => match *inner {
                        Type::Pair(t1, t2) => (
                            Type::Linear(Permission::Full, t1),
                            Type::Linear(Permission::Full, t2)
                        ),
                        _ => return Err("Type Error: Cannot unpack a non-pair linear resource".to_string())
                    },
                    Type::Linear(Permission::Fraction(n, d), inner) => match *inner {
                        Type::Pair(t1, t2) => (
                            Type::Linear(Permission::Fraction(n, d), t1),
                            Type::Linear(Permission::Fraction(n, d), t2)
                        ),
                        _ => return Err("Type Error: Cannot unpack a non-pair fractional resource".to_string())
                    },
                    _ => return Err("Type Error: Can only unpack a Pair".to_string()),
                };

                let mut body_ctx = self.clone();
                let res_x = match &t_x {
                    Type::Linear(_, _) => Resource::Linear(t_x.clone()),
                    _ => Resource::Persistent(t_x.clone()),
                };
                let res_y = match &t_y {
                    Type::Linear(_, _) => Resource::Linear(t_y.clone()),
                    _ => Resource::Persistent(t_y.clone()),
                };

                body_ctx.insert(alias1.clone(), res_x);
                body_ctx.insert(alias2.clone(), res_y);

                let result_type = body_ctx.check(body)?;

                for (name, res) in body_ctx.resources.iter() {
                    if let Resource::Linear(_) = res {
                        if name == alias1 || name == alias2 {
                            return Err(format!("Linearity Violation: Unpacked variable '{}' was never consumed", name));
                        }
                    }
                }

                body_ctx.resources.remove(alias1);
                body_ctx.resources.remove(alias2);
                self.resources = body_ctx.resources;

                Ok(result_type)
            }
            Term::If(cond, t_branch, f_branch) => { 
                let cond_type = self.check(cond)?;
                if cond_type != Type::Bool {
                    return Err("Type Error: Condition of 'if' must be a Bool".to_string());
                }

                let mut ctx_true = self.clone();
                let mut ctx_false = self.clone();

                let t_type = ctx_true.check(t_branch)?;
                let f_type = ctx_false.check(f_branch)?;

                if t_type != f_type {
                    return Err(format!("Type Error: Branches of 'if' return different types ({:?} vs {:?})", t_type, f_type));
                }

                if ctx_true.resources != ctx_false.resources {
                    return Err("Linearity Violation: Both branches of an 'if' must consume the exact same linear resources!".to_string());
                }

                self.resources = ctx_true.resources;
                Ok(t_type)
            }
            Term::Add(t1, t2) => {
                let type1 = self.check(t1)?;
                let type2 = self.check(t2)?;
                if type1 == Type::Int && type2 == Type::Int { Ok(Type::Int) } 
                else { Err("Type Error: Operands of addition must be Int".to_string()) }
            }
            Term::Sub(t1, t2) => { // NEW
                let type1 = self.check(t1)?;
                let type2 = self.check(t2)?;
                if type1 == Type::Int && type2 == Type::Int { Ok(Type::Int) } 
                else { Err("Type Error: Operands of subtraction must be Int".to_string()) }
            }
            Term::Eq(t1, t2) => { // NEW
                let type1 = self.check(t1)?;
                let type2 = self.check(t2)?;
                if type1 == Type::Int && type2 == Type::Int { Ok(Type::Bool) } 
                else { Err("Type Error: Operands of == must be Int".to_string()) }
            }
            Term::Fix(inner) => { // NEW
                let inner_type = self.check(inner)?;
                match inner_type {
                    // Verifies it is exactly A -> A
                    Type::Pi(_, t1, t2) if t1 == t2 => Ok(*t1),
                    _ => Err("Type Error: 'fix' must be applied to a function of type A -> A".to_string()),
                }
            }
            Term::Var(name) => {
                let resource = self.resources.get(name).cloned();
                match resource {
                    Some(Resource::Persistent(t)) => Ok(t),
                    Some(Resource::Linear(t)) => {
                        self.resources.remove(name);
                        Ok(t)
                    }
                    None => Err(format!("Linearity Violation: Unbound or already consumed variable '{}'", name)),
                }
            }
            Term::Free(target) => {
                let target_type = self.check(target)?;
                match target_type {
                    Type::Linear(Permission::Full, _) => {
                        Ok(Type::Unit)
                    }
                    Type::Linear(Permission::Fraction(_, _), _) => {
                        Err("Type Error: Cannot free a fractional permission. You must merge to Full first.".to_string())
                    }
                    _ => Err("Type Error: Can only free a Linear resource.".to_string()),
                }
            }
            Term::Split(target, alias1, alias2, body) => {
                let resource = self.resources.remove(target)
                    .ok_or_else(|| format!("Linearity Violation: Cannot split unbound variable '{}'", target))?;

                let inner_type = match resource {
                    Resource::Linear(Type::Linear(Permission::Full, t)) => t,
                    _ => return Err(format!("Type Error: Can only split a Linear resource with Full permission. '{}' does not qualify.", target)),
                };

                let half_perm = Permission::Fraction(1, 2);
                let alias_type = Type::Linear(half_perm.clone(), inner_type.clone());

                let mut body_ctx = self.clone();
                body_ctx.insert(alias1.clone(), Resource::Linear(alias_type.clone()));
                body_ctx.insert(alias2.clone(), Resource::Linear(alias_type.clone()));

                let result_type = body_ctx.check(body)?;

                for (name, res) in body_ctx.resources {
                    if let Resource::Linear(_) = res {
                        return Err(format!("Linearity Violation: Fractional alias '{}' was never consumed inside the split body", name));
                    }
                }

                Ok(result_type)
            }
            Term::Merge(alias1, alias2, target, body) => {
                let res1 = self.resources.remove(alias1)
                    .ok_or_else(|| format!("Linearity Violation: '{}' not found", alias1))?;
                let res2 = self.resources.remove(alias2)
                    .ok_or_else(|| format!("Linearity Violation: '{}' not found", alias2))?;

                let inner_type = match (res1, res2) {
                    (Resource::Linear(Type::Linear(Permission::Fraction(n1, d1), t1)),
                     Resource::Linear(Type::Linear(Permission::Fraction(n2, d2), t2))) => {
                        if t1 != t2 { 
                            return Err("Type Error: Cannot merge fractions of different types".to_string()); 
                        }
                        if (n1 * d2) + (n2 * d1) != (d1 * d2) {
                            return Err("Type Error: Fractions do not sum to Full permission".to_string());
                        }
                        *t1 
                    },
                    _ => return Err("Type Error: Can only merge linear fractional permissions".to_string()),
                };

                let mut body_ctx = self.clone();
                body_ctx.insert(target.clone(), Resource::Linear(Type::Linear(Permission::Full, Box::new(inner_type))));
                
                let result_type = body_ctx.check(body)?;
                
                for (name, res) in body_ctx.resources {
                    if let Resource::Linear(_) = res {
                        return Err(format!("Linearity Violation: Merged variable '{}' was never consumed", name));
                    }
                }
                Ok(result_type)
            }
            Term::Abs(param_name, param_type, body) => {
                let mut body_ctx = self.clone_persistent();
                
                let resource_type = match param_type {
                    Type::Linear(_, _) => Resource::Linear(param_type.clone()),
                    _ => Resource::Persistent(param_type.clone()),
                };
                body_ctx.insert(param_name.clone(), resource_type);
                
                let body_type = body_ctx.check(body)?;

                for (name, res) in body_ctx.resources {
                    if let Resource::Linear(_) = res {
                        return Err(format!("Linearity Violation: Linear parameter '{}' was never consumed", name));
                    }
                }

                Ok(Type::Pi(param_name.clone(), Box::new(param_type.clone()), Box::new(body_type)))
            }
            Term::App(t1, t2) => {
                let type1 = self.check(t1)?;
                let _type2 = self.check(t2)?;

                match type1 {
                    Type::Pi(param_name, _expected, return_type) => Ok(return_type.substitute(&param_name, t2)),
                    _ => Err("Type Error: Attempted to apply to a non-function term".to_string()),
                }
            }
        }
    }
}