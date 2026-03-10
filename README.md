# 🐍 Autarky Compiler

Autarky is a purely functional, linearly typed programming language and self-hosting compiler. It guarantees memory safety without a Garbage Collector by utilizing a strict linear logic foundation.

## 🏗️ Architecture

The system operates in two distinct layers:
1. **The Rust Bootstrapper:** A hardened execution engine. It handles linear type verification, bytecode generation, and stack-based VM execution.
2. **The Self-Hosted Compiler:** Written in `.aut` files. This layer is currently performing its own lexical analysis and parsing.

## ✨ Core Features

* **Linear Types:** Variables must be used exactly once. This is enforced by the Rust-based theorem prover, ensuring deterministic memory management.
* **Verified Front-End:** The compiler now successfully reads its own source from the OS, tokenizes it, and generates a unified Abstract Syntax Tree (AST).
* **Zero-Leak VM:** A custom stack machine that has been hardened against De Bruijn index offset errors, ensuring perfect variable scoping in deeply nested closures.

## 🚀 Project Status

**Stage 2 Completed:** The front-end pipeline is fully integrated.

### Self-Hosting Milestones:
- [x] **Lexical Scanner:** Byte-stream processing in linear memory.
- [x] **Keyword Classifier:** Functional string equivalence for tokenization.
- [x] **Recursive Descent Parser:** Transformation of token streams into a hierarchical AST.
- [x] **VM Stability:** Patched stack offset logic to support deep functional nesting.
- [ ] **Type Checker (Next Stage):** A linear theorem prover written in Autarky.
- [ ] **Code Generator:** Target bytecode emission.

## 💻 Usage

To execute the integrated front-end and generate the AST for the lexer:

```bash
cargo run --release -- --file main.aut

📜 License
This project is licensed under the MIT License - see the LICENSE file for details.