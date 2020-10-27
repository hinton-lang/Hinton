# The Abstract Syntax Tree (AST)

> In computer science, an abstract syntax tree (AST), or just syntax tree, is a tree representation of the abstract syntactic structure of source code written in a programming language. Each node of the tree denotes a construct occurring in the source code.
>
> [Wikipedia - Abstract syntax tree](https://en.wikipedia.org/wiki/Abstract_syntax_tree)

In Graph Theory's terms, a tree is composed of nodes and leaves. Each tree has a top-most node called a root, and internal nodes that connect the root node to the leaves. The leaves are nodes that do not have any children nodes.

## The Root
The root of the generated syntax tree is a "Program". Ideally, any single file that the interpreter reads is a program whose children are the statements declared in that file. For example, for the following Toy program
```
let x: Int = 22;
print("The value is: ", x);
```
The generated AST should look like:
```
Program
|---- Assignment
|  |---- VarDeclaration
|  |  |---- Identifier: x
|  |  |---- TypeDef: Int
|  |---- IntLiteral: 22
|---- PrintStatement
|  |---- StringLiteral: "The Value is: "
|  |---- Identifier: x
```

## The Nodes
- Assignment
- BinaryOperator
- UnaryOperator
- Compound
- VarDeclaration
- ConstDeclaration
- FuncDeclaration

## The Leaves
- IntLiteral
- RealLiteral
- BoolLiteral
- Identifier
- TypeDef
- NoOp