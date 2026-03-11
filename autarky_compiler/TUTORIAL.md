Welcome to Autarky, a programming language designed for absolute memory safety through the power of Linear Logic.

In most languages, the computer has to guess when you’re done with a piece of data (Garbage Collection) or you have to tell it manually (free or delete). In Autarky, the Linear Type Checker ensures that every variable you create is used exactly once. No leaks, no double-frees, just mathematical certainty.

1. The Core Philosophy: Use it or Lose it
The most important rule in Autarky is the 1:1 Rule.

Zero uses? The compiler panics (Memory Leak).

Two uses? The compiler panics (Use-after-free risk).

Your First Script

Code snippet
\x : Int . 
  x + 5
This is a simple function. It takes an integer x and adds 5 to it. Because x is used exactly once in the addition, the compiler is happy.

2. Working with Pairs (Tuples)
Autarky handles multiple pieces of data using Pairs. Because of linear rules, you must "unpack" a pair to get to the data inside.

Example: Swapping Values

Code snippet
\p : Pair Int Int .
  unpack p into a, b in
    mkpair b a
Here, we take a pair, split it into a and b, and then create a new pair with the values swapped.

3. The Choice: Either/Or (Sum Types)
Sometimes data can be one of two things. We use the Either type for this. To use an Either value, you must use a match statement.

Example: Safe Branching

Code snippet
\input : Either Int Int .
  match input with
    Left val => val + 1
  | Right val => val - 1
The Branching Rule: Autarky is strict. If you have a variable y sitting outside this match statement, and you use it in the Left branch, you must also use it in the Right branch. The "timeline" must remain balanced!

4. Building the Ouroboros (Closures)
You can create functions that "capture" variables from their surroundings.

Code snippet
\y : Int .
  (\x : Int . x + y)
In this example, the inner function captures y. Because y is now "inside" the inner function, it will be consumed whenever that function is finally called.

5. How to Run Your Code
Since Autarky is currently a self-hosted project, you run your scripts through the Bootstrapper:

Create a file named my_code.aut.

Run the compiler:
cargo run --release -- --file my_code.aut

What to expect:

✅ Type Check Passed: Your memory logic is mathematically sound.

✅ Proof Erasure Complete: The compiler has stripped away the "safety checks" to make the code fast.

🚀 Executing...: Your code runs inside the Autarky Virtual Machine.

Next Steps
Congratulations! You've just learned the basics of the world's most disciplined language.