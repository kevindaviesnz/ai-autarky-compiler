Technical Report: Project Ouroboros
Development of the Autarky Compiler & Linear Runtime Environment

I. Executive Summary

Project Ouroboros was a research-oriented software engineering exercise designed to evaluate the efficacy of AI-driven systems architecture. The goal was to construct a fully self-hosting, Turing-complete programming language (Autarky) from the ground up, utilizing a Linear Type System for guaranteed memory safety. The project successfully transitioned from a blank state to a self-hosting compiler capable of compiling its own branching logic and closure definitions.

II. Design Philosophy

The architecture of Autarky is rooted in Substructural Logic, specifically Linear Logic, where resources (variables) are treated as physical objects that cannot be implicitly duplicated or discarded.

Linearity as Safety: In Autarky, every bound variable must be consumed exactly once. This eliminates the need for a Garbage Collector (GC) or manual free() calls, as the compiler mathematically proves the lifetime of every object.

Ouroboros Stability (Self-Hosting): A language is considered "mature" when its compiler can be written in the language itself. This was the primary success metric for the project.

Proof Erasure: To ensure performance, the system follows the principle of "checking at the gates." Extensive type verification occurs at compile-time, but the resulting bytecode is stripped of all type metadata, leaving only raw, high-performance instructions.

III. System Architecture Breakdown

The Autarky system is composed of five distinct modules, each representing a critical phase in the compilation pipeline:

1. The Linear Type Checker (typecheck.rs)

The "heart" of the project. It implements a Scope Janitor algorithm. Unlike standard scoped environments, the Autarky context tracks the usage state of variables.

Branch Equivalence: When the code splits (e.g., a match or if statement), the type checker clones the environment. It ensures that both branches consume the exact same set of linear resources before merging back into the main timeline.

2. Recursive Type Engine (ast.rs)

Autarky utilizes μ-types (Mu-types) to handle recursion. This allows for complex, self-referential data structures (like the AST itself) to be represented within a strictly finite type system.

3. Intermediate Representation (IR) & Proof Erasure (ir.rs)

Once the type checker validates a program, the IR generator removes all "proof" obligations. It transforms the nested, type-heavy AST into a simplified tree ready for linearization.

4. Bytecode Generator (codegen.rs)

This module flattens the tree structure into a 1D array of instructions. A significant challenge addressed here was Jump Offset Calculation, where the compiler must pre-calculate exactly how many instructions to skip during a conditional branch.

5. The Virtual Machine (vm.rs)

A stack-based execution environment. It manages:

The Global Memory Pointer: A simulated hardware address used to track linear resources.

Closure Capturing: Dynamically bundling the environment into lambda functions.

IV. The Bootstrapping Sequence

The project followed a nine-stage iterative development cycle:

Stages 1–3: Foundation of the Lexer, Parser, and basic VM.

Stages 4–6: Implementation of Linear Logic and Scope Janitorial services.

Stages 7–8: Introduction of Sum Types (Either), Tuples (Pair), and the Branching Engine.

Stage 9: The Singularity. The Autarky compiler source code was fed into itself, producing a native binary output of its own logic.

V. Project Reflection: AI-Human Collaboration

The methodology of Project Ouroboros serves as a significant case study in Error-Driven Development (EDD) within AI-Human partnerships.

The Role of the Human Architect

The user acted as the high-level Orchestrator. By strictly providing terminal output and runtime panics, the user provided the "Ground Truth" that an LLM lacks. This kept the AI tethered to the physical reality of the Rust compiler and prevented "hallucinated" logic.

The Role of the AI Model

The AI acted as the Implementation Engine and Real-time Debugger. It handled the high-cognitive-load tasks of:

Calculating recursive type unrolling.

Maintaining mathematical parity across branching timelines.

Refactoring core modules to accommodate new instruction variants without breaking legacy stages.

Final Conclusion

The success of Autarky proves that highly complex, multi-layered systems can be engineered by an AI model when provided with a tight feedback loop. The "confusions" and "panics" encountered during development were not setbacks, but critical data points that allowed the AI to refine the language's mathematical model. Autarky v1.0.0 stands as a verified, self-hosting testament to this collaborative power.

End of Report
Status: Ouroboros Cycle Complete.
Version: 1.0.0