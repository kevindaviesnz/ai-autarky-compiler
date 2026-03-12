```markdown
# Autarky Quick-Reference Cheat Sheet

This cheat sheet covers the core syntax, types, and keywords for the Autarky programming language. Remember the Golden Rule: **Every bound variable must be consumed exactly once.**

### 1. Types
Autarky's type system is strict to enforce memory safety.

| Type | Description | Example |
| :--- | :--- | :--- |
| `Int` | A standard integer value. | `42` |
| `Pair A B` | A tuple containing two values of types `A` and `B`. | `Pair Int Int` |
| `Either A B` | A sum type that is *either* a Left `A` or a Right `B`. | `Either Int Int` |
| `A -> B` | A function taking type `A` and returning type `B`. | `Int -> Int` |

### 2. Core Syntax & Functions
Everything is built around single-argument functions (lambdas).

| Operation | Syntax | Description |
| :--- | :--- | :--- |
| **Function Definition** | `\var : Type . body` | Defines an anonymous function. The `.` separates the signature from the body. |
| **Function Application** | `( \x : Int . x ) 10` | Executes a function. Wrap the function in parentheses and put the argument next to it. |
| **Addition** | `a + b` | Consumes `a` and `b`, returns their sum. |
| **Subtraction** | `a - b` | Consumes `a` and `b`, returns their difference. |

### 3. Working with Pairs (Tuples)
Pairs bundle data, but you must unpack them to use the values.

| Operation | Syntax | Description |
| :--- | :--- | :--- |
| **Make Pair** | `mkpair x y` | Creates a `Pair` containing `x` and `y`. Consumes both `x` and `y`. |
| **Unpack Pair** | `unpack p into a, b in body` | Destructures pair `p` into `a` and `b`. You must consume exactly both `a` and `b` in the `body`. |

### 4. Branching and Sum Types (Either)
Handling distinct timelines and conditional logic.

| Operation | Syntax | Description |
| :--- | :--- | :--- |
| **Make Left** | `Left x` | Wraps `x` in the Left variant of an `Either` type. |
| **Make Right** | `Right y` | Wraps `y` in the Right variant of an `Either` type. |
| **Match / Branch** | `match e with Left a => ... \| Right b => ...` | Evaluates `e`. If `Left`, binds inner value to `a`. If `Right`, binds to `b`. Both branches must consume identical outside variables. |

---

### 🚨 Common Compiler Panics (The Scope Janitor)
* **"Unbound Variable"**: You tried to use a variable that isn't in your current scope, or you already consumed it earlier.
* **"Variable Leaked"**: You defined or unpacked a variable but forgot to use it before the function ended.
* **"Branch Mismatch"**: You used an outside variable in the `Left` branch of a match statement, but forgot to use it in the `Right` branch. Both timelines must be perfectly balanced.