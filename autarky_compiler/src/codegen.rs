// src/codegen.rs

use std::cell::Cell;
use std::collections::HashMap;
use inkwell::builder::{Builder, BuilderError};
use inkwell::context::Context;
use inkwell::module::Module;
use inkwell::values::{BasicValueEnum, FunctionValue};
use crate::ir::IrNode;

pub struct Compiler<'ctx> {
    pub context: &'ctx Context,
    pub builder: Builder<'ctx>,
    pub module: Module<'ctx>,
    pub lambda_counter: Cell<usize>,
}

impl<'ctx> Compiler<'ctx> {
    pub fn new(context: &'ctx Context, module_name: &str) -> Self {
        let module = context.create_module(module_name);
        let builder = context.create_builder();
        
        let i64_type = context.i64_type();
        let ptr_type = context.ptr_type(inkwell::AddressSpace::default());
        let malloc_fn_type = ptr_type.fn_type(&[i64_type.into()], false);
        module.add_function("malloc", malloc_fn_type, None);

        Self {
            context,
            builder,
            module,
            lambda_counter: Cell::new(0),
        }
    }

    pub fn create_main_function(&self, name: &str) -> FunctionValue<'ctx> {
        let i64_type = self.context.i64_type();
        let fn_type = i64_type.fn_type(&[], false); 
        let function = self.module.add_function(name, fn_type, None);
        let basic_block = self.context.append_basic_block(function, "entry");
        self.builder.position_at_end(basic_block);
        function
    }

    pub fn compile_and_return(&self, ir: &IrNode) -> Result<(), String> {
        let mut env = HashMap::new();
        let return_value = self.compile_ir(ir, &mut env)?;

        let ret_int = if return_value.is_pointer_value() {
            self.builder.build_ptr_to_int(return_value.into_pointer_value(), self.context.i64_type(), "ret_cast")
                .map_err(|e: BuilderError| e.to_string())?.into()
        } else if return_value.is_float_value() {
            self.builder.build_bit_cast(return_value, self.context.i64_type(), "f_cast")
                .map_err(|e: BuilderError| e.to_string())?.into()
        } else {
            return_value
        };
        
        self.builder.build_return(Some(&ret_int)).map_err(|e: BuilderError| e.to_string())?;
        Ok(())
    }

    pub fn compile_ir(
        &self, 
        ir: &IrNode, 
        env: &mut HashMap<String, BasicValueEnum<'ctx>>
    ) -> Result<BasicValueEnum<'ctx>, String> {
        match ir {
            IrNode::Int(n) => {
                let i64_type = self.context.i64_type();
                Ok(i64_type.const_int(*n as u64, false).into())
            }
            IrNode::Float(f) => {
                let f64_type = self.context.f64_type();
                Ok(f64_type.const_float(*f).into())
            }
            IrNode::Var(name) => {
                env.get(name).copied().ok_or_else(|| format!("LLVM Error: Undefined variable '{}'", name))
            }
            IrNode::Add(left, right) => {
                let lhs = self.compile_ir(left, env)?;
                let rhs = self.compile_ir(right, env)?;
                if lhs.is_int_value() && rhs.is_int_value() {
                    self.builder.build_int_add(lhs.into_int_value(), rhs.into_int_value(), "addtmp").map_err(|e| e.to_string()).map(Into::into)
                } else if lhs.is_float_value() && rhs.is_float_value() {
                    self.builder.build_float_add(lhs.into_float_value(), rhs.into_float_value(), "faddtmp").map_err(|e| e.to_string()).map(Into::into)
                } else {
                    Err("LLVM Error: Type mismatch in Add".to_string())
                }
            }
            IrNode::Sub(left, right) => {
                let lhs = self.compile_ir(left, env)?;
                let rhs = self.compile_ir(right, env)?;
                if lhs.is_int_value() && rhs.is_int_value() {
                    self.builder.build_int_sub(lhs.into_int_value(), rhs.into_int_value(), "subtmp").map_err(|e| e.to_string()).map(Into::into)
                } else if lhs.is_float_value() && rhs.is_float_value() {
                    self.builder.build_float_sub(lhs.into_float_value(), rhs.into_float_value(), "fsubtmp").map_err(|e| e.to_string()).map(Into::into)
                } else {
                    Err("LLVM Error: Type mismatch in Sub".to_string())
                }
            }
            IrNode::Mul(left, right) => {
                let lhs = self.compile_ir(left, env)?;
                let rhs = self.compile_ir(right, env)?;
                if lhs.is_int_value() && rhs.is_int_value() {
                    self.builder.build_int_mul(lhs.into_int_value(), rhs.into_int_value(), "multmp").map_err(|e| e.to_string()).map(Into::into)
                } else if lhs.is_float_value() && rhs.is_float_value() {
                    self.builder.build_float_mul(lhs.into_float_value(), rhs.into_float_value(), "fmultmp").map_err(|e| e.to_string()).map(Into::into)
                } else {
                    Err("LLVM Error: Type mismatch in Mul".to_string())
                }
            }
            IrNode::Div(left, right) => {
                let lhs = self.compile_ir(left, env)?;
                let rhs = self.compile_ir(right, env)?;
                if lhs.is_int_value() && rhs.is_int_value() {
                    self.builder.build_int_signed_div(lhs.into_int_value(), rhs.into_int_value(), "divtmp").map_err(|e| e.to_string()).map(Into::into)
                } else if lhs.is_float_value() && rhs.is_float_value() {
                    self.builder.build_float_div(lhs.into_float_value(), rhs.into_float_value(), "fdivtmp").map_err(|e| e.to_string()).map(Into::into)
                } else {
                    Err("LLVM Error: Type mismatch in Div".to_string())
                }
            }
            IrNode::Eq(left, right) => {
                let lhs = self.compile_ir(left, env)?;
                let rhs = self.compile_ir(right, env)?;
                if lhs.is_int_value() && rhs.is_int_value() {
                    let cmp = self.builder.build_int_compare(inkwell::IntPredicate::EQ, lhs.into_int_value(), rhs.into_int_value(), "eqtmp").map_err(|e| e.to_string())?;
                    Ok(cmp.into())
                } else if lhs.is_float_value() && rhs.is_float_value() {
                    let cmp = self.builder.build_float_compare(inkwell::FloatPredicate::OEQ, lhs.into_float_value(), rhs.into_float_value(), "feqtmp").map_err(|e| e.to_string())?;
                    Ok(cmp.into())
                } else {
                    Err("LLVM Error: Type mismatch in Eq".to_string())
                }
            }
            IrNode::Lam(param, body) => {
                let count = self.lambda_counter.get();
                self.lambda_counter.set(count + 1);
                let fn_name = format!("lambda_{}", count);
                let i64_type = self.context.i64_type();
                let fn_type = i64_type.fn_type(&[i64_type.into()], false);
                let function = self.module.add_function(&fn_name, fn_type, None);
                
                let current_block = self.builder.get_insert_block().ok_or("No current block")?;
                let basic_block = self.context.append_basic_block(function, "entry");
                self.builder.position_at_end(basic_block);
                
                let arg = function.get_nth_param(0).unwrap();
                arg.set_name(param);
                
                let mut local_env = env.clone();
                local_env.insert(param.clone(), arg);
                
                let return_val = self.compile_ir(body, &mut local_env)?;
                
                let ret_int = if return_val.is_pointer_value() {
                    self.builder.build_ptr_to_int(return_val.into_pointer_value(), i64_type, "ret_cast").map_err(|e| e.to_string())?.into()
                } else if return_val.is_float_value() {
                    self.builder.build_bit_cast(return_val, i64_type, "f_cast").map_err(|e: BuilderError| e.to_string())?.into()
                } else {
                    return_val
                };
                
                self.builder.build_return(Some(&ret_int)).map_err(|e| e.to_string())?;
                self.builder.position_at_end(current_block);
                Ok(function.as_global_value().as_pointer_value().into())
            }
            IrNode::App(func, arg) => {
                if let IrNode::Lam(bind_name, in_body) = &**func {
                    if let IrNode::Lam(fn_param, fn_body) = &**arg {
                        let i64_type = self.context.i64_type();
                        let fn_type = i64_type.fn_type(&[i64_type.into()], false);
                        
                        let fn_val = self.module.add_function(&format!("rec_{}", bind_name), fn_type, None);
                        
                        let mut rec_env = env.clone();
                        rec_env.insert(bind_name.clone(), fn_val.as_global_value().as_pointer_value().into());
                        
                        let current_block = self.builder.get_insert_block().unwrap();
                        let basic_block = self.context.append_basic_block(fn_val, "entry");
                        self.builder.position_at_end(basic_block);
                        
                        let arg_val = fn_val.get_nth_param(0).unwrap();
                        arg_val.set_name(fn_param);
                        
                        let mut fn_env = rec_env.clone();
                        fn_env.insert(fn_param.clone(), arg_val);
                        
                        let ret_val = self.compile_ir(fn_body, &mut fn_env)?;
                        let ret_int = if ret_val.is_pointer_value() {
                            self.builder.build_ptr_to_int(ret_val.into_pointer_value(), i64_type, "ret_cast").unwrap().into()
                        } else if ret_val.is_float_value() {
                            self.builder.build_bit_cast(ret_val, i64_type, "f_cast").unwrap().into()
                        } else {
                            ret_val
                        };
                        self.builder.build_return(Some(&ret_int)).unwrap();
                        
                        self.builder.position_at_end(current_block);
                        return self.compile_ir(in_body, &mut rec_env);
                    }
                }

                let compiled_func = self.compile_ir(func, env)?;
                let compiled_arg = self.compile_ir(arg, env)?;
                let i64_type = self.context.i64_type();
                let fn_type = i64_type.fn_type(&[i64_type.into()], false);
                let ptr_type = self.context.ptr_type(inkwell::AddressSpace::default());
                
                let func_ptr = if compiled_func.is_int_value() {
                    self.builder.build_int_to_ptr(compiled_func.into_int_value(), ptr_type, "fn_cast").unwrap()
                } else {
                    compiled_func.into_pointer_value()
                };

                let arg_int = if compiled_arg.is_pointer_value() {
                    self.builder.build_ptr_to_int(compiled_arg.into_pointer_value(), i64_type, "arg_cast").unwrap().into()
                } else if compiled_arg.is_float_value() {
                    self.builder.build_bit_cast(compiled_arg, i64_type, "f_cast").unwrap().into()
                } else {
                    compiled_arg
                };
                
                let call_site = self.builder.build_indirect_call(fn_type, func_ptr, &[arg_int.into()], "calltmp").map_err(|e| e.to_string())?;
                
                call_site.set_tail_call(true);
                
                // FIXED: Changed to .left().unwrap()
                Ok(call_site.try_as_basic_value().left().unwrap())
            }
            IrNode::MkPair(left, right) => {
                let lhs = self.compile_ir(left, env)?;
                let rhs = self.compile_ir(right, env)?;
                let i64_type = self.context.i64_type();
                
                let lhs_int = if lhs.is_pointer_value() { self.builder.build_ptr_to_int(lhs.into_pointer_value(), i64_type, "l_cast").unwrap().into() } else if lhs.is_float_value() { self.builder.build_bit_cast(lhs, i64_type, "f_cast").unwrap().into() } else { lhs };
                let rhs_int = if rhs.is_pointer_value() { self.builder.build_ptr_to_int(rhs.into_pointer_value(), i64_type, "r_cast").unwrap().into() } else if rhs.is_float_value() { self.builder.build_bit_cast(rhs, i64_type, "f_cast").unwrap().into() } else { rhs };

                let struct_type = self.context.struct_type(&[i64_type.into(), i64_type.into()], false);
                
                let null_ptr = self.context.ptr_type(inkwell::AddressSpace::default()).const_null();
                let size_ptr = unsafe { self.builder.build_gep(struct_type, null_ptr, &[i64_type.const_int(1, false)], "size_gep").unwrap() };
                let size = self.builder.build_ptr_to_int(size_ptr, i64_type, "size_int").unwrap();
                
                let malloc_fn = self.module.get_function("malloc").unwrap();
                let malloc_call = self.builder.build_direct_call(malloc_fn, &[size.into()], "malloc_pair").map_err(|e| e.to_string())?;
                
                // FIXED: Changed to .left().unwrap()
                let pair_ptr = malloc_call.try_as_basic_value().left().unwrap().into_pointer_value();
                let ptr_0 = self.builder.build_struct_gep(struct_type, pair_ptr, 0, "p0").map_err(|e| e.to_string())?;
                self.builder.build_store(ptr_0, lhs_int).map_err(|e| e.to_string())?;
                let ptr_1 = self.builder.build_struct_gep(struct_type, pair_ptr, 1, "p1").map_err(|e| e.to_string())?;
                self.builder.build_store(ptr_1, rhs_int).map_err(|e| e.to_string())?;
                
                Ok(pair_ptr.into())
            }
            IrNode::Left(pair) => {
                let c_pair = self.compile_ir(pair, env)?;
                let ptr_type = self.context.ptr_type(inkwell::AddressSpace::default());
                let pair_ptr = if c_pair.is_int_value() { self.builder.build_int_to_ptr(c_pair.into_int_value(), ptr_type, "pcast").unwrap() } else { c_pair.into_pointer_value() };
                
                let i64_type = self.context.i64_type();
                let struct_type = self.context.struct_type(&[i64_type.into(), i64_type.into()], false);
                let ptr_0 = self.builder.build_struct_gep(struct_type, pair_ptr, 0, "l_gep").map_err(|e| e.to_string())?;
                Ok(self.builder.build_load(i64_type, ptr_0, "l_val").map_err(|e| e.to_string())?)
            }
            IrNode::Right(pair) => {
                let c_pair = self.compile_ir(pair, env)?;
                let ptr_type = self.context.ptr_type(inkwell::AddressSpace::default());
                let pair_ptr = if c_pair.is_int_value() { self.builder.build_int_to_ptr(c_pair.into_int_value(), ptr_type, "pcast").unwrap() } else { c_pair.into_pointer_value() };
                
                let i64_type = self.context.i64_type();
                let struct_type = self.context.struct_type(&[i64_type.into(), i64_type.into()], false);
                let ptr_1 = self.builder.build_struct_gep(struct_type, pair_ptr, 1, "r_gep").map_err(|e| e.to_string())?;
                Ok(self.builder.build_load(i64_type, ptr_1, "r_val").map_err(|e| e.to_string())?)
            }
            IrNode::Unpack(v1, v2, pair, body) => {
                let c_pair = self.compile_ir(pair, env)?;
                let ptr_type = self.context.ptr_type(inkwell::AddressSpace::default());
                let pair_ptr = if c_pair.is_int_value() { self.builder.build_int_to_ptr(c_pair.into_int_value(), ptr_type, "pcast").unwrap() } else { c_pair.into_pointer_value() };
                
                let i64_type = self.context.i64_type();
                let struct_type = self.context.struct_type(&[i64_type.into(), i64_type.into()], false);
                
                let p0 = self.builder.build_struct_gep(struct_type, pair_ptr, 0, "u0").map_err(|e| e.to_string())?;
                let p1 = self.builder.build_struct_gep(struct_type, pair_ptr, 1, "u1").map_err(|e| e.to_string())?;
                
                let mut local_env = env.clone();
                local_env.insert(v1.clone(), self.builder.build_load(i64_type, p0, "v1").map_err(|e| e.to_string())?);
                local_env.insert(v2.clone(), self.builder.build_load(i64_type, p1, "v2").map_err(|e| e.to_string())?);
                
                self.compile_ir(body, &mut local_env)
            }
            IrNode::Match(expr, l_var, l_body, r_var, r_body) => {
                let match_val = self.compile_ir(expr, env)?.into_int_value();
                let parent_fn = self.builder.get_insert_block().unwrap().get_parent().unwrap();
                
                let left_bb = self.context.append_basic_block(parent_fn, "left_branch");
                let right_bb = self.context.append_basic_block(parent_fn, "right_branch");
                let merge_bb = self.context.append_basic_block(parent_fn, "match_cont");

                self.builder.build_conditional_branch(match_val, left_bb, right_bb).map_err(|e| e.to_string())?;

                // LEFT BRANCH
                self.builder.position_at_end(left_bb);
                let mut left_env = env.clone();
                left_env.insert(l_var.clone(), match_val.into());
                let left_res = self.compile_ir(l_body, &mut left_env)?;
                self.builder.build_unconditional_branch(merge_bb).map_err(|e| e.to_string())?;
                let new_left_bb = self.builder.get_insert_block().unwrap();

                // RIGHT BRANCH
                self.builder.position_at_end(right_bb);
                let mut right_env = env.clone();
                right_env.insert(r_var.clone(), match_val.into());
                let right_res = self.compile_ir(r_body, &mut right_env)?;
                self.builder.build_unconditional_branch(merge_bb).map_err(|e| e.to_string())?;
                let new_right_bb = self.builder.get_insert_block().unwrap();

                // MERGE (PHI)
                self.builder.position_at_end(merge_bb);
                let phi = self.builder.build_phi(left_res.get_type(), "matchtmp").map_err(|e| e.to_string())?;
                phi.add_incoming(&[(&left_res, new_left_bb), (&right_res, new_right_bb)]);
                
                Ok(phi.as_basic_value())
            }
        }
    }
}