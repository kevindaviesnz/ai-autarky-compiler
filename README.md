# Autarky (Project Ouroboros)

Autarky is a statically typed, functional systems programming language built on a strict **Linear Type System**. 

It is designed for domains where correctness is critical and failure is catastrophic. By treating variables as physical resources that must be consumed exactly once, Autarky mathematically guarantees memory safety and eliminates the need for a Garbage Collector.



## ⚡ Core Philosophy

* **Zero Garbage Collection:** Memory is perfectly managed at compile-time. There are no runtime GC pauses, making execution 100% deterministic.
* **The "Scope Janitor":** If you forget to use a variable (memory leak) or try to use a variable twice (double-free/data race), the compiler will panic and refuse to build your code. 
* **Proof Erasure:** Once the compiler mathematically proves your program is safe, it strips all type metadata. The resulting bytecode runs with zero overhead.
* **Self-Hosting (The Singularity):** Autarky is capable of compiling itself. The compiler understands its own linear logic.

## 🚀 Quick Start

Ensure you have [Rust](https://www.rust-lang.org/) installed, then clone the repository and run the compiler:

```bash
git clone [https://github.com/yourusername/autarky.git](https://github.com/yourusername/autarky.git)
cd autarky
cargo build --release
To run an Autarky script through the built-in Virtual Machine:

Bash
cargo run --release -- --file examples/hello.aut
📖 Documentation
Whether you are writing your first linear program or looking to contribute to the compiler's core, start here:

The Language Tutorial: Learn how to think in linear types, unpack pairs, and manage state without mutating variables.

Syntax Cheat Sheet: A quick reference guide for Autarky's keywords and grammar.

Project FAQ: Answers to common questions about Substructural Logic, Branch Equivalence, and the "Ouroboros" cycle.

Contributor Guide: Read this before submitting a Pull Request. Learn how our AST is structured and how to interface with the Type Checker.

💻 Example: The Linear Timeline
In Autarky, variables cannot be duplicated or dropped. They must flow perfectly from creation to destruction.

Code snippet
// A function that buys a coffee. 
// It strictly consumes a Token (Int) and returns a Pair containing the Coffee and the Change.
\buy_coffee : (Int -> Pair Int Int) .

  // The user's wallet containing a single Token
  \my_token : Int . 

    // We must unpack the result and strictly consume BOTH the coffee and the change.
    unpack (buy_coffee my_token) into coffee, change in
      
      // We bundle them back up to satisfy the compiler's requirement that nothing is lost.
      mkpair coffee change
🗺️ Roadmap: The LLVM Backend
Autarky currently compiles to a custom bytecode executed by our own stack-based Virtual Machine.

Next Major Milestone: We are actively replacing the VM with a native LLVM Backend (via the inkwell crate). Because Autarky's linear types map flawlessly onto LLVM's Static Single Assignment (SSA) form, this upgrade will allow Autarky to compile directly into highly optimized x86_64 and ARM machine code. Check the issues tab if you want to help us bridge the FFI boundary!

Project Ouroboros — Digital Physics, enforced at compile-time.