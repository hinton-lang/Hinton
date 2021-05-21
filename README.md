# The Hinton Language

![Hinton Logo](Assets/Logos/Logo-wide.png)

This is a simple, stack-based, byte-code interpreter written in Rust for a programming language called Hinton. The project is an extension of the code found in the book [Crafting Interpreters](https://craftinginterpreters.com/) by Bob Nystrom.

## Features
Though this interpreter is based on the Crafting Interpreters book, it implements many things differently. Here are some of the differences between Hinton and Lox, as well as some of the features Hinton has that Lox does not have:

* Hinton source code is first parsed into an AST, then compiled to bytecode, then interpreted by the VM. This is because traversing the AST is easier and allows for easier optimization of the code (optimization strategies will be added later).

* Hinton stores all variables on the stack. There is no concept of a "global" variable in Hinton. Variables will be stored this way unless having a different strategy for global variables is needed in the future.

* Hinton has extra built-in data structures like `Arrays`, `Tuples`, `Iterators`, and `Ranges`.

* Hinton has extra built-in functions like:
    * `print(...)`: To print to the console,
    * `input(...)`: To receive user input,
    * `iter(...)`: To convert an object to an iterator, and
    * `next(...)`: To get the next item in an iterator,

* Hinton has support for more operators like `%`, `**`, `<<`, `>>`, `^`, `&`, `~`, nullish coalescing (`??`), ternary conditionals (`? :`), advanced reassignment (`+=`, `**=`, `%=`, etc...), plus binary, hexadecimal, and octal numbers.

* Hinton supports the `break` statement in loops. The `continue` statement is coming soon.

** Hinton is a work-in-progress, and many other features are yet to come. To see a list of the features currently being worked on, visit the [Planned Features](https://github.com/hinton-lang/Hinton/projects/1) page. For a list of features without a near-by implementation date, visit the [Missing Features](#missing-features) section of this README.

<sub>**NOTE:** All highlighted Hinton code in this README is being highlighted by GitHub's Swift syntax highlighter for illustration purposes only. The code is not actual Swift code, and GitHub does not provide a syntax Highlight for Hinton code.</sub>

## Hello World
To run a "hello world" program, simply download and unzip this repo, create a file named `hello-world.ht` on your computer, and place the following inside it:
```swift
print("Hello Hinton!");
```

Then, in a terminal window, navigate to the unzipped folder and run:
```
cargo run '</path/to/hello-world.ht>'
```
NOTE: You must install Rust to run Hinton. I know, I know, but Hinton isn't a full programming language yet, so this will have to do.

## Advanced Programs
### The Classic Fibonacci Number Calculator:
On average, running with release mode, the algorithm takes ~361ms to compute the `fib(25)` on my MacBook Pro 2019 with 16GB of RAM running MacOS Big Sur. It is not particularly fast, but it is not particularly slow either.
```swift
func fib(n := 0) {
    if (n < 2) return n;
    return fib(n - 2) + fib(n - 1);
}

let 25th_fib = fib(25);

print(25th_fib);
```
### A Little Greeting Loop
```swift
while true {
    let name = input("Who are we greeting? ");
    print("Hello there, " + name + "!\n");

    if input("Greet again? (y/n): ") != "y" {
        break;
    }
}
```

## Lifecycle of a Hinton Program
Hinton programs get executed in three separate steps: parsing, compiling and executing.
* **Parsing**: The parser finds tokens in the source code and groups those tokens into `ASTNode`s to create an Abstract Syntax Tree (AST) of the source code. This syntax tree can be analyzed for code optimizations like Constant Folding and Loop Unrolling (optimizations coming in the future).

* **Compiling**: The compiler takes an AST, walks the tree, and generates bytecode instructions as it goes. It creates a `SymbolTable` to keep track of declarations made in local scopes and enforces lexical scoping at compile time so that the VM does not have to perform checks for the existence of variables at runtime. (You can also [print the bytecode](#printing-bytecode) of a program).

* **Executing**: The execution step involves the creation of a stack-based Virtual Machine (VM). The VM takes a chunk of bytecode and executes each instruction in the chunk one at a time, popping and pushing values onto a Value stack for variables and temporary objects. It also pops and pushes frames onto a Frames stack for recursive function calls.

Because Hinton programs are executed in these three separate steps, Hinton takes longer to start executing programs. To see the time each step takes to execute, run the programs with the `bench_time` Cargo feature flag:
```
cargo run --features bench_time </path/to/program.ht>
```
This should print a message similar to the following after the program finishes executing:
```
======= ⚠️  Execution Results ⚠️  =======
Parse Time:     <parsing time in ms>
Compile Time:   <compile time in ms>
Run Time:       <execution time in ms>
=======================================
```

When running Hinton programs with this flag, you will notice that the parser takes a longer time to finish relative to the other components. There is much room for improvement in all components, but especially in the parser.

## Printing Bytecode
To print the generated bytecode for a program, run the file with the `show_bytecode` Cargo feature flag:
```
cargo run --features show_bytecode </path/to/program.ht>
```
For example, running the following program from a file called './test.ht' results in the following bytecode:

**Program**
```swift
let x = 0;

while x <= 10 {
    print("X equals " + x);
    x += 1;
}
```
**Bytecode**
```
==== Script: './test.ht' ====
001     0000 0x11 – LOAD_IMM_0I                
003     0001 0x24 – GET_VAR                    1
 |      0003 0x27 – LOAD_IMM_N                 10
 |      0005 0x0F – LESS_THAN_EQ               
 |      0006 0x38 – POP_JUMP_IF_FALSE          29 (add 20 to IP)
004     0009 0x26 – LOAD_CONSTANT              0 -> (print)
 |      0011 0x17 – LOAD_NATIVE                
 |      0012 0x26 – LOAD_CONSTANT              1 -> (X equals )
 |      0014 0x24 – GET_VAR                    1
 |      0016 0x00 – ADD                        
 |      0017 0x23 – FUNC_CALL                  1
003     0019 0x20 – POP_STACK_1                
005     0020 0x24 – GET_VAR                    1
 |      0022 0x13 – LOAD_IMM_1I                
 |      0023 0x00 – ADD                        
 |      0024 0x2D – SET_VAR                    1
004     0026 0x20 – POP_STACK_1                
003     0027 0x28 – LOOP_JUMP                  1 (sub 28 from IP)
000     0029 0x15 – LOAD_IMM_NULL              
 |      0030 0x2C – RETURN                     1
```

And to see the raw bytes, run the file with the `show_raw_bytecode` flag:
```
cargo run --features show_raw_bytecode </path/to/program.ht>
```
Which, for the above program, results in the following chunk of bytes:
```
==== Script: './test.ht' ====
0x11 0x24 0x01 0x27 0x0A 0x0F 0x38 0x00 
0x14 0x26 0x00 0x17 0x26 0x01 0x24 0x01 
0x00 0x23 0x01 0x20 0x24 0x01 0x13 0x00 
0x2D 0x01 0x20 0x28 0x1C 0x15 0x2C 0x01 
```

## Missing Features
As mentioned before, Hinton is a work in progress. And as a matter of fact, I (Fausto German) am only in [Chapter #25](https://craftinginterpreters.com/closures.html) of the Crafting Interpreters book. After adding functions and calls, the chapters of the book that follow become a lot more complex and harder to translate to Rust code that can work with the current implementation of Hinton. Because of this, I am trying to add as many smaller features as possible and improve the three components of the interpreter before moving on. However, I cannot assure you that those remaining features will be added anytime soon. Here is a list of features that Hinton is currently missing and that may take longer to be added:
* Function Closures
    * Accessing variables outside of functions.
    * Lambda expressions.
* Garbage Collection
* Classes & Inheritance
* Importing Modules

Because accessing class members is similar to accessing dictionary members, Hinton is also missing:
* Dictionaries
* Enums
* Calling functions from native objects like `Array.len()`

**NOTE:** Because I am creating Hinton to learn about programming language design and compiler/interpreter implementation, I will not be accepting any pull requests that add any of these *missing features* to Hinton (I want to learn how to do it myself). However, any other contributions that improve the current state of the interpreter are welcomed. For a list of planned features or issues to which you can contribute visit the [Planned Features](https://github.com/hinton-lang/Hinton/projects/1) or [Issues](https://github.com/hinton-lang/Hinton/issues) page.

