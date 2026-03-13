use std::fs;
use std::collections::HashMap;
use inkwell::context::Context;
use serde_json::json;
use clap::Parser;

mod ast;
mod parser;
mod typecheck;
mod ir;
mod vm;
mod codegen;

#[derive(Parser, Debug)]
#[command(author, version, about = "Autarky Compiler (atk)", long_about = None)]
struct Args {
    /// The .aut file to compile and execute
    #[arg(short, long)]
    file: String,

    /// Output results as JSON for AI agent parsing
    #[arg(long, default_value_t = false)]
    json: bool,

    /// Allocated compute hours (Injected as SYS_COMPUTE)
    #[arg(long, default_value_t = 0)]
    compute: i64,

    /// Allocated network credits (Injected as SYS_CREDITS)
    #[arg(long, default_value_t = 0)]
    credits: i64,
}

fn main() {
    let args = Args::parse();
    
    let source = fs::read_to_string(&args.file).unwrap_or_else(|_| {
        if args.json {
            println!("{}", json!({"status": "error", "stage": "fs", "message": "Could not read file"}));
        } else {
            println!("❌ File Error: Could not read {}", args.file);
        }
        std::process::exit(1);
    });

    // 1. Parse
    let mut parser = parser::Parser::new(&source);
    let mut ast = match parser.parse() {
        Ok(nodes) => nodes,
        Err(e) => {
            if args.json { println!("{}", json!({"status": "error", "stage": "parser", "message": e})); }
            else { println!("❌ Parsing Error: {}", e); }
            return;
        }
    };

    // 2. Inject External Resources (Compute & Credits) into the AST
    // We do this by wrapping the parsed AST in two Lambda applications,
    // functionally equivalent to: let SYS_COMPUTE = args.compute in (let SYS_CREDITS = args.credits in ast)
    ast = ast::Expr::App {
        func: Box::new(ast::Expr::Lambda {
            param: "SYS_COMPUTE".to_string(),
            param_type: ast::Type::Int,
            body: Box::new(ast::Expr::App {
                func: Box::new(ast::Expr::Lambda {
                    param: "SYS_CREDITS".to_string(),
                    param_type: ast::Type::Int,
                    body: Box::new(ast),
                }),
                arg: Box::new(ast::Expr::IntLiteral(args.credits)),
            }),
        }),
        arg: Box::new(ast::Expr::IntLiteral(args.compute)),
    };

    // 3. Type Check
    let checker = typecheck::TypeChecker::new();
    let (final_type, _) = match checker.check(HashMap::new(), &ast) {
        Ok(t) => t,
        Err(e) => {
            if args.json { println!("{}", json!({"status": "error", "stage": "typecheck", "message": e})); }
            else { println!("❌ Type Check Error: {}", e); }
            return;
        }
    };

    // 4. IR Generation
    let ir_root = ir::erase_proofs(&ast);

    // 5. VM Execution
    let mut machine = vm::VirtualMachine::new();
    let vm_res = match machine.evaluate(&HashMap::new(), &ir_root) {
        Ok(res) => res,
        Err(e) => {
            if args.json { println!("{}", json!({"status": "error", "stage": "vm", "message": e})); }
            else { println!("❌ VM Execution Error: {}", e); }
            return;
        }
    };

    // 6. LLVM Codegen
    let context = Context::create();
    let compiler = codegen::Compiler::new(&context, "autarky_module");
    compiler.create_main_function("autarky_main");
    
    if let Err(e) = compiler.compile_and_return(&ir_root) {
        if args.json { println!("{}", json!({"status": "error", "stage": "codegen", "message": e})); }
        else { println!("❌ LLVM Codegen Error: {}", e); }
        return;
    }
    
    if let Err(e) = compiler.module.print_to_file("output.ll") {
        if args.json { println!("{}", json!({"status": "error", "stage": "codegen_save", "message": e.to_string()})); }
        else { println!("❌ LLVM Save Error: {}", e.to_string()); }
        return;
    }

    if args.json {
        // AI-Optimized Output
        let output = json!({
            "status": "success",
            "type": format!("{:?}", final_type),
            "vm_result": format!("{:?}", vm_res),
            "injected_resources": {
                "compute": args.compute,
                "credits": args.credits
            },
            "native_ir_generated": true
        });
        println!("{}", output);
    } else {
        // Human-Optimized Output
        println!("--- Compiling {} ---", args.file);
        println!("💉 Injected SYS_COMPUTE: {}", args.compute);
        println!("💉 Injected SYS_CREDITS: {}", args.credits);
        println!("✅ Type Check Passed: {:?}", final_type);
        println!("🚀 VM Result: {:?}", vm_res);
        println!("💾 Native IR saved to output.ll");
    }
}