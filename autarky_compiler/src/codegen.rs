use crate::ir::IRTerm;
use crate::vm::Value;

#[derive(Debug, Clone, PartialEq)]
pub enum Instruction {
    PushInt(u32), PushUnit, PushBool(bool), PushString(String),
    AllocArray, ReadArray, WriteArray, ReadFileOS,
    MakeLeft, MakeRight, BranchMatch(usize),
    Bind(String), MakePair, UnpackAndBind(String, String),
    JumpIfFalse(usize), Jump(usize), Add, Sub, Eq, Fix,
    PushVar(String), MakeClosure(String, Vec<Instruction>), Call, Free, Return
}

pub fn generate_bytecode(ir: &IRTerm) -> Vec<Instruction> {
    let mut code = Vec::new();
    match ir {
        IRTerm::IntVal(n) => code.push(Instruction::PushInt(*n)),
        IRTerm::UnitVal => code.push(Instruction::PushUnit),
        IRTerm::BoolVal(b) => code.push(Instruction::PushBool(*b)),
        IRTerm::StringVal(s) => code.push(Instruction::PushString(s.clone())),
        IRTerm::Left(t) => { 
            code.extend(generate_bytecode(t)); 
            code.push(Instruction::MakeLeft); 
        }
        IRTerm::Right(t) => { 
            code.extend(generate_bytecode(t)); 
            code.push(Instruction::MakeRight); 
        }
        IRTerm::Match(tgt, id_l, b_l, id_r, b_r) => {
            code.extend(generate_bytecode(tgt));
            let mut code_l = Vec::new(); 
            code_l.push(Instruction::Bind(id_l.clone())); 
            code_l.extend(generate_bytecode(b_l));
            
            let mut code_r = Vec::new(); 
            code_r.push(Instruction::Bind(id_r.clone())); 
            code_r.extend(generate_bytecode(b_r));
            
            code_l.push(Instruction::Jump(code_r.len() + 1));
            code.push(Instruction::BranchMatch(code_l.len() + 1));
            code.extend(code_l); 
            code.extend(code_r);
        }
        IRTerm::Alloc(s, i) => { 
            code.extend(generate_bytecode(s)); 
            code.extend(generate_bytecode(i)); 
            code.push(Instruction::AllocArray); 
        }
        IRTerm::Read(a, i) => { 
            code.extend(generate_bytecode(a)); 
            code.extend(generate_bytecode(i)); 
            code.push(Instruction::ReadArray); 
        }
        IRTerm::Write(a, i, v) => { 
            code.extend(generate_bytecode(a)); 
            code.extend(generate_bytecode(i)); 
            code.extend(generate_bytecode(v)); 
            code.push(Instruction::WriteArray); 
        }
        IRTerm::ReadFile(p) => { 
            code.extend(generate_bytecode(p)); 
            code.push(Instruction::ReadFileOS); 
        }
        IRTerm::MkPair(t1, t2) => { 
            code.extend(generate_bytecode(t1)); 
            code.extend(generate_bytecode(t2)); 
            code.push(Instruction::MakePair); 
        }
        IRTerm::Unpack(tgt, a1, a2, b) => {
            code.extend(generate_bytecode(tgt));
            code.push(Instruction::UnpackAndBind(a1.clone(), a2.clone()));
            code.extend(generate_bytecode(b));
        }
        IRTerm::If(c, t, f) => {
            code.extend(generate_bytecode(c));
            let mut t_code = generate_bytecode(t); 
            let f_code = generate_bytecode(f);
            
            t_code.push(Instruction::Jump(f_code.len() + 1));
            code.push(Instruction::JumpIfFalse(t_code.len() + 1));
            code.extend(t_code); 
            code.extend(f_code);
        }
        IRTerm::Add(t1, t2) => { 
            code.extend(generate_bytecode(t1)); 
            code.extend(generate_bytecode(t2)); 
            code.push(Instruction::Add); 
        }
        IRTerm::Sub(t1, t2) => { 
            code.extend(generate_bytecode(t1)); 
            code.extend(generate_bytecode(t2)); 
            code.push(Instruction::Sub); 
        }
        IRTerm::Eq(t1, t2) => { 
            code.extend(generate_bytecode(t1)); 
            code.extend(generate_bytecode(t2)); 
            code.push(Instruction::Eq); 
        }
        IRTerm::Fix(i) => { 
            code.extend(generate_bytecode(i)); 
            code.push(Instruction::Fix); 
        }
        IRTerm::Var(name) => {
            code.push(Instruction::PushVar(name.clone()));
        }
        IRTerm::Free(tgt) => { 
            code.extend(generate_bytecode(tgt)); 
            code.push(Instruction::Free); 
        }
        IRTerm::Abs(p, b) => {
            code.push(Instruction::MakeClosure(p.clone(), generate_bytecode(b)));
        }
        IRTerm::App(t1, t2) => { 
            code.extend(generate_bytecode(t2)); 
            code.extend(generate_bytecode(t1)); 
            code.push(Instruction::Call); 
        }
        IRTerm::Erased => {}
    }
    code
}

