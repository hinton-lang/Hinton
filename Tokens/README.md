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

## Logical Operators
- Less than: `<`, `<=`
- Greater than: `>`, `>=`
- Equals: `==`, `equals`
- Not: `!`, `not`, `!=`
- And: `&&`, `and`
- Or: `||`, `or`
- Type Checking: `is`

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
- Declarations: `let`, `const`, `func`, `class`
- Static Types: `Int`, `NInt`, `Real`, `NReal`, `String`, `NString`, `Bool`, `NBool`,
`Dict`, `NDict`, `Set`, `NSet`, `Function`, `NFunction`, `void`, `any`, `None`,
- Logic Flow: `if`, `elif`, `else`, `for`, `while`