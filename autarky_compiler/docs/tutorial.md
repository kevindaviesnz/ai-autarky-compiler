# Autarky 101: The Linear Programmer's Guide

Welcome to Autarky. If you are coming from languages like Python, JavaScript, or C++, you are about to experience a fundamental shift in how you think about memory and variables. 

Autarky does not have a Garbage Collector, nor does it require you to manually manage memory with `malloc` or `free`. Instead, it uses a **Linear Type System**. 

## The Golden Rule: Use It Exactly Once
In Autarky, variables are treated as physical resources. If you are handed a resource, you must consume it. You cannot ignore it, and you cannot use it twice.

If you violate this rule, the Autarky compiler will fail with a "Scope Janitor" panic before your code ever runs. This mathematically guarantees your program will never have memory leaks or use-after-free bugs.

## 1. Functions and Basic Arithmetic
Autarky is a functional language. Everything revolves around lambdas (anonymous functions). The syntax for a function is `\variable : Type . body`.

```autarky
// A function that takes an integer and adds 10
\x : Int . 
  x + 10
Notice that x is used exactly once. If we wrote \x : Int . 10, the compiler would panic because x was discarded. If we wrote \x : Int . x + x, the compiler would panic because x was used twice.

2. Working with Multiple Values (Pairs)
Because functions in Autarky mathematically take a single argument, we group multiple pieces of data together using a Pair.

To use the data inside a Pair, you must strictly "unpack" it. Unpacking consumes the Pair and gives you the two inner variables, which you must now also consume exactly once.

Code snippet
// A function that takes two integers and adds them together
\input : Pair Int Int .
  unpack input into a, b in
    a + b
You can create Pairs using the built-in mkpair instruction:

Code snippet
mkpair 10 20
3. Making Decisions (Sum Types)
Autarky handles conditional logic using the Either type, which represents a value that can be either a Left variant or a Right variant. You evaluate them using a match statement.

The Branch Equivalence Rule: The compiler splits the timeline at a match statement. If you capture an outside variable, you must use it in both the Left and Right branches to ensure the memory state is identical when the timelines merge.

Code snippet
// A function that processes an Either type
\coin_flip : Either Int Int .
  match coin_flip with
    Left heads_val => 
      heads_val + 100
  | Right tails_val => 
      tails_val - 50
4. Closures and State
Autarky supports closures—functions that return other functions and "capture" their environment. When a variable is captured by an inner function, its ownership is transferred to that function.

Code snippet
// A function returning a function (Currying)
\x : Int .
  (\y : Int . 
    x + y
  )
5. Applying Functions
To execute a function immediately, you wrap it in parentheses and place the argument next to it:

Code snippet
// Defines a function and immediately passes it the value 42
( \x : Int . x + 1 ) 42
Summary Checklist
Did I use every variable exactly once?

Did I unpack my Pairs?

Do my match branches use the exact same outside variables?

Did I remember that I cannot duplicate data without explicitly re-creating it?