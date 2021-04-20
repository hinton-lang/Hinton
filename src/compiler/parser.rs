use std::rc::Rc;

use crate::lexer::{
    tokens::{Token, TokenType},
    Lexer,
};

use super::ast::ASTNode;

pub struct Parser<'a> {
    lexer: Lexer<'a>,
    pub previous: Rc<Token<'a>>,
    pub current: Rc<Token<'a>>,
    had_error: bool,
    is_in_panic: bool,
}

pub fn parse<'a>(src: &'a str) -> Vec<ASTNode<'a>> {
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

    let mut program: Vec<ASTNode<'a>> = Vec::new();

    // Start compiling the chunk
    s.advance();
    while !s.matches(TokenType::EOF) && !s.had_error {
        let x = s.expression();

        match x {
            Some(val) => program.push(val),
            None => break,
        }
    }

    return program;
}

impl<'a> Parser<'a> {
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
        self.previous = self.current.clone();

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
}
