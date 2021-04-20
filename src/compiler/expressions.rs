use std::rc::Rc;

use crate::{lexer::tokens::TokenType, objects::Object};

use super::{
    ast::{ASTNode, ASTNode::*, BinaryExprNode, BinaryExprType, LiteralExprNode, UnaryExprNode, UnaryExprType},
    parser::Parser,
};

impl<'a> Parser<'a> {
    /// Parses an expression as specified in the grammar.cfg file.
    ///
    /// ## Returns
    /// `Option<ASTNode<'a>>` – The expression's AST node.
    pub(super) fn expression(&mut self) -> Option<ASTNode<'a>> {
        self.assignment()
    }

    /// Parses an assignment expression as specified in the grammar.cfg file.
    ///
    /// ## Returns
    /// `Option<ASTNode<'a>>` – The assignment expression's AST node.
    pub(super) fn assignment(&mut self) -> Option<ASTNode<'a>> {
        let expr = self.logic_or();

        return expr;
    }

    /// Parses an 'OR' expression as specified in the grammar.cfg file.
    ///
    /// ## Returns
    /// `Option<ASTNode<'a>>` – The expression's AST node.
    pub(super) fn logic_or(&mut self) -> Option<ASTNode<'a>> {
        let mut expr = self.logic_and();

        while self.matches(TokenType::LOGICAL_OR) {
            let opr = self.previous.clone();

            expr = Some(Binary(BinaryExprNode {
                left: match expr {
                    Some(e) => Box::new(e),
                    None => return None, // Could not create lhs of expression
                },
                right: Box::new(self.logic_and().unwrap()),
                token: opr,
                opr_type: BinaryExprType::LogicOR,
            }));
        }

        return expr;
    }

    /// Parses an 'AND' expression as specified in the grammar.cfg file.
    ///
    /// ## Returns
    /// `Option<ASTNode<'a>>` – The expression's AST node.
    pub(super) fn logic_and(&mut self) -> Option<ASTNode<'a>> {
        let mut expr = self.equality();

        while self.matches(TokenType::LOGICAL_AND) {
            let opr = self.previous.clone();

            expr = Some(Binary(BinaryExprNode {
                left: match expr {
                    Some(e) => Box::new(e),
                    None => return None, // Could not create lhs of expression
                },
                right: Box::new(self.equality().unwrap()),
                token: opr,
                opr_type: BinaryExprType::LogicAND,
            }));
        }

        return expr;
    }

    /// Parses an equality expression as specified in the grammar.cfg file.
    ///
    /// ## Returns
    /// `Option<ASTNode<'a>>` – The expression's AST node.
    pub(super) fn equality(&mut self) -> Option<ASTNode<'a>> {
        let mut expr = self.comparison();

        while self.matches(TokenType::LOGICAL_EQ) || self.matches(TokenType::LOGICAL_NOT_EQ) {
            let opr = self.previous.clone();

            let opr_type = if opr.token_type == TokenType::LOGICAL_EQ {
                BinaryExprType::LogicEQ
            } else {
                BinaryExprType::LogicNotEQ
            };

            expr = Some(Binary(BinaryExprNode {
                left: match expr {
                    Some(e) => Box::new(e),
                    None => return None, // Could not create lhs of expression
                },
                right: Box::new(self.comparison().unwrap()),
                token: opr,
                opr_type,
            }));
        }

        return expr;
    }

    /// Parses a comparison expression as specified in the grammar.cfg file.
    ///
    /// ## Returns
    /// `Option<ASTNode<'a>>` – The expression's AST node.
    pub(super) fn comparison(&mut self) -> Option<ASTNode<'a>> {
        let mut expr = self.range();

        while self.matches(TokenType::LESS_THAN)
            || self.matches(TokenType::LESS_THAN_EQ)
            || self.matches(TokenType::GREATER_THAN)
            || self.matches(TokenType::GREATER_THAN_EQ)
        {
            let opr = self.previous.clone();

            let opr_type = if opr.token_type == TokenType::LESS_THAN {
                BinaryExprType::LogicLessThan
            } else if opr.token_type == TokenType::LESS_THAN_EQ {
                BinaryExprType::LogicLessThanEQ
            } else if opr.token_type == TokenType::GREATER_THAN {
                BinaryExprType::LogicGreaterThan
            } else {
                BinaryExprType::LogicGreaterThanEQ
            };

            expr = Some(Binary(BinaryExprNode {
                left: match expr {
                    Some(e) => Box::new(e),
                    None => return None, // Could not create lhs of expression
                },
                right: Box::new(self.range().unwrap()),
                token: opr,
                opr_type,
            }));
        }

        return expr;
    }

    /// Parses a range expression as specified in the grammar.cfg file.
    ///
    /// ## Returns
    /// `Option<ASTNode<'a>>` – The expression's AST node.
    pub(super) fn range(&mut self) -> Option<ASTNode<'a>> {
        let mut expr = self.term();

        if self.matches(TokenType::RANGE_OPERATOR) {
            let opr = self.previous.clone();

            expr = Some(Binary(BinaryExprNode {
                left: match expr {
                    Some(e) => Box::new(e),
                    None => return None, // Could not create lhs of expression
                },
                right: Box::new(self.term().unwrap()),
                token: opr,
                opr_type: BinaryExprType::Range,
            }));
        }

        return expr;
    }

    /// Parses a term expression as specified in the grammar.cfg file.
    ///
    /// ## Returns
    /// `Option<ASTNode<'a>>` – The expression's AST node.
    pub(super) fn term(&mut self) -> Option<ASTNode<'a>> {
        let mut expr = self.factor();

        while self.matches(TokenType::PLUS) || self.matches(TokenType::MINUS) {
            let opr = self.previous.clone();

            let opr_type = if opr.token_type == TokenType::PLUS {
                BinaryExprType::Addition
            } else {
                BinaryExprType::Minus
            };

            expr = Some(Binary(BinaryExprNode {
                left: match expr {
                    Some(e) => Box::new(e),
                    None => return None, // Could not create lhs of expression
                },
                right: Box::new(self.factor().unwrap()),
                token: opr,
                opr_type,
            }));
        }

        return expr;
    }

    /// Parses a factor expression as specified in the grammar.cfg file.
    ///
    /// ## Returns
    /// `Option<ASTNode<'a>>` – The expression's AST node.
    pub(super) fn factor(&mut self) -> Option<ASTNode<'a>> {
        let mut expr = self.expo();

        while self.matches(TokenType::SLASH) || self.matches(TokenType::STAR) || self.matches(TokenType::MODULUS) {
            let opr = self.previous.clone();

            let opr_type = if opr.token_type == TokenType::SLASH {
                BinaryExprType::Division
            } else if opr.token_type == TokenType::STAR {
                BinaryExprType::Multiplication
            } else {
                BinaryExprType::Modulus
            };

            expr = Some(Binary(BinaryExprNode {
                left: match expr {
                    Some(e) => Box::new(e),
                    None => return None, // Could not create lhs of expression
                },
                right: Box::new(self.expo().unwrap()),
                token: opr,
                opr_type,
            }));
        }

        return expr;
    }

    /// Parses an exponentiation expression as specified in the grammar.cfg file.
    ///
    /// ## Returns
    /// `Option<ASTNode<'a>>` – The expression's AST node.
    pub(super) fn expo(&mut self) -> Option<ASTNode<'a>> {
        let mut expr = self.unary();

        while self.matches(TokenType::EXPO) {
            let opr = self.previous.clone();

            expr = Some(Binary(BinaryExprNode {
                left: match expr {
                    Some(e) => Box::new(e),
                    None => return None, // Could not create lhs of expression
                },
                right: Box::new(self.unary().unwrap()),
                token: opr,
                opr_type: BinaryExprType::Expo,
            }));
        }

        return expr;
    }

    /// Parses a unary expression as specified in the grammar.cfg file.
    ///
    /// ## Returns
    /// `Option<ASTNode<'a>>` – The expression's AST node.
    pub(super) fn unary(&mut self) -> Option<ASTNode<'a>> {
        if self.matches(TokenType::LOGICAL_NOT) || self.matches(TokenType::MINUS) || self.matches(TokenType::BITWISE_NOT) {
            let opr = self.previous.clone();
            let expr = self.primary();

            let opr_type = if opr.token_type == TokenType::LOGICAL_NOT {
                UnaryExprType::LogicNeg
            } else if opr.token_type == TokenType::BITWISE_NOT {
                UnaryExprType::BitwiseNeg
            } else {
                UnaryExprType::ArithmeticNeg
            };

            return Some(Unary(UnaryExprNode {
                operand: match expr {
                    Some(e) => Box::new(e),
                    None => return None, // Could not create rhs of expression
                },
                token: opr,
                opr_type,
            }));
        } else {
            let expr = self.primary();

            return expr;
        }
    }

    /// Parses a primary (literal) expression as specified in the grammar.cfg file.
    ///
    /// ## Returns
    /// `Option<ASTNode<'a>>` – The expression's AST node.
    pub(super) fn primary(&mut self) -> Option<ASTNode<'a>> {
        self.advance();

        let literal_value = match self.get_previous_tok_type() {
            TokenType::STRING_LITERAL => self.compile_string(),
            TokenType::TRUE_LITERAL => Rc::new(Object::Bool(true)),
            TokenType::FALSE_LITERAL => Rc::new(Object::Bool(false)),
            TokenType::NULL_LITERAL => Rc::new(Object::Null()),
            TokenType::NUMERIC_LITERAL => match self.compile_number() {
                Ok(x) => x,
                Err(_) => return None,
            },
            TokenType::BINARY_LITERAL => match self.compile_int_from_base(2) {
                Ok(x) => x,
                Err(_) => return None,
            },
            TokenType::OCTAL_LITERAL => match self.compile_int_from_base(8) {
                Ok(x) => x,
                Err(_) => return None,
            },
            TokenType::HEXADECIMAL_LITERAL => match self.compile_int_from_base(16) {
                Ok(x) => x,
                Err(_) => return None,
            },
            _ => return None,
        };

        let node = LiteralExprNode {
            value: literal_value,
            token: self.current.clone(),
        };

        return Some(Literal(node));
    }

    /// Compiles a string token to a Hinton String.
    pub(super) fn compile_string(&mut self) -> Rc<Object<'a>> {
        let lexeme = self.previous.lexeme.clone();

        // Remove outer quotes from the source string
        let lexeme = &lexeme[1..(lexeme.len() - 1)];

        // Replace escaped characters with the actual representations
        let lexeme = lexeme
            .replace("\\n", "\n")
            .replace("\\t", "\t")
            .replace("\\r", "\r")
            .replace("\\\\", "\\")
            .replace("\\\"", "\"");

        // Emits the constant instruction
        return Rc::new(Object::String(lexeme));
    }

    /// Compiles a number token to a Hinton Number.
    pub(super) fn compile_number(&mut self) -> Result<Rc<Object<'a>>, ()> {
        let lexeme = self.previous.lexeme.clone();
        // Removes the underscores from the lexeme
        let lexeme = lexeme.replace('_', "");
        // Parses the lexeme into a float
        let num = lexeme.parse::<f64>();

        // If the lexeme could successfully be converted to `isize` integer
        // then we proceed to save it in the constant pool and emit the
        // instruction. Otherwise, we indicate that there was a compilation error.
        return match num {
            Ok(x) => Ok(Rc::new(Object::Number(x))),
            Err(_) => {
                self.error_at_previous("Unexpected token.");
                Err(())
            }
        };
    }

    /// Compiles a binary, octal, or hexadecimal number token to a Hinton Number.
    pub(super) fn compile_int_from_base(&mut self, radix: u32) -> Result<Rc<Object<'a>>, ()> {
        let lexeme = self.previous.lexeme.clone();
        // Removes the underscores from the lexeme
        let lexeme = lexeme.replace('_', "");
        // Parses the lexeme into an integer
        let num = isize::from_str_radix(&lexeme[2..], radix);

        // If the lexeme could successfully be converted to `isize` integer
        // then we proceed to save it in the constant pool and emit the
        // instruction. Otherwise, we indicate that there was a compilation error.
        return match num {
            Ok(x) => Ok(Rc::new(Object::Number(x as f64))),
            Err(_) => {
                self.error_at_previous("Unexpected token.");
                Err(())
            }
        };
    }
}
