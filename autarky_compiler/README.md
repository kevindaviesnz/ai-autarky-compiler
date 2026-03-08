# Project Ouroboros: The Autarky Compiler (Stage 0)

**Autarky** is a novel systems programming language designed at the intersection of Formal Verification and Compiler Design. Its ultimate goal is to guarantee memory safety, absence of data races, and exact resource management *without* a garbage collector and with zero runtime overhead.

This repository contains the **Stage 0 Bootstrapper**: a vertical-slice MVP compiler written in Rust. It utilizes a custom theorem prover based on **Dependent Linear Type Theory** to statically verify memory lifetimes before erasing compile-time proofs and executing the logic on a custom Stack-Based Virtual Machine.

## 🚀 Core Features

* **Calculus of Inductive Constructions (CIC) Prover:** Replaces standard semantic analysis. The type-checker algorithmically threads a linear context to prove memory safety at compile time.
* **Fractional Permissions (Separation Logic):** Safely handles cyclic memory dependencies. `Full` linear permissions can be fractured into multiple `1/2` read-only aliases (`split`) and fused back together (`merge`).
* **Linear Control Flow:** Implements `if/then/else` branching that mathematically guarantees identical resource consumption across diverging execution paths to prevent memory leaks.
* **Data Structures:** Linear Tensor Products (`Pair`) that force safe unpacking and consumption of grouped memory.
* **Recursion:** A type-verified fixed-point combinator (`fix`) that lazy-unrolls at runtime, allowing Turing-complete loops without breaking linear contexts.
* **Explicit Deallocation:** A native `free` keyword that safely destroys `Permission::Full` linear resources from the VM heap.
* **Verified Proof Erasure:** Type annotations, universes, and linear constraints are completely stripped during IR lowering. The emitted bytecode is pure, untyped logic with zero safety-check overhead.

## 🧠 The Architecture Pipeline

1.  **Frontend:** Lexes and parses `.aut` string syntax into a formal Abstract Syntax Tree (AST).
2.  **Theorem Prover (Middle-end):** Evaluates Universe Hierarchies (`Type_0`, `Type_1`) to prevent Girard's Paradox, and enforces strict substructural typing (Linear/Affine) to prevent use-after-free and memory leaks.
3.  **Erasure (IR Generation):** Lowers the verified AST into an untyped Intermediate Representation.
4.  **Backend:** Emits linear stack-machine `Instruction` bytecode.
5.  **Execution:** Runs the bytecode natively in the Autarky VM.

## 🛠️ Getting Started

### Prerequisites
* [Rust toolchain](https://rustup.rs/) (Cargo)

### Installation & Execution
Clone the repository and run the compiler against a target `.aut` file:

```bash
cargo build --release
cargo run --release -- --file main.aut
📝 Language Syntax Examples
Autarky relies heavily on lambda calculus syntax (\param : Type . body) and explicit linear context management.

1. Recursive Mathematical Loops
Evaluates to 15 using the fixed-point combinator.

Plaintext
(
  (fix \rec : Pi n : Int . Int . \n : Int .
     if (n == 0) then
        0
     else
        (n + (rec (n - 1)))
  )
  5
)
2. Tensor Products & Memory Deallocation
Demonstrates safely packing a linear pointer into a Pair, unpacking it, and explicitly freeing the memory.

Plaintext
( 
  ( \p : Pair Int (Lin Type_1) . 
      unpack p into num, ptr in 
        (free ptr) 
  ) 
  (mkpair 42 memory_ptr) 
)
🗺️ Roadmap to Stage 1 (Self-Hosting)
With the Stage 0 Bootstrapper feature-complete, the next phase is Stage 1: designing the architecture to rewrite the Parser and the Theorem Prover inside Autarky itself.

Project Ouroboros is an exploration of the bleeding edge of Programming Language Theory.


***

Once you've saved that `README.md`, run these commands to immortalize the final Stage 0 build:

```bash
git add .
git commit -m "feat: complete Stage 0 with recursion (fix), pairs, and v0.10.0 README"
git push