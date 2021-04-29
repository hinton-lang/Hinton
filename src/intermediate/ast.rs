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
    Array(ArrayExprNode),
    PostIncrement(PostIncrementExprNode),
    PostDecrement(PostDecrementExprNode),
    ArrayIndexing(ArrayIndexingExprNode),

    // Declarations
    VariableDecl(VariableDeclNode),
    ConstantDecl(ConstantDeclNode),

    // Statements
    PrintStmt(PrintStmtNode),
    ExpressionStmt(ExpressionStmtNode),
    BlockStmt(BlockNode),
    IfStmt(IfStmtNode),
    WhileStmt(WhileStmtNode),
    BreakStmt(BreakStmtNode),
}

impl ASTNode {
    pub fn is_truthy_literal(&self) -> bool {
        match self {
            ASTNode::Literal(x) => !x.value.is_falsey(),
            _ => return false,
        }
    }

    pub fn is_false_literal(&self) -> bool {
        match self {
            ASTNode::Literal(x) => match *x.value {
                Object::Bool(y) => !y,
                _ => false,
            },
            _ => return false,
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

/// Represents an array expression node in Hinton's Abstract Syntax Tree.
#[derive(Clone)]
pub struct ArrayExprNode {
    pub values: Vec<Box<ASTNode>>,
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
    pub opr_token: Rc<Token>,
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
    pub true_branch_token: Rc<Token>,
    pub branch_true: Box<ASTNode>,
    pub false_branch_token: Rc<Token>,
    pub branch_false: Box<ASTNode>,
}

/// Represents an identifier expression node in Hinton's Abstract Syntax Tree.
#[derive(Clone)]
pub struct IdentifierExprNode {
    // This node needs a reference to the token because at compile time,
    // we need information about the token for printing errors when there
    // is a problem with storing the identifier in the constant pool.
    pub token: Rc<Token>,
}

/// Represents a post-increment expression node in Hinton's Abstract Syntax Tree.
#[derive(Clone)]
pub struct PostIncrementExprNode {
    pub target: Rc<Token>,
    pub token: Rc<Token>,
}

/// Represents a post-decrement expression node in Hinton's Abstract Syntax Tree.
#[derive(Clone)]
pub struct PostDecrementExprNode {
    pub target: Rc<Token>,
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

/// Represents an array indexing expression node in Hinton's Abstract Syntax Tree.
#[derive(Clone)]
pub struct ArrayIndexingExprNode {
    pub target: Box<ASTNode>,
    pub index: Box<ASTNode>,
    pub pos: (usize, usize),
}

/// Represents a constant declaration node in Hinton's Abstract Syntax Tree.
#[derive(Clone)]
pub struct ConstantDeclNode {
    pub name: Rc<Token>,
    pub value: Box<ASTNode>,
}

/// Represents a block node in Hinton's Abstract Syntax Tree.
#[derive(Clone)]
pub struct BlockNode {
    pub body: Vec<ASTNode>,
}

/// Represents an if statement node in Hinton's Abstract Syntax Tree.
#[derive(Clone)]
pub struct IfStmtNode {
    pub condition: Box<ASTNode>,
    pub then_token: Rc<Token>,
    pub then_branch: Box<ASTNode>,
    pub else_branch: Box<Option<ASTNode>>,
    pub else_token: Option<Rc<Token>>,
}

/// Represents a while statement node in Hinton's Abstract Syntax Tree.
#[derive(Clone)]
pub struct WhileStmtNode {
    pub token: Rc<Token>,
    pub condition: Box<ASTNode>,
    pub body: Box<ASTNode>,
}

/// Represents a break statement node in Hinton's Abstract Syntax Tree.
#[derive(Clone)]
pub struct BreakStmtNode {
    pub token: Rc<Token>,
}
