use crate::lexer::find_tokens::LexerMode;
use crate::lexer::tokens::{Token, TokenKind::*};
use crate::lexer::Lexer;

impl Lexer {
  /// Makes a string literal.
  pub fn make_string_token(&mut self) -> Token {
    // The opener single or double quote.
    let quote = self.get_previous();
    let str_start = self.current;
    let mut is_interpol_str = false;

    // Skip the opening quote in literal
    self.token_start += 1;

    loop {
      if self.is_at_end() {
        return self.make_error_token("Unterminated string.", false);
      }

      // If the string turns out to be interpolated, then finish the string literal
      // using the interpolation mode. Otherwise, this loop will continue running
      // and a single str literal will be returned at the end of this function.
      if self.get_current() == '$' && self.get_next() == '{' && self.get_previous() != '\\' {
        is_interpol_str = true;

        self.lex_interpolated_string(str_start);

        if self.get_current() == quote {
          self.advance();
          return self.make_token(END_INTERPOL_STR);
        }
      }

      // If we reach an unescaped quote, break the loop
      // without consuming the closing quote.
      if self.get_current() == quote && self.get_previous() != '\\' {
        break;
      }

      // Advance through the string, taking new lines into account
      if self.advance() == '\n' {
        self.line_num += 1;
        continue;
      }
    }

    let str_token = self.make_token(STR_LIT);
    self.advance();

    if is_interpol_str {
      self.tokens.push(self.make_token(END_INTERPOL_STR));
    }

    str_token
  }

  fn lex_interpolated_string(&mut self, str_start: usize) {
    let current_pos = self.current;

    self.token_start -= 1;
    self.current = str_start;
    self.tokens.push(self.make_token(START_INTERPOL_STR));
    self.token_start += 1;
    self.current = current_pos;

    // Mark any content before the `${` as a str literal
    if self.current - str_start > 0 {
      self.tokens.push(self.make_token(STR_LIT));
    }

    self.token_start = self.current;
    self.advance();
    self.advance();
    self.tokens.push(self.make_token(START_INTERPOL_EXPR));
    self.token_start = self.current;
    self.find_tokens(LexerMode::StrInterpol);
    self.tokens.push(self.make_token(END_INTERPOL_EXPR));

    // Advance the right-curly brace from the interpolation
    self.token_start += 1;
  }
}
