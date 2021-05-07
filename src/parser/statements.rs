use super::Parser;
use std::{rc::Rc, vec};

use crate::{
    ast::ASTNode::*,
    ast::*,
    lexer::tokens::{Token, TokenType},
    objects::Object,
};

impl Parser {
    /// Parses a declaration as specified in the grammar.cfg file.
    ///
    /// ## Returns
    /// `Option<ASTNode>` – The declaration's AST node.
    pub(super) fn parse_declaration(&mut self) -> Option<ASTNode> {
        if self.matches(TokenType::LET_KEYWORD) {
            return self.parse_var_declaration();
        } else if self.matches(TokenType::CONST_KEYWORD) {
            return self.parse_const_declaration();
        } else if self.matches(TokenType::FUNC_KEYWORD) {
            return self.parse_func_declaration();
        } else if self.matches(TokenType::ENUM_KEYWORD) {
            // statements.add(enumDeclaration());
            todo!("Implement enum declarations")
        } else {
            return self.parse_statement();
        }

        // if self.is_in_panic {
        //     self.synchronize();
        // }
    }

    /// Parses a statement as specified in the grammar.cfg file.
    ///
    /// ## Returns
    /// `Option<ASTNode>` – The statement's AST node.
    fn parse_statement(&mut self) -> Option<ASTNode> {
        if self.matches(TokenType::LEFT_CURLY_BRACES) {
            self.parse_block()
        } else if self.matches(TokenType::IF_KEYWORD) {
            self.parse_if_statement()
        } else if self.matches(TokenType::WHILE_KEYWORD) {
            self.parse_while_statement()
        } else if self.matches(TokenType::FOR_KEYWORD) {
            todo!("Implement for loops")
        } else if self.matches(TokenType::BREAK_KEYWORD) {
            let tok = Rc::clone(&self.previous);
            self.consume(TokenType::SEMICOLON_SEPARATOR, "Expected ';' after break keyword.");
            return Some(BreakStmt(BreakStmtNode { token: tok }));
        } else if self.matches(TokenType::CONTINUE_KEYWORD) {
            todo!("Implement continue")
        } else if self.matches(TokenType::RETURN_KEYWORD) {
            self.parse_return_stmt()
        } else if self.matches(TokenType::PRINT) {
            self.parse_print_statement()
        } else {
            self.parse_expression_statement()
        }
    }

    /// Parses a print statement.
    ///
    /// ## Returns
    /// `Option<ASTNode>` – The print statement's AST node.
    fn parse_print_statement(&mut self) -> Option<ASTNode> {
        let opr = Rc::clone(&self.previous);

        self.consume(TokenType::LEFT_PARENTHESIS, "Expected '(' before expression.");
        let expr = self.parse_expression();
        self.consume(TokenType::RIGHT_PARENTHESIS, "Expected ')' after expression.");
        self.consume(TokenType::SEMICOLON_SEPARATOR, "Expected ';' after expression.");

        return Some(PrintStmt(PrintStmtNode {
            child: match expr {
                Some(t) => Box::new(t),
                None => return None, // Could not create expression to print
            },
            pos: (opr.line_num, opr.column_num),
        }));
    }

    /// Parses an expression statement as specified in the grammar.cfg file.
    ///
    /// ## Returns
    /// `Option<ASTNode>` – The expression statement's AST node.
    fn parse_expression_statement(&mut self) -> Option<ASTNode> {
        let opr = Rc::clone(&self.previous);
        let expr = self.parse_expression();

        self.consume(TokenType::SEMICOLON_SEPARATOR, "Expected ';' after expression.");

        return Some(ExpressionStmt(ExpressionStmtNode {
            child: match expr {
                Some(t) => Box::new(t),
                None => return None, // Could not create expression to print
            },
            pos: (opr.line_num, opr.column_num),
        }));
    }

