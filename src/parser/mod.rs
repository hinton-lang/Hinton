use crate::errors::ErrorReport;
use crate::lexer::tokens::TokenKind::*;
use crate::lexer::tokens::*;
use crate::parser::ast::*;

// Submodules
pub mod ast;
pub mod legacy_ast;
mod parse_clctns;
mod parse_decls;
mod parse_exprs;
mod parse_stmts;

#[macro_export]
macro_rules! match_tok {
   ($s:expr, $id:ident $(| $ids:ident)* ) => {
      $s.matches(&$id)
      $(|| $s.matches(&$ids) )*
   }
}

#[macro_export]
macro_rules! check_tok {
   ($s:expr, $id:ident $(| $ids:ident)* ) => {
      $s.check(&$id)
      $(|| $s.check(&$ids) )*
   }
}

#[macro_export]
macro_rules! curr_tk {
  ($s:ident) => {
    &$s.tokens[$s.current_pos].kind
  };
}

#[macro_export]
macro_rules! consume_id {
  ($s:ident, $err:expr) => {{
    let tok_idx = $s.consume(&IDENTIFIER, $err)?;
    $s.emit(Identifier(tok_idx.into()))
  }};
}

#[macro_export]
macro_rules! guard_error_token {
  ($s:ident) => {
    if match_tok![$s, ERROR] {
      let err_msg = $s.prev_tok().lexeme.to_string();
      return Err($s.error_at_previous(&err_msg));
    }
  };
}

/// Represents Hinton's parser, which converts source text into
/// an Abstract Syntax Tree representation of the program.
pub struct Parser<'a> {
  /// The lexer used in this parser.
  tokens: &'a [Token],
  /// The position of the parser in the list of tokens.
  current_pos: usize,
  /// The program's AST as an ArenaTree
  pub(crate) ast: ASTArena,
  /// Whether the parser is in error-recovery mode or not.
  is_in_panic: bool,
  /// A list of reported errors generated while parsing.
  errors: Vec<ErrorReport>,
}

impl<'a> Parser<'a> {
  /// Parses a string of source text into a Hinton AST.
  pub fn parse(tokens: &'a [Token]) -> Parser<'a> {
    let mut parser = Parser {
      tokens,
      current_pos: 0,
      is_in_panic: false,
      errors: vec![],
      // Comes with a root `module` node.
      ast: ASTArena::default(),
    };

    // Parse the entire list of tokens into an AST
    parser.parse_module();

    // Return the parse
    parser
  }

  /// Checks that the current token matches the tokenType provided.
  ///
  /// # Parameters
  /// - `tk` The tokenKind we expect to match with the current token.
  ///
  /// # Results
  /// - `bool`: True if the current token matches the given token type false otherwise.
  fn check(&mut self, tk: &TokenKind) -> bool {
    let tt = self.get_curr_tk();
    tt.type_match(tk)
  }

  /// Checks that the current token matches the tokenType provided.
  /// If the tokens match, the current token gets consumed, and the function returns true.  
  /// Otherwise, if the tokens do not match, the token is not consumed, and the function
  /// returns false.
  ///
  /// # Parameters
  /// - `tk` The tokenType we expect to match with the current token.
  ///
  /// # Returns
  /// `bool`: True if the tokens match, false otherwise.
  fn matches(&mut self, tk: &TokenKind) -> bool {
    if self.check(tk) {
      self.advance()
    } else {
      false
    }
  }

  /// Advances the parser to the next token.
  fn advance(&mut self) -> bool {
    self.current_pos += 1;
    true
  }

  fn prev_tok(&self) -> &Token {
    &self.tokens[self.current_pos - 1]
  }

  fn curr_tok(&self) -> &Token {
    &self.tokens[self.current_pos]
  }

  /// Consumes the current token only if it is of a given type. If the token does not match the
  /// type, emits a compiler error.
  ///
  /// # Parameters
  /// - `tok_type`: The expected type of the token to consume.
  /// - `message`: The error message to be displayed if the current token does not match the
  /// provided type.
  fn consume(&mut self, tok_type: &TokenKind, message: &str) -> Result<TokenIdx, ErrorReport> {
    if self.check(tok_type) {
      self.advance();
      return Ok((self.current_pos - 1).into());
    }

    if let SEMICOLON = tok_type {
      Err(self.error_at_previous(message))
    } else {
      Err(self.error_at_current(message))
    }
  }

  /// Gets the type of the current token.
  ///
  /// # Returns
  /// `TokenKind`: The type of the current token.
  fn get_curr_tk(&self) -> &TokenKind {
    &self.curr_tok().kind
  }

  /// Gets the type of the previous token.
  ///
  /// # Returns
  /// `TokenKind`: The type of the previous token.
  fn get_prev_tk(&self) -> &TokenKind {
    &self.prev_tok().kind
  }

  /// Gets the type of the next token.
  ///
  /// # Returns
  /// `TokenKind`: The type of the next token.
  fn get_next_tk(&self) -> &TokenKind {
    &self.tokens[self.current_pos + 1].kind
  }

  /// Emits a compiler error from the current token.
  ///
  /// # Parameters
  /// - `message`: The error message to display.
  fn error_at_current(&mut self, message: &str) -> ErrorReport {
    self.error_at_tok(self.current_pos.into(), message)
  }

  /// Emits a compiler error from the previous token.
  ///
  /// # Parameters
  /// - `message`: The error message to display.
  fn error_at_previous(&mut self, message: &str) -> ErrorReport {
    self.error_at_tok((self.current_pos - 1).into(), message)
  }

  pub fn emit(&mut self, node: ASTNodeKind) -> Result<ASTNodeIdx, ErrorReport> {
    Ok(self.ast.append(node))
  }

  /// Emits a compiler error from the given token.
  ///
  /// # Parameters
  /// - `tok`: The token that caused the error.
  /// - `message`: The error message to display.
  fn error_at_tok(&mut self, tok_idx: TokenIdx, message: &str) -> ErrorReport {
    let tok: &Token = &self.tokens[tok_idx.0];

    // Construct the error message.
    let msg = format!(
      "\x1b[31;1mSyntaxError\x1b[0m\x1b[1m at [{}:{}]: {}\x1b[0m",
      tok.line_num, tok.column_start, message
    );

    ErrorReport {
      line: tok.line_num,
      column: tok.column_start,
      lexeme_len: tok.column_end - tok.column_start,
      message: msg,
    }
  }

  pub fn get_errors_list(&self) -> &Vec<ErrorReport> {
    &self.errors
  }

  /// Synchronizes the compiler when it has found an error.
  /// This method helps minimize the number of cascading errors the compiler emits
  /// when it finds a parsing error. Once it reaches a synchronization point – like
  /// a keyword for a statement – it stops emitting errors.
  fn synchronize(&mut self) {
    self.is_in_panic = false;

    while !self.get_curr_tk().type_match(&EOF) {
      if let SEMICOLON = self.get_prev_tk() {
        return;
      }

      if matches![
        self.get_curr_tk(),
        CLASS_KW | FUNC_KW | LET_KW | FOR_KW | IF_KW | WHILE_KW | RETURN_KW
      ] {
        return;
      }

      self.advance();
    }
  }
}
