# The Tokens

> A lexical token or simply token is a string with an assigned and thus identified meaning. It is structured as a pair consisting of a token name and an optional token value. The token name is a category of lexical unit.
>
> [Wikipedia - Lexical analysis](https://en.wikipedia.org/wiki/Lexical_analysis#Token)

## Supported Tokens
The toy programming language supports the following tokens:

## Literals
- Strings: `"This is a string"`
- Integers: `2345`, `2_3_4_5`, etc...
- Reals: `.342`, `0.342`, `8.99`, `9.89_54`, etc...
- Booleans: `true`, `false`
- Others: `null`

## Arithmetic Operators
- Addition: `+`
- Subtraction: `-`
- Multiplication: `*`
- Division: `/` (both integer and real)
- Exponentiation: `**`
- Remainder: `%`, `mod`
<!-- - Range: `..` -->

## Logical Operators
- Less than: `<`, `<=`
- Greater than: `>`, `>=`
- Equals: `==`, `equals`
- Not: `!`, `not`, `!=`
- And: `&&`, `and`
- Or: `||`, `or`
<!-- - Type Checking: `is` -->
<!-- - Type Casting: `as` -->

## Bitwise Operators
- Right Shift: `>>`
- Left shift: `<<`
- AND: `&`
- OR: `|`
- NOT: `~`
- XOR: `^`

## Delimiters and Separators
`( )`, `;`, `{ }`, `,`, `[ ]`, `.`, `:`

## Keywords and Identifiers
- Declarations: `let`, `const`, `func`, `enum`, `struct`,
<!-- - Declarations: `let`, `const`, `func`, `class`, `enum`, `struct`, `interface` -->
- Static Types: `Int`, `Real`, `String`, `Char`, `Bool`, `Dict`, `Set`, `Function`, `Void`, `Any`, `Null`
- Control Flow: `if`, `else`, `for`, `while`, `return`, `break`, `continue`
<!-- - Logic Flow: `if`, `else`, `for`, `while`, `loop`, `break`, `continue` -->
- Modules and OOP: `import`
<!-- - Modules and OOP: `import`, `export`, `new`, `init`, `final`, `public`, `private`, `static`, `abstract`, `self`, `instanceof`, `implements`,
`extends`, `override`, `optional` -->
- Other: `fn`
<!-- - Other: `fn`, `async`, `await`, `yield` -->