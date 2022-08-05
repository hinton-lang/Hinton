use crate::lexer::tokens::TokenKind::*;
use crate::lexer::tokens::{ErrorTokenKind, Token};
use crate::lexer::Lexer;

#[derive(Debug)]
pub enum LexerMode {
  Default,
  StrInterpol,
  InterpolBlockExpr,
}

impl<'a> Lexer<'a> {
  /// Scans the next token in the source file.
  pub fn find_tokens(&mut self, mode: LexerMode) {
    loop {
      // Skips all empty spaces in the source code
      self.skip_whitespace();

      // If we are at the end, return the EOF token.
      if self.is_at_end() {
        self.make_eof_token();
        return;
      }

      // Reset the start of the token
      self.token_start = self.current;

      // Get the next token
      let next_token = self.next();

      if let &R_CURLY = &next_token.kind {
        match mode {
          // In this case, the right-curly-bracket becomes the
          // closing token for the string interpolation. This is
          // handled by the `lex_interpolated_string` function.
          LexerMode::StrInterpol => return,
          // In this case, we close the block's loop and return the
          // lexer to its previous mode. Notice that we can only enter
          // `InterpolBlockExpr` mode after entering `StrInterpol` mode.
          LexerMode::InterpolBlockExpr => {
            self.tokens.push(next_token);
            return;
          }
          _ => {}
        }
      }

      if matches![&next_token.kind, L_CURLY] && matches![mode, LexerMode::StrInterpol] {
        self.tokens.push(next_token);
        self.find_tokens(LexerMode::InterpolBlockExpr);
      } else {
        self.tokens.push(next_token);
      }
    }
  }

  fn next(&mut self) -> Token {
    match self.advance() {
      '(' => self.make_token(L_PAREN),
      ')' => self.make_token(R_PAREN),
      '[' => self.make_token(L_BRACKET),
      ']' => self.make_token(R_BRACKET),
      '{' => self.make_token(L_CURLY),
      '}' => self.make_token(R_CURLY),
      ';' => self.make_token(SEMICOLON),
      ',' => self.make_token(COMMA),
      '~' => self.make_token(BIT_NOT),
      '#' => self.make_token(HASHTAG),

      // "+", "+="
      '+' if self.matches('=') => self.make_token(PLUS_EQ),
      '+' => self.make_token(PLUS),

      // "-", "-=", "->"
      '-' if self.matches('=') => self.make_token(MINUS_EQ),
      '-' if self.matches('>') => self.make_token(THIN_ARROW),
      '-' => self.make_token(DASH),

      // "*", "*=", "**", "**="
      '*' if self.matches_two('*', '=') => self.make_token(POW_EQUALS),
      '*' if self.matches('=') => self.make_token(STAR_EQ),
      '*' if self.matches('*') => self.make_token(POW),
      '*' => self.make_token(STAR),

      // "/", "/="
      '/' if self.matches('=') => self.make_token(SLASH_EQ),
      '/' => self.make_token(SLASH),

      // "%", "%="
      '%' if self.matches('=') => self.make_token(MOD_EQ),
      '%' => self.make_token(PERCENT),

      // "@", "@="
      '@' if self.matches('=') => self.make_token(AT_EQ),
      '@' => self.make_token(AT),

      // "!", "!="
      '!' if self.matches('=') => self.make_token(LOGIC_NOT_EQ),
      '!' => self.make_token(BANG),

      // "=", "==", "=>"
      '=' if self.matches('=') => self.make_token(LOGIC_EQ),
      '=' if self.matches('>') => self.make_token(THICK_ARROW),
      '=' => self.make_token(EQUALS),

      // ":", ":="
      ':' if self.matches('=') => self.make_token(COLON_EQUALS),
      ':' => self.make_token(COLON),

      // "^", "^="
      '^' if self.matches('=') => self.make_token(BIT_XOR_EQ),
      '^' => self.make_token(BIT_XOR),

      // "&&", "&&=", "&", "&="
      '&' if self.matches_two('&', '=') => self.make_token(LOGIC_AND_EQ),
      '&' if self.matches('&') => self.make_token(DOUBLE_AMPERSAND),
      '&' if self.matches('=') => self.make_token(BIT_AND_EQ),
      '&' => self.make_token(AMPERSAND),

      // "|", "||", "||=", "|=", "|>"
      '|' if self.matches_two('|', '=') => self.make_token(LOGIC_OR_EQ),
      '|' if self.matches('|') => self.make_token(DOUBLE_VERT_BAR),
      '|' if self.matches('=') => self.make_token(BIT_OR_EQ),
      '|' if self.matches('>') => self.make_token(PIPE),
      '|' => self.make_token(VERT_BAR),

      // "?", "??", "??=", "?."
      '?' if self.matches_two('?', '=') => self.make_token(NONISH_EQ),
      '?' if self.matches('?') => self.make_token(NONISH),
      '?' if self.matches('.') => self.make_token(SAFE_ACCESS),
      '?' => self.make_token(QUESTION),

      // ".", "..", "..=", "...", <leading floating-point number>
      '.' if self.get_current().is_ascii_digit() => self.make_numeric_token(),
      '.' if self.matches_two('.', '.') => self.make_token(TRIPLE_DOT),
      '.' if self.matches_two('.', '=') => self.make_token(RANGE_EQ),
      '.' if self.matches('.') => self.make_token(DOUBLE_DOT),
      '.' => self.make_token(DOT),

      // "<", "<=", "<<", "<<="
      '<' if self.matches_two('<', '=') => self.make_token(BIT_L_SHIFT_EQ),
      '<' if self.matches('=') => self.make_token(LESS_THAN_EQ),
      '<' if self.matches('<') => self.make_token(BIT_L_SHIFT),
      '<' => self.make_token(LESS_THAN),

      // ">", ">=", ">>", ">>="
      '>' if self.matches_two('>', '=') => self.make_token(BIT_R_SHIFT_EQ),
      '>' if self.matches('=') => self.make_token(GREATER_THAN_EQ),
      '>' if self.matches('>') => self.make_token(BIT_R_SHIFT),
      '>' => self.make_token(GREATER_THAN),

      '"' | '\'' => self.make_string_token(),

      // Generates an identifier/keyword if the current character is alphanumeric
      ch if ch.is_alphabetic() || ch == '_' || ch == '$' => self.make_identifier_token(),

      // Generates a numeric literal if the current character is a digit
      ch if ch.is_ascii_digit() => self.make_numeric_token(),

      // Everything else is an error token
      _ => self.make_error_token(ErrorTokenKind::InvalidChar, false),
    }
  }
}
