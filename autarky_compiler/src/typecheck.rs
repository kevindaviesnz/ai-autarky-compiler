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
            Term::Split(target, alias1, alias2, body) => {
                // 1. Ensure the target exists and is strictly consumed
                let resource = self.resources.remove(target)
                    .ok_or_else(|| format!("Linearity Violation: Cannot split unbound variable '{}'", target))?;

                // 2. Ensure it has FULL permission before splitting
                let inner_type = match resource {
                    Resource::Linear(Type::Linear(Permission::Full, t)) => t,
                    _ => return Err(format!("Type Error: Can only split a Linear resource with Full permission. '{}' does not qualify.", target)),
                };

                // 3. Create the two 1/2 read-only fractional aliases
                let half_perm = Permission::Fraction(1, 2);
                let alias_type = Type::Linear(half_perm.clone(), inner_type.clone());

                let mut body_ctx = self.clone();
                body_ctx.insert(alias1.clone(), Resource::Linear(alias_type.clone()));
                body_ctx.insert(alias2.clone(), Resource::Linear(alias_type.clone()));

                let result_type = body_ctx.check(body)?;

                // 4. Verify both fractions were safely consumed within the block
                for (name, res) in body_ctx.resources {
                    if let Resource::Linear(_) = res {
                        return Err(format!("Linearity Violation: Fractional alias '{}' was never consumed inside the split body", name));
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