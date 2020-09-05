# Keywords in Hinton Script

Keywords in Hinton Script are special words that have syntactical meaning based on grammar rules, and that cannot be used by the programmer as identifiers.

## Logic Flow and Logical Operators
- `if-elif-else`: Together, the `if`, `elif` and `else` statements define the logic flow statements of a program.
- `and`: Logical binary operator that returns true when both of the operands are true.
- `or`: Logical binary operator that returns true if at least one of the operands is true.
- `not`: Logical unary operator that negates the value of the operand.
- `equals`: Logical binary operator that returns true if both operands are equal.
- `is`: Type-checking logical binary operator that returns true if the type of the left-hand-side operand matches the type of the right-hand-side built-in type.

## Other Operators
- `mod`: Binary operator that returns the arithmetic modulus of the operands. `3 mod 2 // 1`

## Declarations
- `type`: Defines a custom type.
- `struct`: Defines the structure of a dictionary.
- `enum`: Defines an enumerable.
- `let`: Defines a mutable variable.
- `const`: Defines an immutable variable.
- `func`: Defines a function
- `genr`: Defines a generator function.
- `async`: Define an asynchronous function.

## Static Types
Hinton Script is a statically typed language, which means that declarations must be followed by a type specification. Once declared with a type, the identifier's type cannot be changed.

### Primitive types
- `Int`: Integers.
- `Real`: Real-valued numbers. Includes integers (with a .0 suffix).
- `String`: Double quoted strings.
- `Bool`: Booleans.
- `Dict`: Dictionaries.
- `Set`: Sets.
- `Function`: Functions.
- `Null`: Null value.
- `void`: Functions with empty return statement.
- `any`: Any type.

### Nullable Primitive types.
In Hinton Script, assigning a value of `Null` to an identifier that does not have `Null` in its set of possible types will result in a RuntimeError. To prevent errors, Hinton Script provides primitive nullable types, that is, identifiers that can be a primitive type or a null. The bellow types are equivalent to `PrimitiveType | Null`;
- `NInt`: Nullable integers.
- `NReal`: Nullable real-valued numbers. Includes integers (with a .0 suffix).
- `NString`: Nullable double quoted strings.
- `NBool`: Nullable booleans.
- `NDict`: Nullable dictionaries.
- `NSet`: Nullable sets.
- `NFunction`: Nullable functions.

## Object-Oriented Programming
- `class`: Defines a class.
- `self`: Internal class member accessor.
- `interface`: Defines a class interface.
- `implements`: Used to specify the interfaces a class is implementing.
- `extends`: Used to specify the parent class a child class inherits.
- `imports`: Used to import exported components from a program.
- `from`: In a `import-from` statement, defines the path of the exporting program.
- `public`: Method accessor modifier. Makes a member of a class accessible outside the class.
- `private`: Method accessor modifier. Makes a member of a class only available from within the class.
- `static`: Makes a member of a class static.
- `final`: Used to make a class member immutable.
- `method`: Defines a class method.
- `super()`: ...
- `init()`: Defines a class constructor.

## Looping
- `while`: While loops.
- `for`: Used in `for` loops, and `for-in` loops.
- `in`: Used in `for-in` loops.
- `repeat`: Used in `repeat-until` loops.
- `until`: Ued in `repeat-until` loops.
- `break`: Stops a loop from executing.
- `continue`: Re-starts the loop at the current statement.
- `await`: In the context of loops, `await` is used in a `for-await` statement to loop through the yielded values of an asynchronous generator function.

## Expressions
- `as`: Binary operator that typecasts the left-hand-side operand into the right-hand-side operand (type).
- `try`: Used in a `try-catch` statement to handle errors thrown by an expression.
- `catch`: Used in a `try-catch` statement to handle errors thrown by an expression.
- `true`: Boolean literal `1`.
- `false`: Boolean literal `0`.
- `null`: Boolean literal `null`.
- `throw`: Stops the execution of code and print an error message to the console.
- `throws`: Specifies that a method of functions could throw an error.

## Statements
- `return`: The return statement of a function or method.
