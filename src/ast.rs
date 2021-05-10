use crate::{lexer::tokens::Token, objects::Object};
use std::rc::Rc;

#[derive(Clone)]
pub enum ASTNode {
    Module(ModuleNode),

    // Expressions
    Array(ArrayExprNode),
    ArrayIndexing(ArrayIndexingExprNode),
    Binary(BinaryExprNode),
    FunctionCallExpr(FunctionCallExprNode),
    Identifier(IdentifierExprNode),
    Literal(LiteralExprNode),
    TernaryConditional(TernaryConditionalNode),
    Unary(UnaryExprNode),
    VarReassignment(VarReassignmentExprNode),

    // Declarations
    ConstantDecl(ConstantDeclNode),
    FunctionDecl(FunctionDeclNode),
    VariableDecl(VariableDeclNode),

    // Statements
    BlockStmt(BlockNode),
    BreakStmt(BreakStmtNode),
    ExpressionStmt(ExpressionStmtNode),
    IfStmt(IfStmtNode),
    PrintStmt(PrintStmtNode),
    ReturnStmt(ReturnStmtNode),
    WhileStmt(WhileStmtNode),
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
            ASTNode::Literal(x) => match x.value {
                Object::Bool(y) => !y,
                _ => false,
            },
            _ => return false,
        }
    }
}

#[derive(Clone)]
pub struct ModuleNode {
    pub body: Vec<ASTNode>,
}

#[derive(Clone)]
pub struct LiteralExprNode {
    pub value: Object,
    pub token: Rc<Token>,
}

#[derive(Clone)]
pub struct ArrayExprNode {
    pub values: Vec<Box<ASTNode>>,
    pub token: Rc<Token>,
}

#[derive(Clone)]
pub struct UnaryExprNode {
    pub operand: Box<ASTNode>,
    pub opr_type: UnaryExprType,
    pub pos: (usize, usize),
}

#[derive(Clone, Debug)]
pub enum UnaryExprType {
    ArithmeticNeg,
    LogicNeg,
    BitwiseNeg,
}

#[derive(Clone)]
pub struct BinaryExprNode {
    pub left: Box<ASTNode>,
    pub right: Box<ASTNode>,
    pub opr_token: Rc<Token>,
    pub opr_type: BinaryExprType,
}

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

#[derive(Clone)]
pub struct TernaryConditionalNode {
    pub condition: Box<ASTNode>,
    pub true_branch_token: Rc<Token>,
    pub branch_true: Box<ASTNode>,
    pub false_branch_token: Rc<Token>,
    pub branch_false: Box<ASTNode>,
}

#[derive(Clone)]
pub struct IdentifierExprNode {
    pub token: Rc<Token>,
}

#[derive(Clone)]
pub struct PrintStmtNode {
    pub child: Box<ASTNode>,
    pub pos: (usize, usize),
}

#[derive(Clone)]
pub struct ExpressionStmtNode {
    pub child: Box<ASTNode>,
    pub pos: (usize, usize),
}

#[derive(Clone)]
pub struct VariableDeclNode {
    pub identifiers: Vec<Rc<Token>>,
    pub value: Box<ASTNode>,
}

#[derive(Clone)]
pub enum ReassignmentType {
    Plus,  // aka, a += b
    Minus, // aka, a -= b
    Mul,   // aka, a *= b
    Div,   // aka, a /= b
    Expo,  // aka, a **= b
    Mod,   // aka, a %= b
    None,  // Regular reassignment (aka, `a = b`)
}

#[derive(Clone)]
pub struct VarReassignmentExprNode {
    pub target: Rc<Token>,
    pub value: Box<ASTNode>,
    pub opr_type: ReassignmentType,
    pub pos: (usize, usize),
}

#[derive(Clone)]
pub struct ArrayIndexingExprNode {
    pub target: Box<ASTNode>,
    pub index: Box<ASTNode>,
    pub pos: (usize, usize),
}

#[derive(Clone)]
pub struct ConstantDeclNode {
    pub name: Rc<Token>,
    pub value: Box<ASTNode>,
}

#[derive(Clone)]
pub struct BlockNode {
    pub body: Vec<ASTNode>,
}

#[derive(Clone)]
pub struct IfStmtNode {
    pub condition: Box<ASTNode>,
    pub then_token: Rc<Token>,
    pub then_branch: Box<ASTNode>,
    pub else_branch: Box<Option<ASTNode>>,
    pub else_token: Option<Rc<Token>>,
}

#[derive(Clone)]
pub struct WhileStmtNode {
    pub token: Rc<Token>,
    pub condition: Box<ASTNode>,
    pub body: Box<ASTNode>,
}

#[derive(Clone)]
pub struct BreakStmtNode {
    pub token: Rc<Token>,
}

#[derive(Clone)]
pub struct FunctionDeclNode {
    pub name: Rc<Token>,
    pub params: Vec<Parameter>,
    pub min_arity: u8,
    pub max_arity: u8,
    pub body: Box<ASTNode>,
}

#[derive(Clone)]
pub struct Parameter {
    pub name: Rc<Token>,
    pub is_optional: bool,
    pub default: Option<Box<ASTNode>>,
}

#[derive(Clone)]
pub struct FunctionCallExprNode {
    pub target: Box<ASTNode>,
    pub args: Vec<Argument>,
    pub pos: (usize, usize),
}

#[derive(Clone)]
pub struct Argument {
    pub name: Option<ASTNode>,
    pub is_named: bool,
    pub value: Box<ASTNode>,
}

#[derive(Clone)]
pub struct ReturnStmtNode {
    pub token: Rc<Token>,
    pub value: Option<Box<ASTNode>>,
}
