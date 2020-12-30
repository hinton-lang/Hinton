# The Hinton Language 🔮
This is a simple interpreter written in Java for a functional toy language called Hinton. The project is an extension of the code found in the book [Crafting Interpreters](https://craftinginterpreters.com/) by Bob Nystrom.


## Installing
The source code can be downloaded an placed inside an IntelliJ project. That is, inside
`[IntelliJ Project] > src > org > hinton_lang`.

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
/**
 * This is a multiline comment
 * The following is a recursive function
 */
func factorial(n = 0) { // default parameters
    if (n <= 1) return 1;
    return n * factorial(n - 1);
}

// This is a constant
const welcome = "Welcome to the program\n";

// Printing to the console
print(welcome)

// This is a variable.
// The `input(message: String)` is a function to
// obtain user input from the console.
// The `int(x)` is a function to convert the given
// argument to an integer.
let x = int(input("Compute the factorial of: "));
print(factorial(n = x)) // named arguments

// While loops
while(true) {
    let again = input("Compute another? (y/n): ");
    
    if (again equals "y") {
        let x = int(input("Compute the factorial of: "));
        print(factorial(x))
        continue;
    }

    if (again equals "n") break;
}

// This is a dictionary
let myDict = {
    a: 3,
    b: "hello dictionary!!",
    c: {
        arr: [1, 2, 3],
        pow: fn(a, b) -> { // Lambda expression
            return a ** b;
        }
    },
    d: null
}

print(myDict)
print(type(myDict))
print(myDict.c.pow(2, 3))

// The range operator
let r = 100..1;
print(r)
```
 
At the moment, the Interpreter can accurately identify and label different tokens inside a `.ht` file. Some of the tokens it can identify include, but are not limited to:
  - Keywords: `let`, `const`, `func`, `if`, `else`, + more.
  - Static Types: `String`, `Int`, `Real`, `void`, `None`, + more.
  - Literals: `"String Sequences"`, `2342`, `3.1242`, `true`, `false`, `null`, + more.
  - Arithmetic Operators: `+`, `-`, `*`, `/`, `**`, `%`, and `mod`.
  - Logical Operators: `<`, `>`, `==`, `equals`, `!`, `not`, + more.
  - Delimiters & Separators: `()`, `,`, `{}`, `:`, `[]`, `.`, and `;`

** Visit the [Tokens Folder](https://github.com/faustotnc/Hinton-Lang/tree/master/Tokens) for a complete list of tokens.
  
It is able to parse complex grammatical rules as specified inside the `grammar.cfg` file in order to generate an Abstract Syntax Tree: Some of the rule include, but are not limited to:
  - Expressions following a certain order of precedence.
  - Terminals that evaluate to a specific value.
  - Print statements.
  - Variable and Constant declarations.
  - Conditional statements.
  - Looping statements.
  - Function declarations.
  - Lambda expressions.

** Visit the [Parser Folder](https://github.com/faustotnc/Hinton-Lang/tree/master/Parser) for a complete list of grammar rules.

Together with the `RuntimeLib`, which provides a collection of built-in functions, Hinton is able to interpret full programs.

** Visit the [RuntimeLib Folder](https://github.com/faustotnc/Hinton-Lang/tree/master/RuntimeLib) for a list of available built-in functions.