/* ========================================================================
AUTARKY TO RUST BRIDGE (STAGE 8 PARSER)
========================================================================
*/

pub fn parse_autarky_bytecode(val: &Value) -> Vec<Instruction> {
    let mut code = Vec::new();
    let mut current = val;

    loop {
        match current {
            Value::Left(inner) => {
                if matches!(**inner, Value::Unit) {
                    break;
                } else {
                    panic!("Unexpected Left value in list backbone: {:?}", inner);
                }
            }
            Value::Right(inner) => {
                if let Value::Pair(head, tail) = &**inner {
                    flatten_instruction(head, &mut code);
                    current = tail;
                } else {
                    panic!("Expected Pair in list Cons node");
                }
            }
            _ => panic!("Invalid list structure: {:?}", current),
        }
    }
    code
}

fn flatten_instruction(val: &Value, code: &mut Vec<Instruction>) {
    match val {
        // 1. PushVar: Left(Int)
        Value::Left(inner) => {
            if let Value::Int(x) = &**inner {
                code.push(Instruction::PushVar(x.to_string()));
            } else {
                panic!("Expected Int inside PushVar instruction");
            }
        }
        Value::Right(r1) => match &**r1 {
            // 2. MakeClosure: Right(Left(Pair(Int, IL)))
            Value::Left(l2) => {
                if let Value::Pair(p_val, body_val) = &**l2 {
                    if let Value::Int(p) = &**p_val {
                        let body = parse_autarky_bytecode(body_val);
                        code.push(Instruction::MakeClosure(p.to_string(), body));
                    } else {
                        panic!("Expected Int for MakeClosure parameter");
                    }
                } else {
                    panic!("Expected Pair in MakeClosure instruction");
                }
            }
            Value::Right(r2) => match &**r2 {
                // 3. Call: Right(Right(Left(Unit)))
                Value::Left(_) => code.push(Instruction::Call),
                
                Value::Right(r3) => match &**r3 {
                    // 4. Return: Right(Right(Right(Left(Unit))))
                    Value::Left(_) => code.push(Instruction::Return),
                    
                    Value::Right(r4) => match &**r4 {
                        // 5. MakePair: Right(Right(Right(Right(Left(Unit)))))
                        Value::Left(_) => code.push(Instruction::MakePair),
                        
                        Value::Right(r5) => match &**r5 {
                            // 6. Unpack: Right(Right(Right(Right(Right(Left(Pair(Int, Int)))))))
                            Value::Left(l6) => {
                                if let Value::Pair(v1, v2) = &**l6 {
                                    if let (Value::Int(id1), Value::Int(id2)) = (&**v1, &**v2) {
                                        code.push(Instruction::UnpackAndBind(id1.to_string(), id2.to_string()));
                                    } else {
                                        panic!("Expected Ints in Unpack instruction");
                                    }
                                } else {
                                    panic!("Expected Pair in Unpack instruction");
                                }
                            }
                            
                            Value::Right(r6) => match &**r6 {
                                // 7. MakeLeft: Right(Right(Right(Right(Right(Right(Left(Unit)))))))
                                Value::Left(_) => code.push(Instruction::MakeLeft),
                                
                                Value::Right(r7) => match &**r7 {
                                    // 8. MakeRight: Right(Right(Right(Right(Right(Right(Right(Left(Unit))))))))
                                    Value::Left(_) => code.push(Instruction::MakeRight),
                                    
                                    // 9. BranchMatch: Right(Right(Right(Right(Right(Right(Right(Right(Pair(IL, IL)))))))))
                                    Value::Right(r8) => {
                                        if let Value::Pair(il1, il2) = &**r8 {
                                            let mut code_l = parse_autarky_bytecode(il1);
                                            let code_r = parse_autarky_bytecode(il2);
                                            
                                            // Flatten the tree into linear offset jumps!
                                            code_l.push(Instruction::Jump(code_r.len() + 1));
                                            code.push(Instruction::BranchMatch(code_l.len() + 1));
                                            code.extend(code_l);
                                            code.extend(code_r);
                                        } else {
                                            panic!("Expected Pair(IL, IL) in BranchMatch instruction");
                                        }
                                    }
                                    _ => panic!("Expected Left or Right encoding instruction (level 8)"),
                                }
                                _ => panic!("Expected Left or Right encoding instruction (level 7)"),
                            }
                            _ => panic!("Expected Left or Right encoding instruction (level 6)"),
                        }
                        _ => panic!("Expected Left or Right encoding instruction (level 5)"),
                    }
                    _ => panic!("Expected Left or Right encoding instruction (level 4)"),
                }
                _ => panic!("Expected Left or Right encoding instruction (level 3)"),
            }
            _ => panic!("Expected Left or Right encoding instruction (level 2)"),
        }
        _ => panic!("Unknown instruction encoding: {:?}", val),
    }
}