# 🐍 Autarky Compiler

Autarky is a purely functional, linearly typed programming language and self-hosting compiler. It is built on a custom Lambda Calculus foundation that guarantees memory safety without the need for a Garbage Collector.

## 🏗️ Architecture

The project is divided into two distinct phases:

1. **The Rust Bootstrapper:** A minimal execution engine written in Rust. It includes a basic Lexer, AST Parser, Linear Type Checker, Bytecode Generator, and a strict Stack-Based Virtual Machine. 
2. **The Self-Hosted Compiler:** The actual compiler pipeline written entirely in Autarky (`.aut` files). The bootstrapper is used to execute these files until Autarky can compile itself.

## ✨ Core Features (Bootstrapper)

* **Linear Types:** Enforces strict memory ownership. Variables must be used exactly once, eliminating memory leaks and dangling pointers at compile time.
* **No Garbage Collection:** Memory is deterministically freed (`Free` instruction) via linear proofs.
* **Recursive Sum Types:** Full support for dynamically allocated, infinitely recursive data structures (e.g., Linked Lists, Trees) using `Fix`, `Fold`, and `Unfold`.
* **OS Interoperability:** Ability to natively read file streams into linear memory buffers.
* **Custom Virtual Machine:** A high-performance, stack-based VM with strict runtime memory and branch validation.

## 🚀 Project Status

**Stage 1 Completed:** The Rust bootstrapper is fully operational. We have successfully transitioned into the self-hosting phase.

### Self-Hosting Milestones:
- [x] **Lexical Scanner:** Reads a raw byte stream from linear memory.
- [x] **Whitespace Filter:** Purely functional stream filtering to eradicate spaces and newlines.
- [x] **Word Accumulator:** Recursively folds characters into distinct `List Int` word buffers.
- [x] **Keyword Classifier:** Structurally evaluates word buffers using a purely functional `string_eq` algorithm to emit strongly typed compiler Tokens.
- [ ] **Parser (In Progress):** Constructing the recursive Abstract Syntax Tree (AST).
- [ ] **Type Checker:** Linear theorem prover written in Autarky.
- [ ] **Code Generator:** Bytecode emission.

## 💻 Usage

To execute an Autarky script through the bootstrapper's Virtual Machine:

```bash
cargo run --release -- --file main.aut
🧠 Example: Purely Functional String Equivalence
Autarky does not rely on built-in string primitives. String equivalence is calculated by mathematically walking down two dynamically allocated recursive lists and verifying them byte-by-byte:

Plaintext
( fix
    ( \seq : Pi w1 : Word . Pi w2 : Word . Bool .
        \w1 : Word . \w2 : Word .
          match (unfold w1) with
            Left empty1 =>
              match (unfold w2) with
                Left empty2 => True
              | Right node2 => False
          | Right node1 =>
              unpack node1 into v1, r1 in
                match (unfold w2) with
                  Left empty2 => False
                | Right node2 =>
                    unpack node2 into v2, r2 in
                      if (v1 == v2) then ( (seq r1) r2 ) else False
    )
)