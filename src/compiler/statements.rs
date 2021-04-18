use std::rc::Rc;

use crate::{
    chunk::{op_codes::OpCode, ConstantPos},
    lexer::tokens::{Token, TokenType},
    objects::Object,
};

use super::Compiler;

impl<'a> Compiler<'a> {
    pub(super) fn declaration(&mut self) {
        if self.matches(TokenType::LET_KEYWORD) {
            self.variable_declaration();
        } else {
            self.statement();
        }

        if self.is_in_panic {
            self.synchronize();
        }
    }

    pub(super) fn statement(&mut self) {
        if self.matches(TokenType::PRINT) {
            self.print_statement();
        } else {
            self.expression_statement();
        }
    }

    pub(super) fn print_statement(&mut self) {
        self.consume(TokenType::LEFT_PARENTHESIS, "Expected '(' before expression.");
        self.expression();
        self.consume(TokenType::RIGHT_PARENTHESIS, "Expected ')' after expression.");
        self.consume(TokenType::SEMICOLON_SEPARATOR, "Expected ';' after expression.");
        self.emit_op_code(OpCode::OP_PRINT);
    }

    pub(super) fn expression_statement(&mut self) {
        self.expression();
        self.consume(TokenType::SEMICOLON_SEPARATOR, "Expected ';' after expression.");
        self.emit_op_code(OpCode::OP_POP_STACK);
    }

    pub(super) fn variable_declaration(&mut self) {
        let mut identifiers: Vec<u16> = Vec::new();

        // Allow the declaration of multiple variables to the same
        // value in the same statement.
        loop {
            match self.parse_variable_name("Expected variable name.") {
                ConstantPos::Pos(x) => identifiers.push(x),
                ConstantPos::Error => {
                    self.error_at_current("Could not complete variable declaration.");
                    return ();
                }
            }

            if !self.matches(TokenType::COMMA_SEPARATOR) {
                break;
            }
        }

        if self.matches(TokenType::EQUALS_SIGN) {
            self.expression();
        } else {
            self.emit_op_code(OpCode::OP_NULL);
        }

        self.consume(TokenType::SEMICOLON_SEPARATOR, "Expected ';' after variable declaration.");

        // Assign the same value to all the declared variables
        for name in identifiers.iter() {
            self.define_variable(*name);
        }

        // Pop variable's value off the stack once we
        // are done declaring the variables
        self.emit_op_code(OpCode::OP_POP_STACK);
    }

    pub(super) fn parse_variable_name(&mut self, error_message: &str) -> ConstantPos {
        self.consume(TokenType::IDENTIFIER, error_message);
        return self.add_identifier_to_pool(Rc::clone(&self.previous));
    }

    pub(super) fn add_identifier_to_pool(&mut self, token: Rc<Token>) -> ConstantPos {
        self.chunk.add_constant(Rc::new(Object::String(String::from(token.lexeme))))
    }

    pub(super) fn define_variable(&mut self, idx: u16) {
        self.emit_op_code(OpCode::OP_DEFINE_GLOBAL_VAR);
        self.emit_short(idx);
    }

    pub(super) fn consume_variable_identifier(&mut self) {
        self.named_variable(Rc::clone(&self.previous));
    }

    pub(super) fn named_variable(&mut self, token: Rc<Token<'a>>) {
        let arg = self.add_identifier_to_pool(token);

        match arg {
            ConstantPos::Pos(x) => {
                if self.matches(TokenType::EQUALS_SIGN) {
                    self.expression();
                    self.emit_op_code(OpCode::OP_SET_GLOBAL_VAR);
                } else if self.matches(TokenType::INCREMENT) {
                    self.emit_op_code(OpCode::OP_POST_INCREMENT);
                } else if self.matches(TokenType::DECREMENT) {
                    self.emit_op_code(OpCode::OP_POST_DECREMENT);
                } else {
                    self.emit_op_code(OpCode::OP_GET_GLOBAL_VAR);
                }

                self.emit_short(x);
            }
            ConstantPos::Error => {
                self.error_at_current("Could not add variable name to constant pool.");
            }
        }
    }
}
