use crate::{
    ast::*,
    errors::ErrorReport,
    lexer::{
        tokens::{Token, TokenType, TokenType::*},
        Lexer,
    },
};

// Submodules
mod expressions;
mod statements;

/// Represents Hinton's parser, which converts source text into
/// an Abstract Syntax Tree representation of the program.
pub struct Parser {
    lexer: Lexer,
    previous: Token,
    current: Token,
    is_in_panic: bool,
    errors: Vec<ErrorReport>,
}

impl<'a> Parser {
    /// Parses a string of source test into a Hinton AST.
    ///
    /// ## Arguments
    /// * `src` – The source text
    ///
    /// ## Returns
    /// `Vec<ASTNode>` – A list of nodes in the AST
    pub fn parse(src: &'a str) -> Result<ASTNode, Vec<ErrorReport>> {
        // Initialize the compiler
        let mut parser = Parser {
            lexer: Lexer::lex(src),
            previous: Token {
                line_num: 0,
                column_num: 0,
                token_type: __INIT_PARSER__,
                lexeme: String::from(""),
            },
            current: Token {
                line_num: 0,
                column_num: 0,
                token_type: __INIT_PARSER__,
                lexeme: String::from(""),
            },
            is_in_panic: false,
            errors: vec![],
        };

        let mut program = ModuleNode { body: vec![] };

        // Start compiling the chunk
        parser.advance();
        while !parser.matches(&EOF) {
            match parser.parse_declaration() {
                Some(val) => program.body.push(val),
                None => {
                    // If there was an error, continue parsing
                    // to catch other errors in the program, but
                    // the AST will (of course) not be usable.
                }
            }
        }

        return if parser.errors.len() > 0 {
            Err(parser.errors)
        } else {
            Ok(ASTNode::Module(program))
        };
    }

    /// Checks that the current token matches the tokenType provided.
    ///
    /// ## Arguments
    /// * `type` The tokenType we expect to match with the current token.
    ///
    /// # Results
    /// * `bool` – True if the current token matches the given token type
    /// false otherwise.
    fn check(&mut self, tok_type: &TokenType) -> bool {
        self.get_current_tok_type().type_match(tok_type)
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
    fn matches(&mut self, tok_type: &TokenType) -> bool {
        if self.check(tok_type) {
            self.advance();
            return true;
        } else {
            return false;
        }
    }

    /// Advances the compiler to the next token.
    fn advance(&mut self) {
        self.previous = self.current.clone();

        // We need a loop so that if the current
        // token results in an error token, we can
        loop {
            self.current = self.lexer.next_token();

            match self.current.token_type {
                ERROR => self.error_at_current("Unexpected token."),
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
    fn consume(&mut self, tok_type: &TokenType, message: &str) {
        if self.check(tok_type) {
            self.advance();
            return;
        }

        if let SEMICOLON = tok_type {
            self.error_at_previous(message);
        } else {
            self.error_at_current(message);
        }
    }

    /// Gets the type of the current token.
    ///
    /// ## Returns
    /// `TokenType` – The type of the current token.
    fn get_current_tok_type(&self) -> &TokenType {
        &self.current.token_type
    }

    /// Gets the type of the previous token.
    ///
    /// ## Returns
    /// `TokenType` – The type of the previous token.
    fn get_previous_tok_type(&self) -> TokenType {
        self.previous.token_type.clone()
    }

    /// Emits a compiler error from the current token.
    ///
    /// ## Arguments
    /// * `message` – The error message to display.
    fn error_at_current(&mut self, message: &str) {
        self.error_at_token(&self.current.clone(), message);
    }

    /// Emits a compiler error from the previous token.
    ///
    /// ## Arguments
    /// * `message` – The error message to display.
    fn error_at_previous(&mut self, message: &str) {
        self.error_at_token(&self.previous.clone(), message);
    }

    /// Emits a compiler error from the given token.
    ///
    /// ## Arguments
    /// * `tok` – The token that caused the error.
    /// * `message` – The error message to display.
    fn error_at_token(&mut self, tok: &Token, message: &str) {
        if self.is_in_panic {
            return ();
        }
        self.is_in_panic = true;

        // Construct the error message.
        let msg = format!(
            "\x1b[31;1mSyntaxError\x1b[0m\x1b[1m at [{}:{}]: {}\x1b[0m",
            tok.line_num, tok.column_num, message
        );

        // Push the error to the list
        self.errors.push(ErrorReport {
            line: tok.line_num,
            column: tok.column_num,
            lexeme_len: tok.lexeme.len(),
            message: msg,
        });
    }

    /// Synchronizes the compiler when it has found an error.
    /// This method helps minimize the number of cascading errors the compiler emits
    /// when it finds a parsing error. Once it reaches a synchronization point – like
    /// a keyword for a statement – it stops emitting errors.
    fn synchronize(&mut self) {
        self.is_in_panic = false;

        while !self.get_current_tok_type().type_match(&EOF) {
            if let SEMICOLON = self.get_previous_tok_type() {
                return;
            }

            match self.get_current_tok_type() {
                CLASS_KW | FUNC_KW | LET_KW | FOR_KW | IF_KW | WHILE_KW | RETURN_KW => {
                    return;
                }

                _ => {}
            }

            self.advance();
        }
    }
}
