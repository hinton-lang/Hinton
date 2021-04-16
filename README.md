# The Hinton Language

![Hinton Logo](Assets/Logos/Logo-wide.png)

This is a simple, stack-based, byte-code interpreter written in Rust for a programming language called Hinton. The project is an extension of the code found in the book [Crafting Interpreters](https://craftinginterpreters.com/) by Bob Nystrom.

## Hinton's Syntax
This is an example of what Hinton's future syntax would look like. Hinton should be a programming language that can be accessible by anyone, with any level of programming experience. Source code written in Hinton should be easy to read. In Hinton, explicitness is the key.

```javascript
/**
 * This is a multiline comment
 * The following is a recursive function
 */
func factorial(n: Int = 0): Int { // default parameters
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
print(factorial(n := x)) // named arguments

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
        },
        square: fn(a) -> a ** 2 // Short lambda expression
    },
    d: null
}

print(myDict)
print(type(myDict))
print(myDict.c.pow(2, 3))

// The range operator
let theRange = 100..1;

// indexed for-loops
for (let x = 0; x < theRange.length(); x++) {
    print(x)
}

// for-in loops
for (let item in theRange) {
    print(item)
}
```

## The current state of Hinton
The project is still in the "pre-development" stages. This means that anything about the language can change as more experience is acquired.

## To-do List
This to-do list only applies to the Rust implementation of the Hinton interpreter. For the Java implementation, which has more implemented features, visit the [Hinton - Java Implementation](https://github.com/hinton-lang/Hinton/) branch.
- [ ] Parse & Execute Expressions.
    - [x] Arithmetic expressions.
    - [ ] Array literals.
    - [x] Assignment expressions.
    - [x] Binary (0b), Hexadecimal (0x), and Octal (0o) numbers
    - [ ] Elvis expressions.
    - [ ] Function calls.
    - [ ] Lambda expressions.
    - [x] Logical expressions.
    - [ ] Member access expressions.
    - [ ] Named arguments (using the `:=` operator).
    - [x] Nullish coalescing expressions.
    - [ ] Nullish coalescing assignment expressions.
    - [x] Range expressions.
    - [x] String literals.
    - [x] Ternary Conditional expressions.
- [ ] Parse & Execute Statements.
    - [ ] Class declarations.
    - [ ] Constant declarations.
    - [ ] Control flow statements.
    - [ ] Dictionary declarations.
    - [ ] Enum declarations.
    - [ ] Function declarations.
    - [ ] Looping statements.
    - [ ] Named parameters.
    - [x] Variable declarations.
- [ ] Define Built-In Types as Objects
    - [ ] Array objects.
    - [ ] Boolean objects.
    - [ ] Dictionary objects.
    - [ ] Enum objects.
    - [ ] Function objects.
    - [ ] Integer objects.
    - [ ] Real objects.
    - [ ] String objects.
- [ ] Create a Runtime Standard Library.
- [ ] Add Static Typing.
- [ ] Create 'Cosmo'. Hinton's official CLI.
