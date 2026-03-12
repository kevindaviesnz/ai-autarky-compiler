# Contributing to Autarky (Project Ouroboros)

First off, thank you for considering contributing to Autarky! 

Autarky v1.0.0 represents the completion of the "Ouroboros Cycle"—a fully self-hosting compiler that mathematically proves memory safety using a strict Linear Type System, without relying on a Garbage Collector. 

We are excited to open the doors to the community to help push Autarky from a stack-based VM to a native, bare-metal powerhouse.

## 🧠 The Autarky Philosophy
Before writing code, it is critical to understand the core rule of the language: **Every bound variable must be consumed exactly once.** Autarky relies on *Substructural Logic*. If you are touching the `typecheck.rs` file, you are acting as the "Scope Janitor." Your job is to ensure that no variables are leaked and no variables are used twice, especially across diverging branching timelines.

## 🗺️ Architecture Overview
To help you navigate the codebase, here is how the compilation pipeline currently works:

1.  **Front-End:** `parser.rs` and `ast.rs` handle lexing and parsing raw `.aut` text into heavily nested Recursive Mu-Types.
2.  **Middle-End:** `typecheck.rs` mathematically validates the memory timeline. `ir.rs` then performs "Proof Erasure," stripping away the type metadata to create a lean Intermediate Representation (IR).
3.  **Back-End:** `codegen.rs` flattens the IR into a 1D array of custom bytecode instructions. `vm.rs` executes this bytecode in a simulated stack environment.

## 🚀 The Next Frontier: The LLVM Backend
The primary open initiative for Autarky is replacing the custom Virtual Machine with a **Native LLVM Backend**. 



Currently, Autarky produces custom bytecode. We want it to produce blazing-fast native binaries (x86_64, ARM64). 

**How you can help:**
* **Objective:** Introduce an LLVM wrapper (like the `inkwell` crate) into the pipeline.
* **The Hook:** You will want to intercept the compilation right after `ir.rs` (Proof Erasure). Since the IR is already simplified and guaranteed to be memory-safe, you can map Autarky IR nodes directly to LLVM IR instructions.
* **Goal:** A user should be able to run `autarky build main.aut` and receive a standalone executable.

## 🛠️ How to Contribute
1.  **Fork the repo** and clone it locally.
2.  **Run the test suite:** Ensure the self-hosting bootstrapper still works by running `cargo run --release -- --file main.aut`. If the VM panics with an expected `Unbound '1'` error on the final test, the system is perfectly stable.
3.  **Create a branch:** e.g., `git checkout -b feature/llvm-ir-gen`.
4.  **Write code & add examples:** If you add a new feature (like a standard library function), add an `.aut` script to the `examples/` folder to prove it works.
5.  **Submit a Pull Request:** Explain your architectural decisions. If you touched the Type Checker, explain how you maintained linear memory safety.