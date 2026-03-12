# Project Ouroboros (Autarky) - Mini FAQ

**Q: What is a "Linear Type System" and why does it matter?**
Most languages either force you to manually manage memory (like C) or use a Garbage Collector to clean up after you (like Java or Python). Autarky uses a Substructural/Linear Type System. This means the compiler enforces a strict mathematical rule: **every variable must be consumed exactly once.** If you forget to use a variable (a memory leak) or try to use it twice (a double-free), the compiler panics and refuses to build. The safety is guaranteed mathematically at compile-time.



**Q: If there's no Garbage Collector, how does it run so fast?**
Autarky utilizes a concept called **Proof Erasure**. During the compilation pipeline, the "Scope Janitor" rigorously checks the Abstract Syntax Tree (AST) to ensure the 1:1 memory rules are followed. Once the compiler mathematically proves the program is safe, it completely discards the type constraints and generates raw, flattened bytecode. The Virtual Machine executes the code with zero runtime overhead.

**Q: What do you mean by "The Singularity" or "Self-Hosting"?**
Every programming language has to be written in *something*. We built the initial Autarky compiler in Rust. However, the ultimate test of a language's maturity is whether it can understand its own logic. "The Singularity" was the moment we fed the Autarky compiler's source code (written in Autarky) into itself, and it successfully parsed, type-checked, and generated native bytecode for its own operations.

**Q: How does it handle `if/else` or branching without breaking the memory rules?**
This was one of the hardest architectural challenges: **Branch Equivalence**. If execution splits into two timelines (e.g., a `match` statement), Autarky splits the memory context. If you consume a captured variable in the `Left` branch, you *must* also consume it in the `Right` branch. The compiler verifies that all possible execution paths result in the exact same memory state before merging them back together.



**Q: What's next for Autarky?**
Currently, Autarky compiles down to bytecode for a custom stack-based Virtual Machine. The next major frontier is building a native backend (leveraging LLVM) to translate that Intermediate Representation directly into x86 or ARM machine code for bare-metal performance.