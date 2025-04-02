use core::tokens::{ErrorTokenKind, Token, TokenKind::*};

use crate::{Lexer, LexerMode};

impl<'a> Lexer<'a> {
  /// Makes a string literal.
  pub fn make_string_token(&mut self) -> Token {
    // Store the kind of quote that started the string.
    let quote_kind = self.get_previous();

    // Make a "start interpolation" token at string's opening
    // quote just in case this string is actually interpolated.
    // This will not emit the token, only store it in this variable.
    let start_interpol_tok = self.make_token(START_INTERPOL_STR);
    let mut is_interpol_str = false;

    // Do not include the opening quote in the str literal
    self.token_start += 1;

    loop {
      if self.is_at_end() {
        return self.make_error_token(ErrorTokenKind::UnterminatedStr, false);
      }

      // If the string turns out to be interpolated, then finish the string literal
      // using the interpolation mode. Otherwise, this loop will continue running
      // and a single str literal will be returned at the end of this function.
      if self.get_current() == '$' && self.get_next() == '{' && self.get_previous() != '\\' {
        // Emit the start interpol literal token
        if !is_interpol_str {
          self.tokens.push(start_interpol_tok);
        }

        is_interpol_str = true;

        // Emit any string literals before the start of the string interpolation.
        if self.current - self.token_start > 0 {
          self.tokens.push(self.make_token(STR_LIT));
        }

        // Emit the START_INTERPOL_EXPR token with the '${' characters.
        self.token_start = self.current;
        self.advance();
        self.advance();
        self.tokens.push(self.make_token(START_INTERPOL_EXPR));
        self.token_start = self.current;

        // Lex the interpolation content
        self.find_tokens(LexerMode::StrInterpol);

        if self.get_current() == quote_kind {
          self.advance();
          return self.make_token(END_INTERPOL_STR);
        }
      }

      // If we reach an unescaped quote, break the loop  without consuming the closing quote.
      if self.get_current() == quote_kind && self.get_previous() != '\\' {
        break;
      }

      // Advance through the string, taking new lines into account
      if self.advance() == '\n' {
        self.line_num += 1;
      }
    }

    let final_str_part = self.make_token(STR_LIT);
    self.advance();

    if is_interpol_str {
      self.tokens.push(final_str_part);
      self.token_start = self.current - 1;
      self.make_token(END_INTERPOL_STR)
    } else {
      final_str_part
    }
  }
}
