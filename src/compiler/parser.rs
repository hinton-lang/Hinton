use std::rc::Rc;

use crate::{
    lexer::{
        tokens::{Token, TokenType},
        Lexer,
    },
    objects::Object,
};

use super::ast::{
    ASTNode, ASTNode::*, BinaryExprNode, BinaryExprType, IdentifierExprNode, LiteralExprNode, ModuleNode, PrintStmtNode, TernaryConditionalNode,
    UnaryExprNode, UnaryExprType,
};

/// Represents Hinton's parser, which converts source text into
/// an Abstract Syntax Tree representation of the program.
pub struct Parser<'a> {
    lexer: Lexer<'a>,
    pub previous: Rc<Token<'a>>,
    pub current: Rc<Token<'a>>,
    pub had_error: bool,
    pub is_in_panic: bool,
}

impl<'a> Parser<'a> {
    /// Parses a string of source test into a Hinton AST.
    ///
    /// ## Arguments
    /// * `src` – The source text
    ///
    /// ## Returns
    /// `Vec<ASTNode<'a>>` – A list of nodes in the AST
    pub fn parse(src: &'a str) -> ModuleNode<'a> {
        // Initialize the compiler
        let mut s = Parser {
            lexer: Lexer::lex(src),
            previous: Rc::new(Token {
                line_num: 0,
                column_num: 0,
                token_type: TokenType::INTERNAL_INIT_HINTON_COMPILER,
                lexeme: "",
            }),
            current: Rc::new(Token {
                line_num: 0,
                column_num: 0,
                token_type: TokenType::INTERNAL_INIT_HINTON_COMPILER,
                lexeme: "",
            }),
            had_error: false,
            is_in_panic: false,
        };

        let mut program = ModuleNode { body: vec![] };

        // Start compiling the chunk
        s.advance();
        while !s.matches(TokenType::EOF) && !s.had_error {
            let x = s.declaration();

            for decl in x.iter() {
                match decl {
                    // TODO: What can we do so that cloning each node is no longer necessary?
                    // Cloning each node is a very expensive operation because some of the nodes
                    // could have an arbitrarily big amount of data. Fox example, large bodies
                    // of literal text could drastically slow down the performance of the compiler
                    // when those strings have to be cloned.
                    Some(val) => program.body.push(val.clone()),
                    None => break,
                }
            }
        }

        return program;
    }

    /// Checks that the current token matches the tokenType provided.
    ///
    /// ## Arguments
    /// * `type` The tokenType we expect to match with the current token.
    pub(super) fn check(&mut self, tok_type: TokenType) -> bool {
        if tok_type == self.get_current_tok_type() {
            true
        } else {
            false
        }
    }

    /// Checks that the current token matches the tokenType provided.
    /// If the the tokens match, the current token gets consumed and the
    /// function returns true. Otherwise, if the tokens do not match,
    /// the token is not consumed, and the function returns false.
    ///
    /// ## Arguments
    /// * `tok_type` The tokenType we expect to match with the current token.
    ///
    /// ## Returns
    /// `bool` – True if the tokens match, false otherwise.
    pub(super) fn matches(&mut self, tok_type: TokenType) -> bool {
        if self.check(tok_type) {
            self.advance();
            return true;
        } else {
            return false;
        }
    }

    /// Advances the compiler to the next token.
    pub(super) fn advance(&mut self) {
        self.previous = Rc::clone(&self.current);

        // We need a loop so that if the current
        // token results in an error token, we can
        loop {
            self.current = self.lexer.next_token();

            match Rc::clone(&self.current).token_type {
                TokenType::ERROR => self.error_at_current("Unexpected token."),
                _ => break,
            }
        }
    }

    /// Consumes the current token only if it is of a given type.
    /// If the token does not match the type, emits a compiler error.
    ///
    /// ## Arguments
    /// * `tok_type` – the expected type of the token to consume.
    /// * `message` – the error message to be displayed if the current token does
    /// not match the provided type.
    pub(super) fn consume(&mut self, tok_type: TokenType, message: &str) {
        if self.check(tok_type) {
            self.advance();
            return ();
        }

        self.error_at_current(message);
    }

    /// Gets the type of the current token.
    ///
    /// ## Returns
    /// `TokenType` – The type of the current token.
    pub(super) fn get_current_tok_type(&self) -> TokenType {
        Rc::clone(&self.current).token_type.clone()
    }

    /// Gets the type of the previous token.
    ///
    /// ## Returns
    /// `TokenType` – The type of the previous token.
    pub(super) fn get_previous_tok_type(&self) -> TokenType {
        self.previous.token_type.clone()
    }

    /// Emits a compiler error from the current token.
    ///
    /// ## Arguments
    /// * `message` – The error message to display.
    pub(super) fn error_at_current(&mut self, message: &str) {
        self.error_at_token(Rc::clone(&self.previous), message);
    }

    /// Emits a compiler error from the previous token.
    ///
    /// ## Arguments
    /// * `message` – The error message to display.
    pub(super) fn error_at_previous(&mut self, message: &str) {
        self.error_at_token(Rc::clone(&self.previous), message);
    }

    /// Emits a compiler error from the given token.
    ///
    /// ## Arguments
    /// *  `tok` – The token that caused the error.
    /// * `message` – The error message to display.
    pub(super) fn error_at_token(&mut self, tok: Rc<Token<'a>>, message: &str) {
        if self.is_in_panic {
            return ();
        }
        self.is_in_panic = true;

        print!("SyntaxError [{}:{}]", tok.line_num, tok.column_num);

        if let TokenType::EOF = tok.token_type {
            println!(" – At the end of the program.");
        } else if let TokenType::ERROR = tok.token_type {
            // Nothing...
        } else {
            print!(" at '{}' – ", tok.lexeme);
        }

        println!("{}", message);
        self.had_error = true;
    }

    /// Synchronizes the compiler when it has found an error.
    /// This method helps minimize the number of cascading errors the compiler emits
    /// when it finds a parsing error. Once it reaches a synchronization point – like
    /// a keyword for a statement – it stops emitting errors.
    pub(super) fn synchronize(&mut self) {
        self.is_in_panic = false;

        while self.get_current_tok_type() != TokenType::EOF {
            if self.get_previous_tok_type() == TokenType::SEMICOLON_SEPARATOR {
                return ();
            }

            match self.get_current_tok_type() {
                TokenType::CLASS_KEYWORD
                | TokenType::FUNC_KEYWORD
                | TokenType::LET_KEYWORD
                | TokenType::FOR_KEYWORD
                | TokenType::IF_KEYWORD
                | TokenType::WHILE_KEYWORD
                | TokenType::PRINT
                | TokenType::RETURN_KEYWORD => {
                    return ();
                }

                _ => {}
            }

            self.advance();
        }
    }

    pub(super) fn declaration(&mut self) -> Vec<Option<ASTNode<'a>>> {
        let mut statements: Vec<Option<ASTNode<'a>>> = Vec::new();

        if self.matches(TokenType::LET_KEYWORD) {
            // statements = varDeclaration();
            todo!("Implement variable declarations.")
        } else if self.matches(TokenType::CONST_KEYWORD) {
            // statements = constDeclaration();
            todo!("Implement constant declarations")
        } else if self.matches(TokenType::FUNC_KEYWORD) {
            // statements.add(function());
            todo!("Implement function declarations")
        } else if self.matches(TokenType::ENUM_KEYWORD) {
            // statements.add(enumDeclaration());
            todo!("Implement enum declarations")
        } else {
            statements.push(self.statement());
        }

        return statements;
        // if self.is_in_panic {
        //     self.synchronize();
        // }
    }

