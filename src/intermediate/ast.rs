use std::rc::Rc;

use crate::{lexer::tokens::Token, objects::Object};

/// Represents a module node in Hinton's Abstract Syntax Tree.
/// This node holds information about a particular file, as well
/// as the declarations and statements made within that file.
/// This is the root of a Hinton AST.
#[derive(Clone)]
pub struct ModuleNode {
    pub body: Vec<ASTNode>,
}

/// Represents a single node in Hinton's Abstract Syntax Tree.
/// This node holds information about expressions, statements,
/// and declarations in the source code.
#[derive(Clone)]
pub enum ASTNode {
    // Expressions
    Literal(LiteralExprNode),
    Binary(BinaryExprNode),
    Unary(UnaryExprNode),
    TernaryConditional(TernaryConditionalNode),
    Identifier(IdentifierExprNode),
    VarReassignment(VarReassignmentExprNode),

    // Declarations
    VariableDecl(VariableDeclNode),

    // Statements
    PrintStmt(PrintStmtNode),
    ExpressionStmt(ExpressionStmtNode),
}

/// Implementation for the ASTNode struct
impl<'a> ASTNode {
    /// Recursively prints the current node's information as well as
    /// its children's information
    ///
    /// ## Arguments
    /// `depth` – The depth in the AST tree of the current node.
    pub fn print(&self, depth: usize) {
        match self {
            ASTNode::Literal(expr) => println!("{}", expr.value),
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
            ASTNode::VariableDecl(decl) => {
                for id in decl.identifiers.iter() {
                    println!("Variable declaration: ");
                    println!("{}Name: {}", "\t".repeat(depth + 1), id.lexeme);
                    print!("{}value: ", "\t".repeat(depth + 1));
                    decl.value.print(depth + 1);
                }
            }
            ASTNode::VarReassignment(_) => {}
        }
    }
}

/// Represents a literal node in Hinton's Abstract Syntax Tree.
/// This node holds information about literal values in hinton
/// like strings, arrays, numerals, booleans, etc...
#[derive(Clone)]
pub struct LiteralExprNode {
    pub value: Rc<Object>,
    // This node needs a reference to the token because at compile time,
    // we need information about the token for printing errors when there
    // is a problem with storing the literal in the constant pool.
    pub token: Rc<Token>,
}

/// Represents a unary expression node in Hinton's Abstract Syntax Tree.
#[derive(Clone)]
pub struct UnaryExprNode {
    pub operand: Box<ASTNode>,
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
pub struct BinaryExprNode {
    pub left: Box<ASTNode>,
    pub right: Box<ASTNode>,
    pub pos: (usize, usize),
    pub opr_type: BinaryExprType,
}

/// Types of binary expressions in Hinton
#[derive(Clone, Debug)]
pub enum BinaryExprType {
    Addition,
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
    Range,
}

/// Represents a ternary conditional expression node in Hinton's Abstract Syntax Tree.
#[derive(Clone)]
pub struct TernaryConditionalNode {
    pub condition: Box<ASTNode>,
    pub branch_true: Box<ASTNode>,
    pub branch_false: Box<ASTNode>,
    pub pos: (usize, usize),
}

/// Represents an identifier expression node in Hinton's Abstract Syntax Tree.
#[derive(Clone)]
pub struct IdentifierExprNode {
    // This node needs a reference to the token because at compile time,
    // we need information about the token for printing errors when there
    // is a problem with storing the identifier in the constant pool.
    pub token: Rc<Token>,
}

/// Represents a print statement node in Hinton's Abstract Syntax Tree.
#[derive(Clone)]
pub struct PrintStmtNode {
    pub child: Box<ASTNode>,
    pub pos: (usize, usize),
}

/// Represents an expression statement node in Hinton's Abstract Syntax Tree.
#[derive(Clone)]
pub struct ExpressionStmtNode {
    pub child: Box<ASTNode>,
    pub pos: (usize, usize),
}

/// Represents a variable declaration node in Hinton's Abstract Syntax Tree.
#[derive(Clone)]
pub struct VariableDeclNode {
    pub identifiers: Vec<Rc<Token>>,
    pub value: Box<ASTNode>,
}

/// Represents an assignment expression node in Hinton's Abstract Syntax Tree.
#[derive(Clone)]
pub struct VarReassignmentExprNode {
    pub target: Rc<Token>,
    pub value: Box<ASTNode>,
    pub pos: (usize, usize),
}
