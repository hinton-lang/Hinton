use super::{tokens::Token, tokens::TokenType::*, Lexer};

impl<'a> Lexer {
    /// Scans the next token in the source file.
    pub fn next_token(&mut self) -> Token {
        self.skip_whitespace();

        // If we are at the end, return the EOF token.
        if self.is_at_end() {
            return self.make_token(EOF);
        }

        // Reset the start of the token
        self.token_start = self.current;
        let c = self.advance();

        // Generates an identifier/keyword if the current character is alphanumeric
        if c.is_alphabetic() {
            return self.make_identifier_token();
        }

        // Generates an numeric literal if the current character is alphanumeric
        if c.is_digit(10) {
            return self.make_numeric_token();
        }

        // Generate symbol-like token tokens
        return match c {
            '"' | '\'' => self.make_string_token(),
            '(' => self.make_token(LEFT_PARENTHESIS),
            ')' => self.make_token(RIGHT_PARENTHESIS),
            '{' => self.make_token(LEFT_CURLY_BRACES),
            '}' => self.make_token(RIGHT_CURLY_BRACES),
            '[' => self.make_token(LEFT_SQUARE_BRACKET),
            ']' => self.make_token(RIGHT_SQUARE_BRACKET),
            ';' => self.make_token(SEMICOLON_SEPARATOR),
            ',' => self.make_token(COMMA_SEPARATOR),
            '~' => self.make_token(BITWISE_NOT),
            '/' => {
                let tok = if self.matches('=') {
                    SLASH_EQUALS
                } else {
                    SLASH
                };
                self.make_token(tok)
            }
            '%' => {
                let tok = if self.matches('=') {
                    MOD_EQUALS
                } else {
                    MODULUS
                };
                self.make_token(tok)
            }
            '!' => {
                let tok = if self.matches('=') {
                    LOGICAL_NOT_EQ
                } else {
                    LOGICAL_NOT
                };
                self.make_token(tok)
            }
            '=' => {
                let tok = if self.matches('=') {
                    LOGICAL_EQ
                } else {
                    EQUALS_SIGN
                };
                self.make_token(tok)
            }
            ':' => {
                let tok = if self.matches('=') {
                    COLON_EQUALS
                } else {
                    COLON_SEPARATOR
                };
                self.make_token(tok)
            }
            '^' => {
                let tok = if self.matches('=') {
                    BITWISE_XOR_EQUALS
                } else {
                    BITWISE_XOR
                };
                self.make_token(tok)
            }
            '&' => {
                let tok = if self.matches('&') {
                    LOGICAL_AND
                } else if self.matches('=') {
                    BITWISE_AND_EQUALS
                } else {
                    BITWISE_AND
                };
                self.make_token(tok)
            }
            '|' => {
                let tok = if self.matches('|') {
                    LOGICAL_OR
                } else if self.matches('=') {
                    BITWISE_OR_EQUALS
                } else {
                    BITWISE_OR
                };
                self.make_token(tok)
            }
            '?' => {
                if self.matches('?') {
                    self.make_token(NULLISH_COALESCING)
                }
                // else if self.matches(':') {
                //     self.make_token(ELVIS_OPERATOR)
                // }
                else {
                    self.make_token(QUESTION_MARK)
                }
            }
            '.' => {
                if self.get_current().is_digit(10) {
                    self.make_numeric_token()
                } else if self.matches('.') {
                    self.make_token(RANGE_OPERATOR)
                } else {
                    self.make_token(DOT_SEPARATOR)
                }
            }
            '-' => {
                if self.matches('=') {
                    self.make_token(MINUS_EQUALS)
                } else if self.matches('>') {
                    self.make_token(THIN_ARROW)
                } else {
                    self.make_token(MINUS)
                }
            }
            '+' => {
                if self.matches('=') {
                    self.make_token(PLUS_EQUALS)
                } else {
                    self.make_token(PLUS)
                }
            }
            '*' => {
                if self.matches('=') {
                    self.make_token(STAR_EQUALS)
                } else if self.matches('*') {
                    if self.matches('=') {
                        self.make_token(EXPO_EQUALS)
                    } else {
                        self.make_token(EXPO)
                    }
                } else {
                    self.make_token(STAR)
                }
            }
            '<' => {
                if self.matches('=') {
                    self.make_token(LESS_THAN_EQ)
                } else if self.matches('<') {
                    if self.matches('=') {
                        self.make_token(BITWISE_LEFT_SHIFT_EQUALS)
                    } else {
                        self.make_token(BITWISE_LEFT_SHIFT)
                    }
                } else {
                    self.make_token(LESS_THAN)
                }
            }
            '>' => {
                if self.matches('=') {
                    self.make_token(GREATER_THAN_EQ)
                } else if self.matches('>') {
                    if self.matches('=') {
                        self.make_token(BITWISE_RIGHT_SHIFT_EQUALS)
                    } else {
                        self.make_token(BITWISE_RIGHT_SHIFT)
                    }
                } else {
                    self.make_token(GREATER_THAN)
                }
            }

            // Everything else is an error token
            _ => self.make_error_token("Unexpected character"),
        };
    }
}
