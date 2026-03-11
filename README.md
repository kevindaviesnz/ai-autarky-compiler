🐍 Autarky Compiler (Project Ouroboros)
Autarky is a Turing-complete, linearly-typed functional programming language, compiler, and custom Virtual Machine built entirely from scratch in Rust.

The crowning achievement of Autarky is its Linear Type System, which guarantees strict memory safety, prevents double-frees, and eliminates memory leaks at compile time—all without the runtime overhead of a Garbage Collector.

By Stage 9, Autarky achieved The Singularity: it is a self-hosting language capable of parsing, type-checking, and compiling its own deeply nested Abstract Syntax Tree.

✨ Key Features
Mathematical Memory Safety: Implements a substructural/linear type system where every bound variable must be consumed exactly once.

The Scope Janitor: A rigorous compile-time environment checker that verifies timeline integrity. If a variable is dropped, or if a conditional branch leaks a variable that its sibling branch consumes, the compiler safely panics before a single byte of code is executed.

Zero-Cost Abstractions: Once the linear type checker mathematically proves the memory timeline is sound, the compiler performs "Proof Erasure," stripping away the type constraints and generating highly optimized, raw Intermediate Representation (IR).

Custom Stack-Based VM: A native Rust runtime environment designed to execute Autarky's flattened bytecode, featuring dynamic jump offsets for branching and heap-allocated closure captures.

Self-Hosting Architecture: Autarky's compiler logic (parsing, unpacking, branching, and bytecode generation) is written in Autarky itself.

🧠 The Compiler Pipeline
Parser: Lexes and parses the raw .aut source code into a heavily nested Abstract Syntax Tree (AST).

Linear Type Checker: Evaluates the AST against strict linear logic rules. It clones and splits memory contexts for Match statements and enforces the Branch Equivalence Rule.

IR Generator (Proof Erasure): Discards type constraints after mathematical validation, producing a lean Intermediate Representation.

Code Generator: Translates tree-based IR into a flat, 1D array of bytecode instructions, dynamically calculating absolute jump offsets for the Virtual Machine.

Virtual Machine (Execution): A stack-based runtime that consumes the bytecode, manages the simulated memory pointer, and handles closures and variable bindings.

🧮 The Bootstrapping Instruction Set
To successfully bootstrap itself and avoid terminal character-limit constraints during compilation, Autarky operates on a highly optimized, 9-variant instruction set:

PushVar(id): Looks up and pushes a variable from the environment.

MakeClosure(id, body): Captures the environment and generates a callable lambda.

Call: Executes a closure.

Return: Exits the current frame.

MakePair: Consumes the top two stack values and allocates a Tuple.

UnpackAndBind(id1, id2): Destructures a Tuple and binds its values to the environment.

MakeLeft: Postfix operation wrapping the stack top in a Left Sum Type.

MakeRight: Postfix operation wrapping the stack top in a Right Sum Type.

BranchMatch(offset): Evaluates a Sum Type. Proceeds on Left, or jumps offset instructions forward on Right.

🚀 Getting Started
Ensure you have Rust and Cargo installed, then run the Autarky Bootstrapper against the target source file:

Bash
cargo run --release -- --file main.aut
Example Execution Pipeline

Plaintext
========================================
🐍 Autarky Compiler Bootstrapper v1.1.0
========================================
✅ Type Check Passed (Memory Safety Guaranteed)
✅ Proof Erasure Complete
✅ Bytecode Generated
----------------------------------------
🚀 Executing inside Autarky VM...
✅ Execution Finished Successfully!