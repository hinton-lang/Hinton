# The Hinton Language

![Hinton Logo](Assets/Logos/Logo-wide.png)

This is a stack-based, multi-pass, bytecode interpreter written in Rust for a programming language called Hinton. The project is an extension of the code found in the book [Crafting Interpreters](https://craftinginterpreters.com/) by Bob Nystrom.

## Features
Though this interpreter is based on the Crafting Interpreters book, it implements many things differently. Here are some of the differences between Hinton and Lox, as well as some of the features Hinton has that Lox does not have:

* Hinton source code is first parsed into an Abstract Syntax Tree (AST), then compiled to bytecode, then interpreted by the VM. This is because traversing the AST allows for easier bytecode generation and optimization (optimization strategies will be added later).

* Hinton has extra built-in data structures like `Arrays`, `Tuples`, `Iterators`, and `Ranges`.

* Hinton has extra built-in functions like:
    * `print(...)`: To print to the console,
    * `input(...)`: To receive user input,
    * `iter(...)`: To convert an object to an iterator,
    * `next(...)`: To get the next item in an iterator,
    * `assert(...)`: To test that an expression is truthy,
    * `assert_eq(...)`: To test that two expressions are equal, and
    * `assert_ne(...)`: To test that two expressions are not equal,

* Hinton has support for more operators like `%`, `**`, `<<`, `>>`, `^`, `&`, `~`, nullish coalescing (`??`), ternary conditionals (`? :`), advanced reassignment (`+=`, `**=`, `%=`, etc...), plus binary, hexadecimal, and octal numbers.

* Hinton supports the `break` statement in loops. The `continue` statement is coming soon.

* Hinton supports the "long" version of almost all instructions that have an argument. For example, while the `DEFINE_GLOBAL` instruction takes the next byte as its operand (only allowing 255 global variables to be declared), the `DEFINE_GLOBAL_LONG` instruction takes the next two bytes as its operand (allowing up to 65,536 global variables to be declared).

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
On average, running with release mode, the algorithm takes ~455ms to compute the `fib(25)` on my MacBook Pro 2019 with 16GB of RAM running MacOS Big Sur. For comparison, a similar program in Python takes ~24ms. Not very fast 😕.
```swift
func fib(n := 0) {
    if (n < 2) return n;
    return fib(n - 2) + fib(n - 1);
}

let the_25th_fib = fib(25);

print(the_25th_fib);
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

* **Executing**: The execution step involves the creation of a stack-based Virtual Machine (VM). The VM takes a chunk of bytecode and executes one instruction in the chunk at a time. It works by pushing and popping objects onto an Object stack where it stores local variables and temporary objects. It also has a Frames stack, where it pushes and pops function call frames.

Because Hinton programs are executed in these three separate steps, they takes longer to start execution. To see the time each step takes to execute, run the programs with the `bench_time` Cargo feature flag:
```
cargo run --features bench_time </path/to/program.ht>
```
This should print a message similar to the following after the program finishes executing:
```
======= ⚠️  Execution Results ⚠️  =======
Parse Time:     <parsing time>
Compile Time:   <compile time>
Run Time:       <execution time>
=======================================
```

## Printing Bytecode
To print the generated bytecode for a program, run the file with the `show_bytecode` Cargo feature flag:
```
cargo run --features show_bytecode </path/to/program.ht>
```
For example, running the following program from a file called `./test.ht` results in the following bytecode:

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
==== <File '/path/to/file.ht'> ====
001     0000 0x11 – LOAD_IMM_0I                
 |      0001 0x2E – DEFINE_GLOBAL              0 -> 'x'
003     0003 0x2F – GET_GLOBAL                 0 -> 'x'
 |      0005 0x27 – LOAD_IMM_N                 10
 |      0007 0x0F – LESS_THAN_EQ               
 |      0008 0x3B – POP_JUMP_IF_FALSE          31 (add 20 to IP)
004     0011 0x26 – LOAD_CONSTANT              1 -> (print)
 |      0013 0x17 – LOAD_NATIVE                
 |      0014 0x26 – LOAD_CONSTANT              2 -> (X equals )
 |      0016 0x2F – GET_GLOBAL                 0 -> 'x'
 |      0018 0x00 – ADD                        
 |      0019 0x23 – FUNC_CALL                  1
003     0021 0x20 – POP_STACK_TOP              
005     0022 0x2F – GET_GLOBAL                 0 -> 'x'
 |      0024 0x13 – LOAD_IMM_1I                
 |      0025 0x00 – ADD                        
 |      0026 0x30 – SET_GLOBAL                 0 -> 'x'
004     0028 0x20 – POP_STACK_TOP              
003     0029 0x28 – LOOP_JUMP                  3 (sub 28 from IP)
000     0031 0x15 – LOAD_IMM_NULL              
 |      0032 0x2C – RETURN                     0
```

And to see the raw bytes, run the file with the `show_raw_bytecode` flag:
```
cargo run --features show_raw_bytecode </path/to/program.ht>
```
Which, for the above program, results in the following chunk of bytes:
```
==== <File '/path/to/file.ht'> ====
0x11 0x2E 0x00 0x2F 0x00 0x27 0x0A 0x0F 
0x3B 0x00 0x14 0x26 0x01 0x17 0x26 0x02 
0x2F 0x00 0x00 0x23 0x01 0x20 0x2F 0x00 
0x13 0x00 0x30 0x00 0x20 0x28 0x1C 0x15 
0x2C 0x00 

Chunk Size: 34
================
```

## Missing Features
As mentioned before, Hinton is a work in progress. And as a matter of fact, I am only in [Chapter #25](https://craftinginterpreters.com/closures.html) of the Crafting Interpreters book. After adding functions and calls, the chapters of the book that follow become a lot more complex and harder to translate to Rust code that can work with the current implementation of Hinton. Because of this, I am trying to add as many smaller features as possible and improve the three components of the interpreter before moving on. However, I cannot assure you that those remaining features will be added anytime soon. Here is a list of features that Hinton is currently missing and that may take longer to be added:
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

## Contributing
Because I am creating Hinton to learn about compiler/interpreter design and programming language implementation, I will not be accepting any pull requests that add any of the above *missing features* to Hinton (I want to learn how to do it myself). However, any other contributions that improve the current state of the interpreter are welcomed. For a list of planned features or issues to which you can contribute visit the [Planned Features](https://github.com/hinton-lang/Hinton/projects/1) or [Issues](https://github.com/hinton-lang/Hinton/issues) page.

