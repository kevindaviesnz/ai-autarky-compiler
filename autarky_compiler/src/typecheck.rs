use std::collections::HashMap;
use crate::ast::{Permission, Resource, Term, Type};

#[derive(Debug, Clone, PartialEq)]
pub struct Context { resources: HashMap<String, Resource> }

impl Context {
    pub fn new() -> Self { Self { resources: HashMap::new() } }
    pub fn insert(&mut self, name: String, resource: Resource) { self.resources.insert(name, resource); }
    pub fn clone_persistent(&self) -> Self {
        let mut new_ctx = Context::new();
        for (name, resource) in &self.resources { if let Resource::Persistent(_) = resource { new_ctx.insert(name.clone(), resource.clone()); } }
        new_ctx
    }

    pub fn check(&mut self, term: &Term) -> Result<Type, String> {
        match term {
            Term::IntVal(_) => Ok(Type::Int),
            Term::UnitVal => Ok(Type::Unit),
            Term::BoolVal(_) => Ok(Type::Bool),
            Term::StringVal(_) => Ok(Type::String), // NEW
            Term::ReadFile(path) => { // NEW: Safe filesystem reads
                let path_ty = self.check(path)?;
                if path_ty != Type::String { return Err("Type Error: read_file requires a String path".to_string()); }
                // Returns an exclusive, linear contiguous array of integers (bytes)
                Ok(Type::Linear(Permission::Full, Box::new(Type::Array(Box::new(Type::Int)))))
            }
            Term::Alloc(size, init) => {
                if self.check(size)? != Type::Int { return Err("Error: Array size must be Int".to_string()); }
                Ok(Type::Linear(Permission::Full, Box::new(Type::Array(Box::new(self.check(init)?)))))
            }
            Term::Read(arr, idx) => {
                if self.check(idx)? != Type::Int { return Err("Error: Array idx must be Int".to_string()); }
                match self.check(arr)? {
                    Type::Linear(p, inner) => match *inner {
                        Type::Array(e_ty) => Ok(Type::Pair(e_ty.clone(), Box::new(Type::Linear(p, Box::new(Type::Array(e_ty)))))),
                        _ => Err("Error: Can only read Linear Array".to_string()),
                    },
                    _ => Err("Error".to_string()),
                }
            }
            Term::Write(arr, idx, val) => {
                if self.check(idx)? != Type::Int { return Err("Error: Array idx must be Int".to_string()); }
                let val_ty = self.check(val)?;
                match self.check(arr)? {
                    Type::Linear(Permission::Full, inner) => match *inner {
                        Type::Array(e_ty) => { if *e_ty != val_ty { return Err("Mismatch".to_string()); } Ok(Type::Linear(Permission::Full, Box::new(Type::Array(e_ty)))) }
                        _ => Err("Error".to_string()),
                    },
                    Type::Linear(Permission::Fraction(_, _), _) => Err("Read-only array".to_string()),
                    _ => Err("Error".to_string()),
                }
            }
            Term::Left(t, r_ty) => Ok(Type::Either(Box::new(self.check(t)?), Box::new(r_ty.clone()))),
            Term::Right(l_ty, t) => Ok(Type::Either(Box::new(l_ty.clone()), Box::new(self.check(t)?))),
            Term::Match(tgt, id_l, b_l, id_r, b_r) => {
                let (t_l, t_r) = match self.check(tgt)? {
                    Type::Either(t1, t2) => (*t1, *t2),
                    Type::Linear(Permission::Full, inner) => match *inner { Type::Either(t1, t2) => (Type::Linear(Permission::Full, t1), Type::Linear(Permission::Full, t2)), _ => return Err("Error".to_string()) },
                    _ => return Err("Error".to_string()),
                };
                let mut ctx_l = self.clone(); let mut ctx_r = self.clone();
                ctx_l.insert(id_l.clone(), match &t_l { Type::Linear(_, _) => Resource::Linear(t_l.clone()), _ => Resource::Persistent(t_l.clone()) });
                ctx_r.insert(id_r.clone(), match &t_r { Type::Linear(_, _) => Resource::Linear(t_r.clone()), _ => Resource::Persistent(t_r.clone()) });
                let ty_l = ctx_l.check(b_l)?; let ty_r = ctx_r.check(b_r)?;
                if ty_l != ty_r { return Err("Mismatch".to_string()); }
                ctx_l.resources.remove(id_l); ctx_r.resources.remove(id_r);
                if ctx_l.resources != ctx_r.resources { return Err("Linearity match mismatch".to_string()); }
                self.resources = ctx_l.resources; Ok(ty_l)
            }
            Term::MkPair(t1, t2) => Ok(Type::Pair(Box::new(self.check(t1)?), Box::new(self.check(t2)?))),
            Term::Unpack(tgt, a1, a2, b) => {
                let (t_x, t_y) = match self.check(tgt)? {
                    Type::Pair(t1, t2) => (*t1, *t2),
                    Type::Linear(Permission::Full, inner) => match *inner { Type::Pair(t1, t2) => (Type::Linear(Permission::Full, t1), Type::Linear(Permission::Full, t2)), _ => return Err("Err".to_string()) },
                    Type::Linear(Permission::Fraction(n, d), inner) => match *inner { Type::Pair(t1, t2) => (Type::Linear(Permission::Fraction(n, d), t1), Type::Linear(Permission::Fraction(n, d), t2)), _ => return Err("Err".to_string()) },
                    _ => return Err("Err".to_string()),
                };
                let mut b_ctx = self.clone();
                b_ctx.insert(a1.clone(), match &t_x { Type::Linear(_, _) => Resource::Linear(t_x.clone()), _ => Resource::Persistent(t_x.clone()) });
                b_ctx.insert(a2.clone(), match &t_y { Type::Linear(_, _) => Resource::Linear(t_y.clone()), _ => Resource::Persistent(t_y.clone()) });
                let r_ty = b_ctx.check(b)?;
                for (name, res) in b_ctx.resources.iter() { if let Resource::Linear(_) = res { if name == a1 || name == a2 { return Err("Unconsumed".to_string()); } } }
                b_ctx.resources.remove(a1); b_ctx.resources.remove(a2); self.resources = b_ctx.resources; Ok(r_ty)
            }
            Term::If(c, t, f) => {
                if self.check(c)? != Type::Bool { return Err("Err".to_string()); }
                let mut ctx_t = self.clone(); let mut ctx_f = self.clone();
                let ty_t = ctx_t.check(t)?; let ty_f = ctx_f.check(f)?;
                if ty_t != ty_f { return Err("Err".to_string()); }
                if ctx_t.resources != ctx_f.resources { return Err("Err".to_string()); }
                self.resources = ctx_t.resources; Ok(ty_t)
            }
            Term::Add(t1, t2) | Term::Sub(t1, t2) => { if self.check(t1)? == Type::Int && self.check(t2)? == Type::Int { Ok(Type::Int) } else { Err("Err".to_string()) } }
            Term::Eq(t1, t2) => { if self.check(t1)? == Type::Int && self.check(t2)? == Type::Int { Ok(Type::Bool) } else { Err("Err".to_string()) } }
            Term::Fix(i) => { match self.check(i)? { Type::Pi(_, t1, t2) if t1 == t2 => Ok(*t1), _ => Err("Err".to_string()) } }
            Term::Var(name) => {
                match self.resources.get(name).cloned() {
                    Some(Resource::Persistent(t)) => Ok(t), Some(Resource::Linear(t)) => { self.resources.remove(name); Ok(t) }, None => Err(format!("Unbound '{}'", name)),
                }
            }
            Term::Free(tgt) => { match self.check(tgt)? { Type::Linear(Permission::Full, _) => Ok(Type::Unit), _ => Err("Err".to_string()) } }
            Term::Split(tgt, a1, a2, b) => {
                let inner = match self.resources.remove(tgt).ok_or("Err")? { Resource::Linear(Type::Linear(Permission::Full, t)) => t, _ => return Err("Err".to_string()) };
                let mut b_ctx = self.clone(); b_ctx.insert(a1.clone(), Resource::Linear(Type::Linear(Permission::Fraction(1, 2), inner.clone()))); b_ctx.insert(a2.clone(), Resource::Linear(Type::Linear(Permission::Fraction(1, 2), inner.clone())));
                let r_ty = b_ctx.check(b)?; for (n, r) in b_ctx.resources { if let Resource::Linear(_) = r { return Err(format!("Unconsumed '{}'", n)); } } Ok(r_ty)
            }
            Term::Merge(a1, a2, tgt, b) => {
                let r1 = self.resources.remove(a1).ok_or("Err")?; let r2 = self.resources.remove(a2).ok_or("Err")?;
                let inner = match (r1, r2) { (Resource::Linear(Type::Linear(Permission::Fraction(n1, d1), t1)), Resource::Linear(Type::Linear(Permission::Fraction(n2, d2), t2))) => { if t1 != t2 || (n1 * d2) + (n2 * d1) != (d1 * d2) { return Err("Err".to_string()); } *t1 } _ => return Err("Err".to_string()) };
                let mut b_ctx = self.clone(); b_ctx.insert(tgt.clone(), Resource::Linear(Type::Linear(Permission::Full, Box::new(inner)))); let r_ty = b_ctx.check(b)?; for (n, r) in b_ctx.resources { if let Resource::Linear(_) = r { return Err(format!("Unconsumed '{}'", n)); } } Ok(r_ty)
            }
            Term::Abs(p, p_ty, b) => {
                let mut b_ctx = self.clone_persistent(); b_ctx.insert(p.clone(), match p_ty { Type::Linear(_, _) => Resource::Linear(p_ty.clone()), _ => Resource::Persistent(p_ty.clone()) });
                let b_ty = b_ctx.check(b)?; for (n, r) in b_ctx.resources { if let Resource::Linear(_) = r { return Err(format!("Unconsumed '{}'", n)); } } Ok(Type::Pi(p.clone(), Box::new(p_ty.clone()), Box::new(b_ty)))
            }
            Term::App(t1, t2) => { match self.check(t1)? { Type::Pi(p, _, r_ty) => Ok(r_ty.substitute(&p, t2)), _ => Err("Err".to_string()) } }
        }
    }
}