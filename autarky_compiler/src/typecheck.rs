use std::collections::HashMap;
use crate::ast::{Permission, Resource, Term, Type};

#[derive(Debug, Clone, PartialEq)]
pub struct Context {
    resources: HashMap<String, Resource>,
}

impl Context {
    pub fn new() -> Self { Self { resources: HashMap::new() } }
    pub fn insert(&mut self, name: String, resource: Resource) { self.resources.insert(name, resource); }
    pub fn clone_persistent(&self) -> Self {
        let mut new_ctx = Context::new();
        for (name, resource) in &self.resources {
            if let Resource::Persistent(_) = resource { new_ctx.insert(name.clone(), resource.clone()); }
        }
        new_ctx
    }

    pub fn check(&mut self, term: &Term) -> Result<Type, String> {
        match term {
            Term::IntVal(_) => Ok(Type::Int),
            Term::UnitVal => Ok(Type::Unit),
            Term::BoolVal(_) => Ok(Type::Bool),
            Term::StringVal(_) => Ok(Type::String), 
            Term::ReadFile(path) => { 
                if self.check(path)? != Type::String { return Err("Type Error: read_file requires a String path".to_string()); }
                Ok(Type::Linear(Permission::Full, Box::new(Type::Array(Box::new(Type::Int)))))
            }
            Term::Fold(ty, t) => { 
                let expected_inner = match ty.clone() {
                    Type::Rec(x, inner_ty) => inner_ty.substitute_type(&x, &ty),
                    _ => return Err("Type Error: fold requires a Rec type".to_string()),
                };
                let actual_ty = self.check(t)?;
                if actual_ty != expected_inner { return Err(format!("Fold type mismatch. Expected {:?}, got {:?}", expected_inner, actual_ty)); }
                Ok(ty.clone())
            }
            Term::Unfold(t) => { 
                let t_ty = self.check(t)?;
                // We match on a clone here so the original t_ty isn't partially moved!
                match t_ty.clone() {
                    Type::Rec(x, inner_ty) => Ok(inner_ty.substitute_type(&x, &t_ty)),
                    Type::Linear(perm, inner) => match *inner {
                        Type::Rec(x, rec_inner) => {
                            let unfolded = rec_inner.substitute_type(&x, &Type::Rec(x.clone(), rec_inner.clone()));
                            Ok(Type::Linear(perm, Box::new(unfolded)))
                        }
                        _ => Err("Type Error: Cannot unfold a non-recursive linear resource".to_string()),
                    },
                    _ => Err("Type Error: unfold requires a Rec type".to_string()),
                }
            }
            Term::Alloc(size, init) => {
                if self.check(size)? != Type::Int { return Err("Type Error: Array size must be Int".to_string()); }
                Ok(Type::Linear(Permission::Full, Box::new(Type::Array(Box::new(self.check(init)?)))))
            }
            Term::Read(arr, idx) => {
                if self.check(idx)? != Type::Int { return Err("Type Error: Array index must be Int".to_string()); }
                match self.check(arr)? {
                    Type::Linear(perm, inner) => match *inner {
                        Type::Array(elem_ty) => Ok(Type::Pair(elem_ty.clone(), Box::new(Type::Linear(perm, Box::new(Type::Array(elem_ty)))))),
                        _ => Err("Type Error: Can only read from a Linear Array".to_string()),
                    },
                    _ => Err("Type Error: Can only read from a Linear Array".to_string()),
                }
            }
            Term::Write(arr, idx, val) => {
                if self.check(idx)? != Type::Int { return Err("Type Error: Array index must be Int".to_string()); }
                let val_ty = self.check(val)?;
                match self.check(arr)? {
                    Type::Linear(Permission::Full, inner) => match *inner {
                        Type::Array(elem_ty) => {
                            if *elem_ty != val_ty { return Err(format!("Type Error: Element type mismatch")); }
                            Ok(Type::Linear(Permission::Full, Box::new(Type::Array(elem_ty))))
                        }
                        _ => Err("Type Error: Can only write to a Linear Array".to_string()),
                    },
                    Type::Linear(Permission::Fraction(_, _), _) => Err("Type Error: Cannot write to a fractional read-only array".to_string()),
                    _ => Err("Type Error: Can only write to a Linear Array".to_string()),
                }
            }
            Term::Left(t, right_ty) => Ok(Type::Either(Box::new(self.check(t)?), Box::new(right_ty.clone()))),
            Term::Right(left_ty, t) => Ok(Type::Either(Box::new(left_ty.clone()), Box::new(self.check(t)?))),
            Term::Match(target, id_l, body_l, id_r, body_r) => {
                let (t_l, t_r) = match self.check(target)? {
                    Type::Either(t1, t2) => (*t1, *t2),
                    Type::Linear(Permission::Full, inner) => match *inner { Type::Either(t1, t2) => (Type::Linear(Permission::Full, t1), Type::Linear(Permission::Full, t2)), _ => return Err("Type Error: Cannot match a non-Either linear resource".to_string()) },
                    _ => return Err("Type Error: 'match' target must be an Either type".to_string()),
                };
                let mut ctx_l = self.clone(); let mut ctx_r = self.clone();
                ctx_l.insert(id_l.clone(), match &t_l { Type::Linear(_, _) => Resource::Linear(t_l.clone()), _ => Resource::Persistent(t_l.clone()) });
                ctx_r.insert(id_r.clone(), match &t_r { Type::Linear(_, _) => Resource::Linear(t_r.clone()), _ => Resource::Persistent(t_r.clone()) });
                let type_l = ctx_l.check(body_l)?; let type_r = ctx_r.check(body_r)?;
                if type_l != type_r { return Err("Type Error: 'match' branches return diff types".to_string()); }
                ctx_l.resources.remove(id_l); ctx_r.resources.remove(id_r);
                if ctx_l.resources != ctx_r.resources { return Err("Linearity Violation: match branches differ in resources".to_string()); }
                self.resources = ctx_l.resources; Ok(type_l)
            }
            Term::MkPair(t1, t2) => Ok(Type::Pair(Box::new(self.check(t1)?), Box::new(self.check(t2)?))),
            Term::Unpack(target, alias1, alias2, body) => {
                let (t_x, t_y) = match self.check(target)? {
                    Type::Pair(t1, t2) => (*t1, *t2),
                    Type::Linear(Permission::Full, inner) => match *inner { Type::Pair(t1, t2) => (Type::Linear(Permission::Full, t1), Type::Linear(Permission::Full, t2)), _ => return Err("Type Error: Cannot unpack a non-pair linear resource".to_string()) },
                    Type::Linear(Permission::Fraction(n, d), inner) => match *inner { Type::Pair(t1, t2) => (Type::Linear(Permission::Fraction(n, d), t1), Type::Linear(Permission::Fraction(n, d), t2)), _ => return Err("Type Error: Cannot unpack a non-pair fractional resource".to_string()) },
                    _ => return Err("Type Error: Can only unpack a Pair".to_string()),
                };
                let mut body_ctx = self.clone();
                body_ctx.insert(alias1.clone(), match &t_x { Type::Linear(_, _) => Resource::Linear(t_x.clone()), _ => Resource::Persistent(t_x.clone()) });
                body_ctx.insert(alias2.clone(), match &t_y { Type::Linear(_, _) => Resource::Linear(t_y.clone()), _ => Resource::Persistent(t_y.clone()) });
                let result_type = body_ctx.check(body)?;
                for (name, res) in body_ctx.resources.iter() { if let Resource::Linear(_) = res { if name == alias1 || name == alias2 { return Err(format!("Linearity Violation: '{}' never consumed", name)); } } }
                body_ctx.resources.remove(alias1); body_ctx.resources.remove(alias2); self.resources = body_ctx.resources; Ok(result_type)
            }
            Term::If(cond, t_branch, f_branch) => {
                if self.check(cond)? != Type::Bool { return Err("Type Error: Condition of 'if' must be Bool".to_string()); }
                let mut ctx_true = self.clone(); let mut ctx_false = self.clone();
                let t_type = ctx_true.check(t_branch)?; let f_type = ctx_false.check(f_branch)?;
                if t_type != f_type { return Err("Type Error: Branches of 'if' diff types".to_string()); }
                if ctx_true.resources != ctx_false.resources { return Err("Linearity Violation: 'if' branches differ".to_string()); }
                self.resources = ctx_true.resources; Ok(t_type)
            }
            Term::Add(t1, t2) | Term::Sub(t1, t2) => { if self.check(t1)? == Type::Int && self.check(t2)? == Type::Int { Ok(Type::Int) } else { Err("Type Error: Int required".to_string()) } }
            Term::Eq(t1, t2) => { if self.check(t1)? == Type::Int && self.check(t2)? == Type::Int { Ok(Type::Bool) } else { Err("Type Error: Int required for Eq".to_string()) } }
            Term::Fix(inner) => { match self.check(inner)? { Type::Pi(_, t1, t2) if t1 == t2 => Ok(*t1), _ => Err("Type Error: 'fix' must be applied to a function of type A -> A".to_string()) } }
            Term::Var(name) => {
                match self.resources.get(name).cloned() {
                    Some(Resource::Persistent(t)) => Ok(t),
                    Some(Resource::Linear(t)) => { self.resources.remove(name); Ok(t) }
                    None => Err(format!("Linearity Violation: Unbound variable '{}'", name)),
                }
            }
            Term::Free(target) => { match self.check(target)? { Type::Linear(Permission::Full, _) => Ok(Type::Unit), Type::Linear(Permission::Fraction(_, _), _) => Err("Type Error: Cannot free a fractional permission.".to_string()), _ => Err("Type Error: Can only free a Linear resource.".to_string()) } }
            Term::Split(target, alias1, alias2, body) => {
                let inner_type = match self.resources.remove(target).ok_or_else(|| format!("Violation: '{}' unbound", target))? { Resource::Linear(Type::Linear(Permission::Full, t)) => t, _ => return Err("Type Error: Must be Full permission to split.".to_string()) };
                let mut body_ctx = self.clone();
                body_ctx.insert(alias1.clone(), Resource::Linear(Type::Linear(Permission::Fraction(1, 2), inner_type.clone())));
                body_ctx.insert(alias2.clone(), Resource::Linear(Type::Linear(Permission::Fraction(1, 2), inner_type.clone())));
                let result_type = body_ctx.check(body)?;
                for (name, res) in body_ctx.resources { if let Resource::Linear(_) = res { return Err(format!("Violation: '{}' never consumed", name)); } }
                Ok(result_type)
            }
            Term::Merge(alias1, alias2, target, body) => {
                let res1 = self.resources.remove(alias1).ok_or_else(|| format!("Violation: '{}' unbound", alias1))?;
                let res2 = self.resources.remove(alias2).ok_or_else(|| format!("Violation: '{}' unbound", alias2))?;
                let inner_type = match (res1, res2) {
                    (Resource::Linear(Type::Linear(Permission::Fraction(n1, d1), t1)), Resource::Linear(Type::Linear(Permission::Fraction(n2, d2), t2))) => {
                        if t1 != t2 { return Err("Type Error: Merge types differ".to_string()); }
                        if (n1 * d2) + (n2 * d1) != (d1 * d2) { return Err("Type Error: Fractions do not sum to Full".to_string()); }
                        *t1
                    }
                    _ => return Err("Type Error: Can only merge linear fractional permissions".to_string()),
                };
                let mut body_ctx = self.clone();
                body_ctx.insert(target.clone(), Resource::Linear(Type::Linear(Permission::Full, Box::new(inner_type))));
                let result_type = body_ctx.check(body)?;
                for (name, res) in body_ctx.resources { if let Resource::Linear(_) = res { return Err(format!("Violation: '{}' never consumed", name)); } }
                Ok(result_type)
            }
            Term::Abs(param_name, param_type, body) => {
                let mut body_ctx = self.clone_persistent();
                body_ctx.insert(param_name.clone(), match param_type { Type::Linear(_, _) => Resource::Linear(param_type.clone()), _ => Resource::Persistent(param_type.clone()) });
                let body_type = body_ctx.check(body)?;
                for (name, res) in body_ctx.resources { if let Resource::Linear(_) = res { return Err(format!("Violation: parameter '{}' never consumed", name)); } }
                Ok(Type::Pi(param_name.clone(), Box::new(param_type.clone()), Box::new(body_type)))
            }
            Term::App(t1, t2) => {
                let type1 = self.check(t1)?;
                let _type2 = self.check(t2)?;
                match type1 { Type::Pi(param_name, _, return_type) => Ok(return_type.substitute(&param_name, t2)), _ => Err("Type Error: Attempted to apply to a non-function term".to_string()) }
            }
        }
    }
}