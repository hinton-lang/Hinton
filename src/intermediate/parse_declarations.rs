use std::rc::Rc;

use crate::{
    lexer::tokens::{Token, TokenType},
    objects::Object,
};

use super::{
    ast::{ASTNode, LiteralExprNode, VariableDeclNode},
    parser::Parser,
};

impl Parser {
    pub(super) fn parse_var_declaration(&mut self) -> Option<ASTNode> {
        let mut declarations: Vec<Rc<Token>> = Vec::new();

        // Gets at least one variable name, or a list of
        // names separated by a comma
        self.consume(TokenType::IDENTIFIER, "Expected variable name.");
        declarations.push(Rc::clone(&self.previous));

        while self.matches(TokenType::COMMA_SEPARATOR) {
            self.consume(TokenType::IDENTIFIER, "Expected variable name.");
            declarations.push(Rc::clone(&self.previous));
        }

        // Since the .forEach loop bellow requires the
        // variables to be final, we use an array of size
        // one to represent the value of the variable.
        let initializer = if self.matches(TokenType::EQUALS_SIGN) {
            match self.parse_expression() {
                Some(val) => val,
                None => return None, // Could not create value for variable
            }
        } else {
            ASTNode::Literal(LiteralExprNode {
                value: Rc::new(Object::Null),
                token: Rc::clone(&self.previous),
            })
        };

        // Requires a semicolon at the end of the declaration
        // if the declaration was not a block (e.g., lambda functions)
        if self.previous.token_type != TokenType::RIGHT_CURLY_BRACES {
            self.consume(TokenType::SEMICOLON_SEPARATOR, "Expected ';' after variable declaration.");
        }

        // But if there is a semicolon after a curly brace, then we consume it
        if self.previous.token_type == TokenType::RIGHT_CURLY_BRACES && self.check(TokenType::SEMICOLON_SEPARATOR) {
            self.advance();
        }

        return Some(ASTNode::VariableDecl(VariableDeclNode {
            identifiers: declarations,
            value: Box::new(initializer.clone()),
        }));
    }
}
