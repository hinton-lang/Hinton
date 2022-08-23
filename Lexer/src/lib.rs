use crate::tokens::TokenLoc;
use core::tokens;
use core::tokens::{ErrorTokenKind, Token, TokenKind};

mod find_tokens;
mod lex_numbers;
mod lex_strings;

#[macro_export]
macro_rules! char_is_ident_start {
    ($ch:expr) => {{$ch.is_alphabetic() || $ch == '_' || $ch == '$'}};
}

#[derive(Debug)]
pub enum LexerMode {
  Default,
  StrInterpol,
  InterpolBlockExpr,
}

/// Struct that represents the scanner.
pub struct Lexer<'a> {
  /// A flat list of characters from the source file.
  source: &'a [char],
  /// The list of tokens found in the source file.
  tokens: Vec<Token>,
  /// The index of the current character.
  current: usize,
  /// The current line index.
  line_num: usize,
  /// The position in the flat source vector of the first
  /// character for the current line.
  line_start: usize,
  /// The position of the first character for the current
  /// token in the flat source vector.
  token_start: usize,
}

impl<'a> Lexer<'a> {
  /// An initialized instance of the Lexer.
  /// # Parameters
  /// - `src` - the source file
  pub fn lex(source: &[char]) -> Vec<Token> {
    // Instantiate a new Lexer
    let mut the_lexer = Lexer {
      source,
      tokens: vec![Token {
        kind: TokenKind::THIS_FILE,
        loc: TokenLoc {
          line_num: 0,
          line_start: 0,
          span: (0, 0),
        },
      }],
      current: 0,
      line_num: 1,
      line_start: 0,
      token_start: 0,
    };

    // Find tokens in the source
    the_lexer.find_tokens(LexerMode::Default);

    // Only return the tokens
    the_lexer.tokens
  }

  /// Gets the previously consumed character.
  ///
  /// # Returns
  /// - `char`: The previous character.
  pub fn get_previous(&self) -> char {
    self.source[self.current - 1]
  }

  /// Gets the current character without consuming it.
  ///
  /// # Returns
  /// - `char`: The current character.
  pub fn get_current(&self) -> char {
    if self.is_at_end() {
      '\0'
    } else {
      self.source[self.current]
    }
  }

  /// Returns the next character without consuming it.
  ///
  /// # Returns
  /// - `char`: The next character.
  pub fn get_next(&self) -> char {
    if self.is_at_end() {
      return '\0';
    }

    self.source[self.current + 1]
  }

  /// Checks if the scanner is at the end of the source.
  ///
  /// # Returns
  /// - `bool`: True if the scanner is at the end of the source, false otherwise.
  pub fn is_at_end(&self) -> bool {
    self.current >= self.source.len()
  }

  /// Matches the current character against a provided character.
  ///
  /// # Returns
  /// - `bool`: True if the current character matched the provided character, false otherwise.
  pub fn matches(&mut self, expected: char) -> bool {
    if self.is_at_end() || self.get_current() != expected {
      return false;
    }
    self.current += 1;
    true
  }

  /// Matches the current and next characters against a provided character sequence.
  ///
  /// # Returns
  /// - `bool`: True if the current and next characters match the provided character sequence,
  /// in that order; false otherwise.
  pub fn matches_two(&mut self, c1: char, c2: char) -> bool {
    if self.source.len() < 3
      || self.current >= self.source.len() - 2
      || self.get_current() != c1
      || self.get_next() != c2
    {
      return false;
    }
    self.current += 2;
    true
  }

  /// Advances to the next char and returns consumed char.
  ///
  /// # Returns
  /// - `char`: The consumed character.
  pub fn advance(&mut self) -> char {
    self.current += 1;
    self.get_previous()
  }

  /// Skips whitespace-like characters from the source code.
  pub fn skip_whitespace(&mut self) {
    loop {
      if self.is_at_end() {
        break;
      }

      match self.get_current() {
        ' ' | '\r' | '\t' => self.current += 1,
        '/' if self.matches('/') => self.skip_single_line_comments(),
        '/' if self.matches('*') => self.skip_block_comments(),
        '\n' => {
          self.line_num += 1;
          self.line_start = self.current + 1;
          self.advance();
        }
        _ => break,
      }
    }
  }

  /// Skips single-line comments from the source code.
  fn skip_single_line_comments(&mut self) {
    while !self.is_at_end() && self.get_current() != '\n' {
      self.advance();
    }

    // Reposition the start of the token to
    // be after the comment has ended
    self.token_start = self.current;
  }

  /// Skips block-comments from the source code
  fn skip_block_comments(&mut self) {
    self.advance();
    self.advance();

    while !self.is_at_end() {
      // Recursively skip nested block comments
      if self.get_current() == '/' && self.get_next() == '*' {
        self.skip_block_comments();

        // break out of the loop if we reached the end of the program
        // before matching the end of this nested block-comment.
        if self.is_at_end() {
          break;
        }
      }

      // Break if we are at the end of the comment.
      if self.get_current() == '*' && self.get_next() == '/' {
        self.advance();
        self.advance();
        break;
      }

      // Take into account new lines inside block comments
      if self.get_current() == '\n' {
        self.line_num += 1;
      }

      // Skip everything inside the comment
      self.advance();
    }

    // Reposition the start of the token to
    // be after the comment has ended
    self.token_start = self.current;
  }

  /// Generates an identifier token with the current state of the scanner.
  pub fn make_identifier_token(&mut self) -> Token {
    while !self.is_at_end() {
      let c = self.get_current();

      if c.is_alphabetic() || c.is_ascii_digit() || c == '_' {
        self.advance();
      } else {
        break;
      }
    }

    let id: String = self.source[(self.token_start)..(self.current)].iter().collect();
    let tok_kind = tokens::make_identifier_kind(id.as_str());

    self.make_token(tok_kind)
  }

  /// Generates a token with the current state of the scanner.
  pub fn make_token(&self, kind: TokenKind) -> Token {
    let loc = TokenLoc {
      line_num: self.line_num,
      line_start: self.line_start,
      span: (self.token_start, self.current),
    };

    Token { loc, kind }
  }

  fn make_eof_token(&mut self) {
    let loc = TokenLoc {
      line_num: self.line_num,
      line_start: self.line_start,
      span: (self.token_start + 1, self.current),
    };

    self.tokens.push(Token { loc, kind: TokenKind::EOF });
  }

  /// Generates an error at the current character with the provided message as its lexeme.
  ///
  /// # Parameters
  /// - `message`: A message for the error token. This will be used as the token's lexeme.
  ///
  /// # Returns
  /// - `Token`: The generated error token.
  pub fn make_error_token(&mut self, err: ErrorTokenKind, advance: bool) -> Token {
    let loc = TokenLoc {
      line_num: self.line_num,
      line_start: self.line_start,
      span: (self.token_start, self.current),
    };

    let tok = Token {
      loc,
      kind: TokenKind::ERROR(err),
    };

    if advance {
      self.advance();
    }

    tok
  }

  /// Generates an error at the previous character with the provided message as its lexeme.
  ///
  /// # Returns
  /// - `Token`: The generated error token.
  pub fn make_error_token_at_prev(&self, err: ErrorTokenKind) -> Token {
    let loc = TokenLoc {
      line_num: self.line_num,
      line_start: self.line_start,
      span: (self.token_start, self.current),
    };

    Token {
      loc,
      kind: TokenKind::ERROR(err),
    }
  }
}
