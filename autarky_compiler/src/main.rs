use clap::Parser as CliParser;
use std::fs;

mod ast;
mod codegen;
mod ir;
mod parser;
mod typecheck;
mod vm;

use ast::{Permission, Resource, Type};
use typecheck::Context;
use vm::{Value, VM};

/// Project Ouroboros: Autarky Compiler Stage 1
#[derive(CliParser, Debug)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[arg(short, long)]
    file: String,

    #[arg(short, long, default_value_t = false)]
    prove: bool,
}

fn main() {
    let cli = Cli::parse();

    println!("========================================");
    println!("🐍 Autarky Compiler Bootstrapper v1.1.0");
    println!("========================================");

    let source_code = match fs::read_to_string(&cli.file) {
        Ok(code) => code,
        Err(e) => { 
            eprintln!("❌ Failed to read file '{}': {}", cli.file, e); 
            std::process::exit(1); 
        }
    };

    let mut parser = parser::Parser::new(&source_code);
    let ast = match parser.parse_term() {
        Ok(term) => term,
        Err(e) => { 
            eprintln!("❌ Parsing Failed!\n{}", e); 
            std::process::exit(1); 
        }
    };

    let mut ctx = Context::new();
    ctx.insert(
        "memory_ptr".to_string(), 
        Resource::Linear(Type::Linear(Permission::Full, Box::new(Type::Universe(1))))
    );

    if let Err(e) = ctx.check(&ast) {
        eprintln!("❌ Verification Failed!"); 
        eprintln!("{}", e); 
        std::process::exit(1);
    }
    println!("✅ Type Check Passed (Memory Safety Guaranteed)");

    let optimized_ir = ir::generate_ir(&ast);
    println!("✅ Proof Erasure Complete");
    
    let bytecode = codegen::generate_bytecode(&optimized_ir);
    println!("✅ Bytecode Generated");
    
    println!("----------------------------------------");
    println!("🚀 Executing inside Autarky VM...");
    
    let mut runtime = VM::new();
    runtime.insert_global("memory_ptr".to_string(), Value::MemoryAddress(0xDEADBEEF));

    match runtime.execute(&bytecode) {
        Ok(Some(result)) => { 
            println!("✅ Execution Finished Successfully!"); 
            println!("-> Raw Return Value: {:?}", result); 

            if let Value::Pair(_proof, bytecode_val) = &result {
                println!("----------------------------------------");
                println!("🧩 Parsing Self-Hosted Compiler Output...");
                
                let parsed_instructions = codegen::parse_autarky_bytecode(bytecode_val);
                
                println!("✅ Native Rust Bytecode Generated from Autarky:");
                println!("{:#?}", parsed_instructions);

                println!("----------------------------------------");
                println!("🧪 Testing Self-Compiled Code...");
                
                // Construct a test: Push(10) + Push(11) + MakePair + [Generated Closure] + Call
                let mut test_program = vec![
                    codegen::Instruction::PushInt(10),
                    codegen::Instruction::PushInt(11),
                    codegen::Instruction::MakePair,
                ];
                test_program.extend(parsed_instructions);
                test_program.push(codegen::Instruction::Call);

                let mut test_vm = VM::new();
                match test_vm.execute(&test_program) {
                    Ok(Some(final_val)) => {
                        println!("✨ SELF-EXECUTION SUCCESS!");
                        println!("-> Sent: Pair(10, 11)");
                        println!("-> Received: {:?}", final_val);
                    }
                    Ok(None) => println!("❌ Test returned no value."),
                    Err(e) => println!("💥 Test VM Panic: {}", e),
                }
            }
        }
        Ok(None) => { 
            println!("✅ Execution Finished (No Return Value)"); 
        }
        Err(e) => { 
            eprintln!("💥 VM Panic!"); 
            eprintln!("{}", e); 
            std::process::exit(1); 
        }
    }
}