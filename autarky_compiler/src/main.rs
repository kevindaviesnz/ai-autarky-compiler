// src/main.rs

mod ast;
mod codegen;
mod ir;
mod parser;
mod typecheck;
mod vm;

use std::collections::HashMap;
use std::env;
use std::fs;
use std::path::Path;
use crate::ir::IrNode;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 3 || args[1] != "--file" {
        println!("Usage: cargo run --release -- --file <filename.aut>");
        return;
    }

    let filename = &args[2];
    let contents = fs::read_to_string(filename).unwrap_or_else(|_| {
        println!("Error: Could not read file {}", filename);
        String::new()
    });

    if contents.is_empty() { return; }

    println!("--- Compiling {} ---", filename);

    let mut parser = parser::Parser::new(&contents);
    let ast = match parser.parse() {
        Ok(expr) => expr,
        Err(e) => { println!("Parse Error: {}", e); return; }
    };

    println!("✅ Parsing Complete!");

    let checker = typecheck::TypeChecker::new();
    let empty_env = HashMap::new();

    match checker.check(empty_env, &ast) {
        Ok((final_type, _)) => {
            println!("✅ Type Check Passed! Final Evaluated Type: {:?}", final_type);
            
            let ir_root = ir::erase_proofs(&ast);
            println!("✅ Proof Erasure Complete! IR Generated.");
            
            // Construct the application node: (\x . Body) 42
            let application_ir = IrNode::App(
                Box::new(ir_root), 
                Box::new(IrNode::Int(42))
            );

            println!("🚀 Booting Virtual Machine...");
            let mut virtual_machine = vm::VirtualMachine::new();
            match virtual_machine.evaluate(&HashMap::new(), &application_ir) {
                Ok(result) => {
                    println!("=====================================");
                    println!("🎉 VM EXECUTED SUCCESSFULLY!");
                    println!("FINAL RESULT: {:?}", result);
                    println!("=====================================");
                }
                Err(e) => println!("❌ VM Error: {}", e),
            }

            println!("\n⚙️  Generating LLVM IR...");
            let context = inkwell::context::Context::create();
            let compiler = codegen::Compiler::new(&context, "autarky_module");
            compiler.create_main_function("autarky_main");

            match compiler.compile_and_return(&application_ir) {
                Ok(_) => {
                    println!("✅ LLVM IR Generated Successfully!");
                    let out_path = Path::new("output.ll");
                    compiler.module.print_to_file(out_path).ok();
                    println!("💾 IR saved to output.ll");
                    println!("=====================================");
                    compiler.module.print_to_stderr();
                    println!("=====================================");
                }
                Err(e) => println!("❌ LLVM Compilation Error: {}", e),
            }
        }
        Err(e) => println!("❌ Type Check Error: {}", e),
    }
}