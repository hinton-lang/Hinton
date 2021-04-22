use std::rc::Rc;

use crate::{lexer::tokens::Token, objects::Object};

/// Represents a module node in Hinton's Abstract Syntax Tree.
/// This node holds information about a particular file, as well
/// as the declarations and statements made within that file.
/// This is the root of a Hinton AST.
#[derive(Clone)]
pub struct ModuleNode<'a> {
    pub body: Vec<ASTNode<'a>>,
}

/// Represents a single node in Hinton's Abstract Syntax Tree.
/// This node holds information about expressions, statements,
/// and declarations in the source code.
#[derive(Clone)]
pub enum ASTNode<'a> {
    // Expressions
    Literal(LiteralExprNode<'a>),
    Binary(BinaryExprNode<'a>),
    Unary(UnaryExprNode<'a>),
    TernaryConditional(TernaryConditionalNode<'a>),
    Identifier(IdentifierExprNode<'a>),

    // Declarations
    // coming soon...

    // General statements
    PrintStmt(PrintStmtNode<'a>),
    ExpressionStmt(ExpressionStmtNode<'a>),
}

/// Implementation for the ASTNode struct
impl<'a> ASTNode<'a> {
    /// Recursively prints the current node's information as well as
    /// its children's information
    ///
    /// ## Arguments
    /// `depth` – The depth in the AST tree of the current node.
    pub fn print(&self, depth: usize) {
        match self {
            ASTNode::Literal(expr) => println!("Literal: {}", expr.value),
            ASTNode::Binary(expr) => {
                println!("{:?}", expr.opr_type);
                print!("{}Left: ", "\t".repeat(depth + 1));
                expr.left.print(depth + 1);
                print!("{}Right: ", "\t".repeat(depth + 1));
                expr.right.print(depth + 1);
            }
            ASTNode::Unary(expr) => {
                println!("{:?}", expr.opr_type);
                print!("{}Operand: ", "\t".repeat(depth + 1));
                expr.operand.print(depth + 1);
            }
            ASTNode::TernaryConditional(expr) => {
                println!("Ternary Conditional");
                print!("{}Condition: ", "\t".repeat(depth + 1));
                expr.condition.print(depth + 1);
                print!("{}Branch True: ", "\t".repeat(depth + 1));
                expr.branch_true.print(depth + 1);
                print!("{}Branch False: ", "\t".repeat(depth + 1));
                expr.branch_false.print(depth + 1);
            }
            ASTNode::Identifier(expr) => {
                println!("Identifier: {}", expr.token.lexeme)
            }
            ASTNode::PrintStmt(stmt) => {
                print!("Print Stmt:");
                stmt.child.print(depth + 1);
            }
            ASTNode::ExpressionStmt(stmt) => {
                print!("Expression Stmt:");
                stmt.child.print(depth + 1);
            }
        }
    }
}

/// Represents a literal node in Hinton's Abstract Syntax Tree.
/// This node holds information about literal values in hinton
/// like strings, arrays, numerals, booleans, etc...
#[derive(Clone)]
pub struct LiteralExprNode<'a> {
    pub value: Rc<Object<'a>>,
    // This node needs a reference to the token because at compile time,
    // we need information about the token for printing errors when there
    // is a problem with storing the literal in the constant pool.
    pub token: Rc<Token<'a>>,
}

/// Represents a unary expression node in Hinton's Abstract Syntax Tree.
#[derive(Clone)]
pub struct UnaryExprNode<'a> {
    pub operand: Box<ASTNode<'a>>,
    pub opr_type: UnaryExprType,
    pub pos: (usize, usize),
}

/// Types of unary expressions in Hinton
#[derive(Clone, Debug)]
pub enum UnaryExprType {
    ArithmeticNeg,
    LogicNeg,
    BitwiseNeg,
}

/// Represents a binary expression node in Hinton's Abstract Syntax Tree.
#[derive(Clone)]
pub struct BinaryExprNode<'a> {
    pub left: Box<ASTNode<'a>>,
    pub right: Box<ASTNode<'a>>,
    pub pos: (usize, usize),
    pub opr_type: BinaryExprType,
}

/// Types of binary expressions in Hinton
#[derive(Clone, Debug)]
pub enum BinaryExprType {
    BitwiseAND,
    BitwiseOR,
    BitwiseShiftLeft,
    BitwiseShiftRight,
    BitwiseXOR,
    Division,
    Expo,
    LogicAND,
    LogicEQ,
    LogicGreaterThan,
    LogicGreaterThanEQ,
    LogicLessThan,
    LogicLessThanEQ,
    LogicNotEQ,
    LogicOR,
    Minus,
    Modulus,
    Multiplication,
    Nullish,
    Addition,
    Range,
}

/// Represents a ternary conditional expression node in Hinton's Abstract Syntax Tree.
#[derive(Clone)]
pub struct TernaryConditionalNode<'a> {
    pub condition: Box<ASTNode<'a>>,
    pub branch_true: Box<ASTNode<'a>>,
    pub branch_false: Box<ASTNode<'a>>,
    pub pos: (usize, usize),
}

/// Represents an identifier expression node in Hinton's Abstract Syntax Tree.
#[derive(Clone)]
pub struct IdentifierExprNode<'a> {
    // This node needs a reference to the token because at compile time,
    // we need information about the token for printing errors when there
    // is a problem with storing the identifier in the constant pool.
    pub token: Rc<Token<'a>>,
}

#[derive(Clone)]
pub struct PrintStmtNode<'a> {
    pub child: Box<ASTNode<'a>>,
    pub pos: (usize, usize),
}

#[derive(Clone)]
pub struct ExpressionStmtNode<'a> {
    pub child: Box<ASTNode<'a>>,
    pub pos: (usize, usize),
}
