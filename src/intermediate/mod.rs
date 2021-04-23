/// The `ast` module holds information about how Hinton
/// code is represented in an Abstract Syntax Tree.
pub mod ast;

/// * The `parser` module converts Hinton source code into
/// an abstract syntax tree (AST) intermediate form.
pub mod parser;
pub mod parse_declarations;

/// * The `compiler` module converts the AST into Hinton
/// bytecode for execution.
pub mod compiler;
