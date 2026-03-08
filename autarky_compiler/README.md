# Project Ouroboros: The Autarky Compiler (Stage 0)

**Autarky** is a novel systems programming language designed at the intersection of Formal Verification and Compiler Design. Its ultimate goal is to guarantee memory safety, absence of data races, and exact resource management *without* a garbage collector and with zero runtime overhead.

This repository contains the **Stage 0 Bootstrapper**: a vertical-slice MVP compiler written in Rust. It utilizes a custom theorem prover based on **Dependent Linear Type Theory** to statically verify memory lifetimes before erasing compile-time proofs and executing the logic on a custom Stack-Based Virtual Machine.



## 🚀 Core Features

* **Calculus of Inductive Constructions (CIC) Prover:** Replaces standard semantic analysis. The type-checker algorithmically threads a linear context to prove memory safety at compile time.
* **Fractional Permissions (Separation Logic):** Safely handles cyclic memory dependencies. `Full` linear permissions can be fractured into multiple `1/2` read-only aliases (`split`) and fused back together (`merge`), proving at compile time that aliased pointers are never written to concurrently or double-freed.
* **Verified Proof Erasure:** Type annotations, universes, and linear constraints are completely stripped during Intermediate Representation (IR) lowering. The emitted bytecode is pure, untyped logic with zero safety-check overhead.
* **Lexical Closures:** Functions correctly capture and persist their defining lexical environments at runtime.
* **Stack-Based Virtual Machine:** A custom execution engine that evaluates the emitted linear bytecode.

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

1. Basic Arithmetic with Closures
Evaluates to Int(42).

Plaintext
( ( ( \x : Int . \y : Int . (x + y) ) 10 ) 32 )
2. Fractional Permissions (The Cyclic Memory Solution)
Demonstrates taking a linear memory pointer, splitting it into two read-only aliases, merging them back to prove 1/2+1/2=1, and returning the fully reconstructed pointer.

Plaintext
( \x : Lin Type_1 . 
    split x into ptr1, ptr2 in 
        merge ptr1 and ptr2 into full_ptr in 
            full_ptr 
memory_ptr )
🗺️ Roadmap to Stage 1 (Self-Hosting)
To escape Stage 0 and allow Autarky to compile itself, the following features are actively being developed:

[ ] Control Flow: Implementation of if / then / else branching, requiring the prover to enforce identical resource consumption across diverging execution paths.

[ ] Explicit Deallocation: Introduction of the free keyword to safely destroy Permission::Full linear resources from the VM heap.

[ ] Unification Engine: Upgrading the naive context splitter to intelligently deduce linear resource routing during complex function applications.

[ ] Data Types: Sum types, product types, and inductive data structures (Lists, Trees).

Project Ouroboros is an exploration of the bleeding edge of Programming Language Theory.