    /// Parses a block statement.
    ///
    /// ## Returns
    /// `Option<ASTNode>` – The block statement's AST node.
    fn parse_block(&mut self) -> Option<ASTNode> {
        let mut statements = BlockNode { body: vec![] };

        while !self.check(TokenType::RIGHT_CURLY_BRACES) && !self.check(TokenType::EOF) {
            match self.parse_declaration() {
                Some(val) => statements.body.push(val),
                // Report parse error if node has None value
                None => return None,
            }
        }

        self.consume(TokenType::RIGHT_CURLY_BRACES, "Expect '}' after block.");
        return Some(BlockStmt(statements));
    }

    /// Parses a variable declaration as specified in the grammar.cfg file.
    ///
    /// Returns
    /// * `Option<ASTNode>` – A variable declaration AST node.
    fn parse_var_declaration(&mut self) -> Option<ASTNode> {
        let mut declarations: Vec<Rc<Token>> = Vec::new();

        // Gets at least one variable name, or a list of
        // names separated by a comma
        self.consume(TokenType::IDENTIFIER, "Expected variable name.");
        declarations.push(Rc::clone(&self.previous));

        while self.matches(TokenType::COMMA_SEPARATOR) {
            self.consume(TokenType::IDENTIFIER, "Expected variable name.");
            declarations.push(Rc::clone(&self.previous));
        }

        // Gets the variable's value.
        let initializer = if self.matches(TokenType::EQUALS_SIGN) {
            match self.parse_expression() {
                Some(val) => val,
                None => return None, // Could not create value for variable
            }
        } else {
            ASTNode::Literal(LiteralExprNode {
                value: Object::Null,
                token: Rc::clone(&self.previous),
            })
        };

        // Requires a semicolon at the end of the declaration if the declaration
        // was not a block (e.g., when assigning a lambda function to a variable).
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

    /// Parses a variable declaration as specified in the grammar.cfg file.
    ///
    /// Returns
    /// * `Option<ASTNode>` – A variable declaration AST node.
    fn parse_const_declaration(&mut self) -> Option<ASTNode> {
        self.consume(TokenType::IDENTIFIER, "Expected variable name.");
        let name = Rc::clone(&self.previous);

        self.consume(TokenType::EQUALS_SIGN, "Constants must have a value.");

        let initializer = match self.parse_expression() {
            Some(val) => val,
            None => return None, // Could not create value for variable
        };

        // Requires a semicolon at the end of the declaration if the declaration
        // was not a block (e.g., when assigning a lambda function to a constant).
        if self.previous.token_type != TokenType::RIGHT_CURLY_BRACES {
            self.consume(TokenType::SEMICOLON_SEPARATOR, "Expected ';' after variable declaration.");
        }

        // But if there is a semicolon after a curly brace, then we consume it
        if self.previous.token_type == TokenType::RIGHT_CURLY_BRACES && self.check(TokenType::SEMICOLON_SEPARATOR) {
            self.advance();
        }

        return Some(ASTNode::ConstantDecl(ConstantDeclNode {
            name,
            value: Box::new(initializer.clone()),
        }));
    }

    /// Parses an if statement as specified in the grammar.cfg file.
    ///
    /// Returns
    /// * `Option<ASTNode>` – An if statement AST node.
    fn parse_if_statement(&mut self) -> Option<ASTNode> {
        let then_tok = Rc::clone(&self.previous);
        self.consume(TokenType::LEFT_PARENTHESIS, "Expected '(' after 'if'.");

        let condition = match self.parse_expression() {
            Some(val) => val,
            None => return None, // Could not create condition for if-statement
        };

        self.consume(TokenType::RIGHT_PARENTHESIS, "Expected ')' after 'if' condition.");

        let then_branch = match self.parse_statement() {
            Some(val) => val,
            None => return None, // Could not create then branch
        };

        let mut else_branch = None;
        let mut else_tok = None;
        if self.matches(TokenType::ELSE_KEYWORD) {
            else_tok = Some(Rc::clone(&self.previous));

            else_branch = match self.parse_statement() {
                Some(val) => Some(val),
                None => return None, // Could not create else branch
            };
        }

        return Some(IfStmt(IfStmtNode {
            condition: Box::new(condition),
            then_token: then_tok,
            then_branch: Box::new(then_branch),
            else_branch: Box::new(else_branch),
            else_token: else_tok,
        }));
    }

    fn parse_while_statement(&mut self) -> Option<ASTNode> {
        let tok = Rc::clone(&self.previous);
        self.consume(TokenType::LEFT_PARENTHESIS, "Expected '(' after 'while'.");

        let condition = match self.parse_expression() {
            Some(val) => val,
            None => return None, // Could not create condition for while-loop
        };

        self.consume(TokenType::RIGHT_PARENTHESIS, "Expected ')' after 'while' condition.");

        let body = match self.parse_statement() {
            Some(val) => val,
            None => return None, // Could not create then branch
        };

        return Some(WhileStmt(WhileStmtNode {
            token: Rc::clone(&tok),
            condition: Box::new(condition),
            body: Box::new(body),
        }));
    }

    fn parse_func_declaration(&mut self) -> Option<ASTNode> {
        self.consume(TokenType::IDENTIFIER, "Expected a function name.");
        let name = Rc::clone(&self.previous);
        self.consume(TokenType::LEFT_PARENTHESIS, "Expected opening parenthesis after function name.");

        let mut params: Vec<Parameter> = vec![];
        let mut min_arity: u8 = 0;
        let mut max_arity: u8 = 0;

        while !self.matches(TokenType::RIGHT_PARENTHESIS) {
            if params.len() > 255 {
                self.error_at_current("Can't have more than 255 parameters.");
                return None;
            }

            match self.parse_parameter() {
                Some(p) => {
                    if params.len() > 0 && !p.is_optional && params.last().unwrap().is_optional {
                        self.error_at_token(
                            Rc::clone(&params.last().unwrap().name),
                            "Optional and named parameters must be declared after all required parameters.",
                        );
                        return None;
                    }

                    max_arity += 1;

                    if !p.is_optional {
                        min_arity += 1
                    }

                    params.push(p);
                }
                None => return None, // Could not parse the parameter
            }

            if !self.matches(TokenType::RIGHT_PARENTHESIS) {
                self.consume(TokenType::COMMA_SEPARATOR, "Expected comma after parameter.");
            } else {
                break;
            }
        }

        self.consume(TokenType::LEFT_CURLY_BRACES, "Expected opening curly braces before function body.");

        let body = match self.parse_block() {
            Some(b) => b,
            None => return None,
        };

        return Some(FunctionDecl(FunctionDeclNode {
            name,
            params,
            min_arity,
            max_arity,
            body: Box::new(body),
        }));
    }

    fn parse_parameter(&mut self) -> Option<Parameter> {
        self.consume(TokenType::IDENTIFIER, "Expected a parameter name.");
        let name = Rc::clone(&self.previous);

        if self.matches(TokenType::QUESTION_MARK) {
            return Some(Parameter {
                name,
                is_optional: true,
                default: None,
            });
        }

        if self.matches(TokenType::COLON_EQUALS) {
            return Some(Parameter {
                name,
                is_optional: true,
                default: match self.parse_expression() {
                    Some(x) => Some(Box::new(x)),
                    None => return None, // Could not compile default value for parameter
                },
            });
        }

        Some(Parameter {
            name,
            is_optional: false,
            default: None,
        })
    }

    fn parse_return_stmt(&mut self) -> Option<ASTNode> {
        let tok = Rc::clone(&self.previous);

        // Compiles the return expression
        if !self.matches(TokenType::SEMICOLON_SEPARATOR) {
            let expr = match self.parse_expression() {
                Some(val) => val,
                // Report parse error if node has None value
                None => return None,
            };

            self.consume(TokenType::SEMICOLON_SEPARATOR, "Expected ';' after break keyword.");
            return Some(ReturnStmt(ReturnStmtNode {
                token: tok,
                value: Some(Box::new(expr)),
            }));
        }

        return Some(ReturnStmt(ReturnStmtNode { token: tok, value: None }));
    }
}
