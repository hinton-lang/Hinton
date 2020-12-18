# The Hinton Language 🔮
This is a simple interpreter written in Java for a functional toy language called Hinton. The code is an extension of the code found in the book [Crafting Interpreters](https://craftinginterpreters.com/) by Bob Nystrom.


## Installing
The source code can be downloaded an placed inside an IntelliJ Idea project. That is, inside `IntelliJ Project > src > org > hinton_lang`.

## Hello World
To run the interpreter, run the program with the following commands:

`run [permissions]? [program filepath] [program flags]?`

For example, to run the `test.ht` file provided in this repository, run the project with the following command:

`run --allow-input ./src/org/hinton_lang/test.ht`

A program can be as simple as printing an expression:
```swift
print("Hello World!!")
```

## Basic Syntax
```swift
// This is a comment
let myVar = "this is a variable";
const myConst = 3.14;

/**
* This us a block comment.
* And the code bellow is a function.
*/
func thisIsAFunction(x) {
    return x ** 2;
}

print(thisIsAFunction(4)) // 16
```
 
At the moment, the Interpreter can accurately identify and label different tokens inside a `.ht` file. Some of the tokens it can identify include, but are not limited to:
  - Keywords: `let`, `const`, `func`, `if`, `else`, + more.
  - Static Types: `String`, `Int`, `Real`, `void`, `None`, + more.
  - Literals: `"String Sequences"`, `2342`, `3.1242`, `true`, `false`, `none`, + more.
  - Arithmetic Operators: `+`, `-`, `*`, `/`, `**`, `%`, and `mod`.
  - Logical Operators: `<`, `>`, `==`, `equals`, `!`, `not`, + more.
  - Delimiters & Separators: `()`, `,`, `{}`, `:`, `[]`, `.`, and `;`

Visit the [Tokens Folder](https://github.com/faustotnc/Hinton-Lang/tree/master/Tokens) for a complete list of tokens.
  
It is able to parse complex grammatical rules as specified inside the `grammar.cfg` file in order to generate an Abstract Syntax Tree: Some of the rule include, but are not limited to:
  - Expressions following a certain order of precedence.
  - Terminals that evaluate to a specific value.
  - Print statements.
  - Variable and Constant declarations.
  - Conditional statements.
  - Looping statements.
  - Function declarations.
  - Lambda expressions.

Visit the [Parser Folder](https://github.com/faustotnc/Hinton-Lang/tree/master/Parser) for a complete list of grammar rules.

Together with the `RuntimeLib` which provides a collection of built-in functions, Hinton is able to interpret full programs.

Visit the [RuntimeLib Folder](https://github.com/faustotnc/Hinton-Lang/tree/master/Parser) for a list of available built-in functions.
