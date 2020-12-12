# The Parser

> A parser is a software component that takes input data (frequently text) and builds a data structure – often some kind of parse tree, abstract syntax tree or other hierarchical structure, giving a structural representation of the input while checking for correct syntax
> 
> [Wikipedia - Parsing](https://en.wikipedia.org/wiki/Parsing#Parser)


The current grammar rules:
```
expression      -> equality ;
equality        -> comparison ( ( "!=" | "==" ) comparison )* ;
comparison      -> logic_or ( ( ">" | ">=" | "<" | "<=" ) logic_or )* ;
logic_or        -> logic_and ("||" logic_and)* ;
logic_and       -> term ("&&" term)* ;
term            -> factor ( ( "-" | "+" ) factor )* ;
factor          -> expo ( ( "/" | "*" | "%" ) expo )* ;
expo            -> unary ("**" unary)* ;
unary           -> ( "!" | "-" | "+" ) unary | primary ;

primary         -> INTEGER | REAL | STRING | "true" | "false" | "null"
                | "(" expression ")" ;
```