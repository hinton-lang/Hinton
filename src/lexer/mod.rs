use crate::lexer::find_tokens::LexerMode;
use crate::lexer::tokens::{Token, TokenKind};
use std::cmp::{max, min};

// Submodules
pub mod find_tokens;
pub mod lex_numbers;
mod lex_strings;
pub mod tokens;

/// Struct that represents the scanner.
pub struct Lexer {
  /// A flat list of characters from the source file.
  source: Vec<char>,
  /// The list of tokens found in the source file.
  pub tokens: Vec<Token>,
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

impl Lexer {
  /// An initialized instance of the lexer.
  /// # Parameters
  /// - `src` - the source file
  ///
  /// ## Example
  /// ```
  /// let mut l = Lexer::lex("let x = 22;");
  /// ```
  pub fn lex(src: &str) -> Lexer {
    let chars: Vec<char> = src.chars().collect();

    // Instantiate a new lexer
    let mut the_lexer = Self {
      source: chars,
      tokens: vec![],
      current: 0,
      line_num: 1,
      line_start: 0,
      token_start: 0,
    };

    the_lexer.find_tokens(LexerMode::Default);
    the_lexer
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
    self.source[self.current]
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
    let current = self.get_current();
    self.current += 1;
    current
  }

  /// Skips whitespace-like characters from the source code.
  pub fn skip_whitespace(&mut self) {
    loop {
      if self.is_at_end() {
        break;
      }

      let c = self.get_current();

      if c == ' ' || c == '\r' || c == '\t' {
        self.advance();
      } else if c == '\n' {
        self.line_num += 1;
        self.line_start = self.current + 1;
        self.advance();
      } else if c == '/' && self.get_next() == '/' {
        self.skip_single_line_comments();
      } else if c == '/' && self.get_next() == '*' {
        self.skip_block_comments();
      } else {
        break;
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
  pub fn make_token(&self, token_kind: TokenKind) -> Token {
    let col_start = match token_kind {
      TokenKind::EOF => max(self.line_start, self.current) - min(self.line_start, self.current),
      _ => self.token_start - self.line_start,
    };

    let lexeme = match token_kind {
      TokenKind::EOF => String::from("\0"),
      _ => self.source[(self.token_start)..(self.current)].iter().collect(),
    };

    Token {
      line_num: self.line_num,
      column_start: col_start,
      column_end: self.current,
      kind: token_kind,
      lexeme,
    }
  }

  /// Generates an error at the current character with the provided message as its lexeme.
  ///
  /// # Parameters
  /// - `message`: A message for the error token. This will be used as the token's lexeme.
  ///
  /// # Returns
  /// - `Token`: The generated error token.
  pub fn make_error_token(&mut self, message: &str) -> Token {
    let tok = Token {
      line_num: self.line_num,
      column_start: self.current - self.line_start,
      column_end: self.current - self.line_start + 1,
      kind: TokenKind::ERROR,
      lexeme: String::from(message),
    };
    self.advance();
    tok
  }

  /// Generates an error at the previous character with the provided message as its lexeme.
  ///
  /// # Parameters
  /// - `message`: A message for the error token. This will be used as the token's lexeme.
  ///
  /// # Returns
  /// - `Token`: The generated error token.
  pub fn make_error_token_at_prev(&self, message: &str) -> Token {
    Token {
      line_num: self.line_num,
      column_start: self.current - self.line_start - 1,
      column_end: self.current - self.line_start,
      kind: TokenKind::ERROR,
      lexeme: String::from(message),
    }
  }

  /// Generates an error at the next character with the provided message as its lexeme.
  ///
  /// # Parameters
  /// - `message`: A message for the error token. This will be used as the token's lexeme.
  ///
  /// # Returns
  /// - `Token`: The generated error token.
  #[allow(dead_code)]
  pub fn make_error_token_at_next(&self, message: &str) -> Token {
    Token {
      line_num: self.line_num,
      column_start: self.current - self.line_start - 1,
      column_end: self.current - self.line_start,
      kind: TokenKind::ERROR,
      lexeme: String::from(message),
    }
  }
}
