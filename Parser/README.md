# The Parser

> A parser is a software component that takes input data (frequently text) and builds a data structure – often some kind of parse tree, abstract syntax tree or other hierarchical structure, giving a structural representation of the input while checking for correct syntax
> 
> [Wikipedia - Parsing](https://en.wikipedia.org/wiki/Parsing#Parser)


The current grammar rules:
```
program         -> declaration* EOF ;

# Declarations ===============================================================

declaration     -> varDecl
                | constDecl
                | function
                | statement ;

varDecl         -> "let" IDENTIFIER ( "," IDENTIFIER )* ( "=" expression )? ";" ;
constDecl       -> "const" IDENTIFIER ( "," IDENTIFIER )* "=" expression ";" ;

function        -> "func" IDENTIFIER "(" parameters? ")" block ;

parameters      -> IDENTIFIER ( "," IDENTIFIER )* ;

# Statements ================================================================

statement       -> exprStmt
                | ifStmt
                | whileStmt
                | forStmt
                | breakStmt
                | continueStmt
                | returnStmt
                | importStmt
                | block ;

whileStmt       -> "while" "(" expression ")" statement ;
forStmt         -> "for" "(" ( varDecl | exprStmt | ";" ) expression? ";" expression? ")" statement ;

ifStmt          -> "if" "(" expression ")" statement ( "else" statement )? ;

breakStmt       -> "break" ";"? ;
continueStmt    -> "continue" ";"? ;
returnStmt      -> "return" expression? ";"? ;

block           -> "{" declaration* "}" ;

importStmt      -> "import" STRING ";"? ;

exprStmt        -> expression ";"? ;


# Expressions ================================================================

expression      -> assignment ;
assignment      -> ( (call | indexing | memberAccess) "." )? IDENTIFIER "=" assignment
                | logic_or ;
logic_or        -> logic_and ("||" logic_and)* ;
logic_and       -> equality ("&&" equality)* ;
equality        -> comparison ( ( "!=" | "==" ) comparison )* ;
comparison      -> term ( ( ">" | ">=" | "<" | "<=" ) term )* ;
term            -> factor ( ( "-" | "+" ) factor )* ;
factor          -> expo ( ( "/" | "*" | "%" ) expo )* ;
expo            -> unary ("**" unary)* ;
unary           -> ( "!" | "-" | "+" ) unary
                | indexing
                | lambda
                | memberAccess
                | call ;

call            -> primary ( "(" arguments? ")" )* ;
indexing        -> primary ( "[" expression "]" )* ;
memberAccess    -> primary ("." IDENTIFIER)* ;
lambda          -> "func" "(" parameters? ")" block ;

primary         -> INTEGER | REAL | STRING
                | "true" | "false" | "null"
                | "(" expression ")"
                | array
                | IDENTIFIER ;

array           -> "[" (expression ("," expression)*)? "]" ;

arguments       -> expression ( "," expression )* ;
```