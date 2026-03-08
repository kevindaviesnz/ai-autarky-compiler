use std::collections::HashMap;
use crate::ast::{Permission, Resource, Term, Type};

#[derive(Debug, Clone)]
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
            Term::UnitVal => Ok(Type::Unit), // NEW: Handled here
            Term::Add(t1, t2) => {
                let type1 = self.check(t1)?;
                let type2 = self.check(t2)?;
                
                if type1 == Type::Int && type2 == Type::Int {
                    Ok(Type::Int)
                } else {
                    Err("Type Error: Both operands of addition must be Int".to_string())
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
            Term::Free(target) => { // NEW: Handled here
                let target_type = self.check(target)?;
                match target_type {
                    Type::Linear(Permission::Full, _) => {
                        // The prover has consumed the full permission resource and verified it.
                        // We safely return Unit.
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