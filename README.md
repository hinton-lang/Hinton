# The Hinton Language 🔮
This is a simple interpreter written in Java for a toy language called Hinton. The code is a custom interpretation of the code found in the book [Crafting Interpreters](https://craftinginterpreters.com/) by Bob Nystrom.
 
 At the moment, the Interpreter can accurately identify and label different tokens inside a `.ht` file. Some of the tokens it can identify include, but are not limited to:
  - Keywords: `let`, `const`, `func`, `if`, `elif`, `else`, + more.
  - Static Types: `String`, `Int`, `Real`, `void`, `None`, + more.
  - Literals: `"String Sequences"`, `2342`, `3.1242`, `true`, `false`, `none`, + more.
  - Arithmetic Operators: `+`, `-`, `*`, `/`, `**`, `%`, and `mod`.
  - Logical Operators: `<`, `>`, `==`, `equals`, `!`, `not`, + more.
  - Delimiters & Separators: `()`, `,`, `{}`, `:`, `[]`, `.`, and `;`
  
It is able to parse complex grammatical rules as specified inside the `grammar.cfg` file in order to generate an Abstract Syntax Tree: Some of the rule include, but are not limited to:
  - Expressions following a certain order of precedence.
  - Terminals that evaluate to a specific value.
  - Print statements

It is also able to execute those expressions and statements.
   
## The General Idea
The general idea of an interpreter implementation is as follows:

1. The programmer writes a program in a source file.
2. The contents of the source file are read and inputted into a [Lexer](https://github.com/faustotnc/Interpreter/tree/master/Lexer) that converts the input text into a list of [Tokens](https://github.com/faustotnc/Interpreter/tree/master/Tokens).
3. The tokens are then fed into a [Parser](https://github.com/faustotnc/Interpreter/tree/master/Parser) which generates an [Abstract Syntax Tree (AST)](https://github.com/faustotnc/Interpreter/tree/master/AbstractSyntaxTree). The AST is a type of abstract data structure that represents the syntactic anatomy of the source code.
4. Finally, an [Interpreter](https://github.com/faustotnc/Interpreter/tree/master/Interpreter) takes the AST and "visits" each node and leaf of the AST until it has completed executing the source code.

## The Selfish Idea
Hopefully, at some point during my career as a computer scientist, Hinton will become a full programming language.