    pub(super) fn statement(&mut self) -> Option<ASTNode<'a>> {
        if self.matches(TokenType::LEFT_CURLY_BRACES) {
            todo!("Implement blocks")
        } else if self.matches(TokenType::IF_KEYWORD) {
            todo!("Implement `if` statements")
        } else if self.matches(TokenType::WHILE_KEYWORD) {
            todo!("Implement while loops")
        } else if self.matches(TokenType::FOR_KEYWORD) {
            todo!("Implement for loops")
        } else if self.matches(TokenType::BREAK_KEYWORD) {
            todo!("Implement breaks")
        } else if self.matches(TokenType::CONTINUE_KEYWORD) {
            todo!("Implement continue")
        } else if self.matches(TokenType::RETURN_KEYWORD) {
            todo!("Implement return")
        } else if self.matches(TokenType::PRINT) {
            self.print_statement()
        } else {
            self.expression_statement()
        }
    }

    pub(super) fn print_statement(&mut self) -> Option<ASTNode<'a>> {
        let opr = Rc::clone(&self.previous);

        self.consume(TokenType::LEFT_PARENTHESIS, "Expected '(' before expression.");
        let expr = self.expression();
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

    pub(super) fn expression_statement(&mut self) -> Option<ASTNode<'a>> {
        let opr = Rc::clone(&self.previous);
        let expr = self.expression();

        self.consume(TokenType::SEMICOLON_SEPARATOR, "Expected ';' after expression.");

        return Some(PrintStmt(PrintStmtNode {
            child: match expr {
                Some(t) => Box::new(t),
                None => return None, // Could not create expression to print
            },
            pos: (opr.line_num, opr.column_num),
        }));
    }

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
        let expr = self.ternary_conditional();

        return expr;
    }

    /// Parses a ternary conditional expression as specified in the grammar.cfg file.
    ///
    /// ## Returns
    /// `Option<ASTNode<'a>>` – The assignment expression's AST node.
    pub(super) fn ternary_conditional(&mut self) -> Option<ASTNode<'a>> {
        let mut expr = self.logic_or();

        if self.matches(TokenType::QUESTION_MARK) {
            let opr = Rc::clone(&self.previous);

            let branch_true = match self.expression() {
                Some(t) => t,
                None => return None, // Could not create expression for branch_true
            };

            self.consume(TokenType::COLON_SEPARATOR, "Expected ':' in ternary operator.");

            let branch_false = match self.expression() {
                Some(t) => t,
                None => return None, // Could not create expression for branch_false
            };

            expr = Some(TernaryConditional(TernaryConditionalNode {
                condition: match expr {
                    Some(e) => Box::new(e),
                    None => return None, // Could not create conditional expression
                },
                branch_true: Box::new(branch_true),
                branch_false: Box::new(branch_false),
                pos: (opr.line_num, opr.column_num),
            }));
        }

        return expr;
    }

    /// Parses an 'OR' expression as specified in the grammar.cfg file.
    ///
    /// ## Returns
    /// `Option<ASTNode<'a>>` – The expression's AST node.
    pub(super) fn logic_or(&mut self) -> Option<ASTNode<'a>> {
        let mut expr = self.logic_and();

        while self.matches(TokenType::LOGICAL_OR) {
            let opr = Rc::clone(&self.previous);

            expr = Some(Binary(BinaryExprNode {
                left: match expr {
                    Some(e) => Box::new(e),
                    None => return None, // Could not create lhs of expression
                },
                right: match self.logic_and() {
                    Some(e) => Box::new(e),
                    None => return None, // Could not create rhs of expression
                },
                pos: (opr.line_num, opr.column_num),
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
        let mut expr = self.bitwise_or();

        while self.matches(TokenType::LOGICAL_AND) {
            let opr = Rc::clone(&self.previous);

            expr = Some(Binary(BinaryExprNode {
                left: match expr {
                    Some(e) => Box::new(e),
                    None => return None, // Could not create lhs of expression
                },
                right: match self.bitwise_or() {
                    Some(e) => Box::new(e),
                    None => return None, // Could not create rhs of expression
                },
                pos: (opr.line_num, opr.column_num),
                opr_type: BinaryExprType::LogicAND,
            }));
        }

        return expr;
    }

    /// Parses a 'BITWISE OR' expression as specified in the grammar.cfg file.
    ///
    /// ## Returns
    /// `Option<ASTNode<'a>>` – The expression's AST node.
    pub(super) fn bitwise_or(&mut self) -> Option<ASTNode<'a>> {
        let mut expr = self.bitwise_xor();

        while self.matches(TokenType::BITWISE_OR) {
            let opr = Rc::clone(&self.previous);

            expr = Some(Binary(BinaryExprNode {
                left: match expr {
                    Some(e) => Box::new(e),
                    None => return None, // Could not create lhs of expression
                },
                right: match self.bitwise_xor() {
                    Some(e) => Box::new(e),
                    None => return None, // Could not create rhs of expression
                },
                pos: (opr.line_num, opr.column_num),
                opr_type: BinaryExprType::BitwiseOR,
            }));
        }

        return expr;
    }

    /// Parses a 'BITWISE XOR' expression as specified in the grammar.cfg file.
    ///
    /// ## Returns
    /// `Option<ASTNode<'a>>` – The expression's AST node.
    pub(super) fn bitwise_xor(&mut self) -> Option<ASTNode<'a>> {
        let mut expr = self.bitwise_and();

        while self.matches(TokenType::BITWISE_XOR) {
            let opr = Rc::clone(&self.previous);

            expr = Some(Binary(BinaryExprNode {
                left: match expr {
                    Some(e) => Box::new(e),
                    None => return None, // Could not create lhs of expression
                },
                right: match self.bitwise_and() {
                    Some(e) => Box::new(e),
                    None => return None, // Could not create rhs of expression
                },
                pos: (opr.line_num, opr.column_num),
                opr_type: BinaryExprType::BitwiseXOR,
            }));
        }

        return expr;
    }

    /// Parses a 'BITWISE AND' expression as specified in the grammar.cfg file.
    ///
    /// ## Returns
    /// `Option<ASTNode<'a>>` – The expression's AST node.
    pub(super) fn bitwise_and(&mut self) -> Option<ASTNode<'a>> {
        let mut expr = self.equality();

        while self.matches(TokenType::BITWISE_AND) {
            let opr = Rc::clone(&self.previous);

            expr = Some(Binary(BinaryExprNode {
                left: match expr {
                    Some(e) => Box::new(e),
                    None => return None, // Could not create lhs of expression
                },
                right: match self.equality() {
                    Some(e) => Box::new(e),
                    None => return None, // Could not create rhs of expression
                },
                pos: (opr.line_num, opr.column_num),
                opr_type: BinaryExprType::BitwiseAND,
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
            let opr = Rc::clone(&self.previous);

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
                right: match self.comparison() {
                    Some(e) => Box::new(e),
                    None => return None, // Could not create rhs of expression
                },
                pos: (opr.line_num, opr.column_num),
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
            let opr = Rc::clone(&self.previous);

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
                right: match self.range() {
                    Some(e) => Box::new(e),
                    None => return None, // Could not create rhs of expression
                },
                pos: (opr.line_num, opr.column_num),
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
        let mut expr = self.bitwise_shift();

        if self.matches(TokenType::RANGE_OPERATOR) {
            let opr = Rc::clone(&self.previous);

            expr = Some(Binary(BinaryExprNode {
                left: match expr {
                    Some(e) => Box::new(e),
                    None => return None, // Could not create lhs of expression
                },
                right: match self.bitwise_shift() {
                    Some(e) => Box::new(e),
                    None => return None, // Could not create rhs of expression
                },
                pos: (opr.line_num, opr.column_num),
                opr_type: BinaryExprType::Range,
            }));
        }

        return expr;
    }

    /// Parses a 'BITWISE SHIFT' expression as specified in the grammar.cfg file.
    ///
    /// ## Returns
    /// `Option<ASTNode<'a>>` – The expression's AST node.
    pub(super) fn bitwise_shift(&mut self) -> Option<ASTNode<'a>> {
        let mut expr = self.term();

        while self.matches(TokenType::BITWISE_LEFT_SHIFT) || self.matches(TokenType::BITWISE_RIGHT_SHIFT) {
            let opr = Rc::clone(&self.previous);

            let opr_type = if opr.token_type == TokenType::BITWISE_LEFT_SHIFT {
                BinaryExprType::BitwiseShiftLeft
            } else {
                BinaryExprType::BitwiseShiftRight
            };

            expr = Some(Binary(BinaryExprNode {
                left: match expr {
                    Some(e) => Box::new(e),
                    None => return None, // Could not create lhs of expression
                },
                right: match self.term() {
                    Some(e) => Box::new(e),
                    None => return None, // Could not create rhs of expression
                },
                pos: (opr.line_num, opr.column_num),
                opr_type: opr_type,
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
            let opr = Rc::clone(&self.previous);

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
                right: match self.factor() {
                    Some(e) => Box::new(e),
                    None => return None, // Could not create rhs of expression
                },
                pos: (opr.line_num, opr.column_num),
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
            let opr = Rc::clone(&self.previous);

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
                right: match self.expo() {
                    Some(e) => Box::new(e),
                    None => return None, // Could not create rhs of expression
                },
                pos: (opr.line_num, opr.column_num),
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
            let opr = Rc::clone(&self.previous);

            expr = Some(Binary(BinaryExprNode {
                left: match expr {
                    Some(e) => Box::new(e),
                    None => return None, // Could not create lhs of expression
                },
                right: match self.unary() {
                    Some(e) => Box::new(e),
                    None => return None, // Could not create rhs of expression
                },
                pos: (opr.line_num, opr.column_num),
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
            let opr = Rc::clone(&self.previous);
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
                pos: (opr.line_num, opr.column_num),
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
            TokenType::LEFT_PARENTHESIS => {
                let expr = self.expression();
                self.consume(TokenType::RIGHT_PARENTHESIS, "Expected closing ')'.");
                // For grouping expression, we don't wrap the inner expression inside a literal.
                // Instead, we return the actual expression that was enclosed in the parenthesis.
                return expr;
            }
            TokenType::IDENTIFIER => {
                // For identifier expressions, the only information we need is enclosed within the token.
                // So we return the token wrapped inside an ASTNode::Identifier.
                return Some(Identifier(IdentifierExprNode {
                    token: Rc::clone(&self.previous),
                }));
            }
            _ => {
                self.error_at_previous("Unexpected token '{}'.");
                return None;
            }
        };

        let node = LiteralExprNode {
            value: literal_value,
            token: Rc::clone(&self.current),
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
                // This should almost never happen.
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
                // This should almost never happen.
                self.error_at_previous("Unexpected token.");
                Err(())
            }
        };
    }
}
