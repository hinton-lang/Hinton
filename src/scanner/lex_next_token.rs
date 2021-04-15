use std::rc::Rc;

use super::{tokens::Token, tokens::TokenType::*, Lexer};

impl<'a> Lexer<'a> {
    /// Scans the next token in the source file.
    pub fn next_token(&mut self) -> Rc<Token<'a>> {
        self.skip_whitespace();

        // If we are at the end, return the EOF token.
        if self.is_at_end() {
            return self.make_token(EOF);
        }

        // Reset the start of the token
        self.token_start = self.current;
        let c = self.advance();

        // Generates an identifier/keyword if the current character is alphanumeric
        if c.unwrap().is_alphabetic() {
            return self.make_identifier_token();
        }

        // Generates an numeric literal if the current character is alphanumeric
        if c.unwrap().is_digit(10) {
            return self.make_numeric_token();
        }

        // Generate symbol-like token tokens
        return match c {
            Some('"') | Some('\'') => self.make_string_token(),
            Some('(') => self.make_token(LEFT_PARENTHESIS),
            Some(')') => self.make_token(RIGHT_PARENTHESIS),
            Some('{') => self.make_token(LEFT_CURLY_BRACES),
            Some('}') => self.make_token(RIGHT_CURLY_BRACES),
            Some('[') => self.make_token(LEFT_SQUARE_BRACKET),
            Some(']') => self.make_token(RIGHT_SQUARE_BRACKET),
            Some(';') => self.make_token(SEMICOLON_SEPARATOR),
            Some(':') => self.make_token(COLON_SEPARATOR),
            Some(',') => self.make_token(COMMA_SEPARATOR),
            Some('^') => self.make_token(BITWISE_XOR),
            Some('~') => self.make_token(BITWISE_NOT),
            Some('/') => {
                let tok = if self.matches(Some('=')) { SLASH_EQUALS } else { SLASH };
                return self.make_token(tok);
            }
            Some('%') => {
                let tok = if self.matches(Some('=')) { MOD_EQUALS } else { MODULUS };
                return self.make_token(tok);
            }
            Some('!') => {
                let tok = if self.matches(Some('=')) { LOGICAL_NOT_EQ } else { LOGICAL_NOT };
                return self.make_token(tok);
            }
            Some('=') => {
                let tok = if self.matches(Some('=')) { LOGICAL_EQ } else { EQUALS_SIGN };
                return self.make_token(tok);
            }
            Some('&') => {
                let tok = if self.matches(Some('&')) { LOGICAL_AND } else { BITWISE_AND };
                return self.make_token(tok);
            }
            Some('|') => {
                let tok = if self.matches(Some('|')) { LOGICAL_OR } else { BITWISE_OR };
                return self.make_token(tok);
            }
            Some('?') => {
                let tok = if self.matches(Some('?')) { NULLISH_COALESCING } else { QUESTION_MARK };
                return self.make_token(tok);
            }
            Some('.') => {
                if self.get_current().unwrap().is_digit(10) {
                    return self.make_numeric_token();
                } else if self.matches(Some('.')) {
                    return self.make_token(RANGE_OPERATOR);
                } else {
                    return self.make_token(DOT_SEPARATOR);
                }
            }
            Some('-') => {
                if self.matches(Some('=')) {
                    return self.make_token(MINUS_EQUALS);
                } else if self.matches(Some('-')) {
                    return self.make_token(DECREMENT);
                } else if self.matches(Some('>')) {
                    return self.make_token(THIN_ARROW);
                } else {
                    return self.make_token(MINUS);
                }
            }
            Some('+') => {
                if self.matches(Some('=')) {
                    return self.make_token(PLUS_EQUALS);
                } else if self.matches(Some('+')) {
                    return self.make_token(INCREMENT);
                } else {
                    return self.make_token(PLUS);
                }
            }
            Some('*') => {
                if self.matches(Some('=')) {
                    return self.make_token(STAR_EQUALS);
                } else if self.matches(Some('*')) && !self.matches(Some('=')) {
                    return self.make_token(EXPO);
                } else if self.matches(Some('*')) && self.matches(Some('=')) {
                    return self.make_token(EXPO_EQUALS);
                } else {
                    return self.make_token(STAR);
                }
            }
            Some('<') => {
                if self.matches(Some('=')) {
                    return self.make_token(LESS_THAN_EQ);
                } else if self.matches(Some('<')) {
                    return self.make_token(BITWISE_LEFT_SHIFT);
                } else {
                    return self.make_token(LESS_THAN);
                }
            }
            Some('>') => {
                if self.matches(Some('=')) {
                    return self.make_token(GREATER_THAN_EQ);
                } else if self.matches(Some('>')) {
                    return self.make_token(BITWISE_RIGHT_SHIFT);
                } else {
                    return self.make_token(GREATER_THAN);
                }
            }

            // Everything else is an error token
            _ => self.make_error_token("Unexpected character"),
        };
    }
}
