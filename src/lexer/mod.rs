use std::rc::Rc;

use self::tokens::{Token, TokenType, TokenType::*, KEYWORDS};

// Tokens-specific module implementations
pub mod tokens;

// Lexer-specific module implementations
pub mod lex_next_token;
pub mod lex_numbers;

/// Struct that represents the scanner.
pub struct Lexer<'a> {
    source: &'a str,
    current: usize,
    size: usize,
    line: usize,
    line_start: usize,
    token_start: usize,
}

impl<'a> Lexer<'a> {
    /// An initialized instance of the lexer.
    /// ## Arguments
    /// * `src` - the source file
    ///
    /// ## Example
    /// ```
    /// let mut l = Lexer::lex("let x = 22;");
    /// ```
    pub fn lex(src: &'a str) -> Self {
        Self {
            source: &src,
            current: 0,
            size: src.len(),
            line: 1,
            line_start: 0,
            token_start: 0,
        }
    }

    /// Gets the previously consumed character.
    ///
    /// ## Returns
    /// * `Option<char>` – The next character.
    pub(super) fn previous(&self) -> Option<char> {
        return self.source.chars().nth(self.current - 1);
    }

    /// Gets the current character without consuming it.
    ///
    /// ## Returns
    /// * `Option<char>` – The current character.
    pub(super) fn get_current(&self) -> Option<char> {
        return self.source.chars().nth(self.current);
    }

    /// Returns the next character without consuming it.
    ///
    /// ## Returns
    /// * `Option<char>` – The previous character.
    pub(super) fn next(&self) -> Option<char> {
        if self.is_at_end() {
            return Some('\0');
        }
        return self.source.chars().nth(self.current + 1);
    }

    /// Checks if the scanner is at the end of the source.
    ///
    /// ## Returns
    /// * `bool` – True if the scanner is at the end of the source, false otherwise.
    pub(super) fn is_at_end(&self) -> bool {
        return self.current >= self.size;
    }

    /// Matches the current character against a provided character
    ///
    /// ## Returns
    /// * `bool` – True if the current character matched the provided
    /// character, false otherwise.
    pub(super) fn matches(&mut self, expected: Option<char>) -> bool {
        if self.is_at_end() || self.get_current() != expected {
            return false;
        }
        self.current += 1;
        return true;
    }

    /// Advances to the next token
    ///
    /// ## Returns
    /// * `Option<char>` – The consumed character.
    pub(super) fn advance(&mut self) -> Option<char> {
        let c = self.get_current();
        self.current += 1;
        return c;
    }

    /// Skips whitespace-like characters from the source code.
    pub(super) fn skip_whitespace(&mut self) {
        loop {
            let c = self.get_current();

            if c == Some(' ') || c == Some('\r') || c == Some('\t') {
                self.advance();
            } else if c == Some('\n') {
                self.line += 1;
                self.line_start = self.current + 1;
                self.advance();
            } else if c == Some('/') && (self.next() == Some('/') || self.next() == Some('*')) {
                self.skip_comments();
            } else {
                break;
            }
        }
    }

    /// Skips single-line and block comments from from the source code.
    /// ### TODO Allow nesting block comments
    pub(self) fn skip_comments(&mut self) {
        // single-line comments
        if self.get_current() == Some('/') && self.next() == Some('/') {
            while self.get_current() != Some('\n') && !self.is_at_end() {
                self.advance();
            }
        }

        // block comments
        if self.get_current() == Some('/') && self.next() == Some('*') {
            self.advance();
            self.advance();

            loop {
                // Break if we are at the end of the comment.
                if self.get_current() == Some('*') && self.next() == Some('/') {
                    self.advance();
                    self.advance();
                    break;
                }

                // Break if we have reached the end of the program
                if self.is_at_end() {
                    break;
                };

                // Take into account new lines inside block comments
                if self.get_current() == Some('\n') {
                    self.line += 1;
                }

                // Skip everything inside the comment
                self.advance();
            }
        }

        // Reposition the start of the token to
        // be after the comment has ended
        self.token_start = self.current;
    }

    /// Makes a string literal.
    ///
    /// ## Returns
    /// * `Token` – A string token.
    pub(super) fn make_string_token(&mut self) -> Rc<Token<'a>> {
        // The opener single or double quote.
        let quote = self.previous();

        // Keep consuming characters until there is an unescaped quote or
        // the program reaches the end of the source file.
        while (self.get_current() != quote || (self.get_current() == quote && self.previous() == Some('\\'))) && !self.is_at_end() {
            self.advance();
        }

        if self.is_at_end() {
            return self.make_error_token("Unterminated string.");
        }

        // The closing quote.
        self.advance();

        return self.make_token(STRING_LITERAL);
    }

    /// Generates an identifier token with the current state of the scanner.
    ///
    /// ## Returns
    /// * `Token` – An identifier token.
    pub(super) fn make_identifier_token(&mut self) -> Rc<Token<'a>> {
        while !self.is_at_end() {
            let c = self.get_current().unwrap();

            if c.is_alphabetic() || c.is_digit(10) {
                self.advance();
            } else {
                break;
            }
        }

        let id = &self.source[(self.token_start)..(self.current)];
        if KEYWORDS.contains_key(id) {
            return self.make_token(KEYWORDS.get(id).unwrap().clone());
        } else {
            return self.make_token(IDENTIFIER);
        }
    }

    /// Generates a token with the current state of the scanner
    /// ## Arguments
    /// * `tok_type` – The type of the token to generate.
    ///
    /// ## Returns
    /// * `Token` – The generated token (of any type).
    ///
    /// ## Example
    /// ```
    /// // This will generate a token of type `VAR_KEYWORD`
    /// // from the current state of the scanner.
    /// self.make_token(VAR_KEYWORD);
    /// ```
    pub(super) fn make_token(&self, tok_type: TokenType) -> Rc<Token<'a>> {
        return Rc::new(Token {
            line_num: self.line,
            column_num: if tok_type != EOF {
                self.token_start - self.line_start
            } else {
                self.current
            },
            token_type: tok_type.clone(),
            // If the token is the EOF token, then the lexeme becomes the null terminator.
            // It is okay to use the null terminator for the EOF value because the EOF Token's
            // lexeme is never used for anything.
            lexeme: if tok_type != EOF {
                &self.source[(self.token_start)..(self.current)]
            } else {
                "\0"
            },
        });
    }

    /// Generates an error token with the provided message as its lexeme.
    ///
    /// ## Arguments
    /// * `message` – A message to display for the error token. This will
    /// be used as the token's lexeme.
    ///
    /// ## Returns
    /// * `Token` – The generated error token.
    pub(super) fn make_error_token(&self, message: &'a str) -> Rc<Token<'a>> {
        return Rc::new(Token {
            line_num: self.line,
            column_num: self.token_start - self.line_start,
            token_type: ERROR,
            lexeme: message,
        });
    }
}
