use std::rc::Rc;

use crate::{lexer::tokens::Token, objects::Object};

pub enum ASTNode<'a> {
    Literal(LiteralExprNode<'a>),
    Binary(BinaryExprNode<'a>),
    Unary(UnaryExprNode<'a>),
}

impl<'a> ASTNode<'a> {
    pub fn print(&self, depth: usize) {
        match self {
            ASTNode::Literal(_) => println!("Literal"),
            ASTNode::Binary(expr) => {
                println!("{:?}", expr.token.token_type);
                print!("{}Left: ", "\t".repeat(depth + 1));
                expr.left.print(depth + 1);
                print!("{}Right: ", "\t".repeat(depth + 1));
                expr.right.print(depth + 1);
            }
            ASTNode::Unary(expr) => {
                println!("{:?}", expr.token.token_type);
                print!("{}Operand: ", "\t".repeat(depth + 1));
                expr.operand.print(depth + 1);
            }
        }
    }
}

// A literal
pub struct LiteralExprNode<'a> {
    pub value: Rc<Object<'a>>,
    pub token: Rc<Token<'a>>,
}

// A unary
pub enum UnaryExprType {
    ArithmeticNeg,
    LogicNeg,
    BitwiseNeg,
}

pub struct UnaryExprNode<'a> {
    pub operand: Box<ASTNode<'a>>,
    pub opr_type: UnaryExprType,
    pub token: Rc<Token<'a>>,
}

// A binary expression
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

pub struct BinaryExprNode<'a> {
    pub left: Box<ASTNode<'a>>,
    pub right: Box<ASTNode<'a>>,
    pub token: Rc<Token<'a>>,
    pub opr_type: BinaryExprType,
}
