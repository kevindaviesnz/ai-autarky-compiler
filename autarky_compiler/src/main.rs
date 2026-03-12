// src/main.rs

mod ast;
mod ir;
mod parser;
mod typecheck;
mod vm;

use std::collections::HashMap;
use std::env;
use std::fs;

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

    if contents.is_empty() {
        return;
    }

    println!("--- Compiling {} ---", filename);

    // 1. Front-End: Parse the raw text into the AST
    let mut parser = parser::Parser::new(&contents);
    let ast = match parser.parse() {
        Ok(expr) => expr,
        Err(e) => {
            println!("Parse Error: {}", e);
            return;
        }
    };

    println!("✅ Parsing Complete!");

    // 2. Middle-End: Run the Scope Janitor to mathematically prove memory safety
    let checker = typecheck::TypeChecker::new();
    let empty_env = HashMap::new(); // The universe starts with no variables

    match checker.check(empty_env, &ast) {
        Ok((final_type, _remaining_env)) => {
            println!("✅ Type Check Passed! Final Evaluated Type: {:?}", final_type);
            
            // 3. Back-End Prep: Erase types to generate pure execution instructions
            let ir = ir::erase_proofs(&ast);
            println!("✅ Proof Erasure Complete! IR Generated.");
            
            // 4. Execution: Pass the IR to the Virtual Machine
            println!("🚀 Booting Virtual Machine...");
            let mut virtual_machine = vm::VirtualMachine::new();
            
            // Because our dummy parser currently spits out `\x: Int . x + 10`,
            // we will simulate applying the argument `42` to it so it actually runs!
            let application_ir = ir::IrNode::App(
                Box::new(ir), 
                Box::new(ir::IrNode::Int(42))
            );

            let vm_env = HashMap::new();
            match virtual_machine.evaluate(&vm_env, &application_ir) {
                Ok(result) => {
                    println!("=====================================");
                    println!("🎉 PROGRAM EXECUTED SUCCESSFULLY!");
                    println!("FINAL RESULT: {:?}", result);
                    println!("=====================================");
                }
                Err(e) => println!("❌ {}", e),
            }
        }
        Err(e) => {
            println!("❌ {}", e);
        }
    }
}