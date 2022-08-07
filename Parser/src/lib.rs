use core::ast::*;
use core::errors::ErrorReport;
use core::tokens::TokenKind::*;
use core::tokens::*;

mod parse_classes;
mod parse_collections;
mod parse_declarations;
mod parse_expressions;
mod parse_statements;

/// If the current token matches any of the provided tokens,
/// consume it and return true, otherwise simple return false.
#[macro_export]
macro_rules! match_tok {
   ($s:expr, $id:ident $(| $ids:ident)*) => {
      $s.matches(&$id) $(|| $s.matches(&$ids))*
   }
}

/// Return true if the current token matches any of
/// the provided tokens, otherwise simple return false.
#[macro_export]
macro_rules! check_tok {
   ($s:expr, $id:ident $(| $ids:ident)*) => {
      $s.check(&$id) $(|| $s.check(&$ids))*
   }
}

/// Get a reference to tje current token's kind.
#[macro_export]
macro_rules! curr_tk {
  ($s:ident) => {
    &$s.tokens[$s.current_pos].kind
  };
}

/// Consume an identifier token and emit an identifier node.
#[macro_export]
macro_rules! consume_id {
  ($s:ident, $err:expr) => {{
    let tok_idx = $s.consume(&IDENTIFIER, $err)?;
    $s.emit(Identifier(tok_idx.into()))
  }};
}

/// Consume a list of comma-separated identifiers, emit their
/// respective identifier nodes, and return a list of with
/// the ASTNodeIdx's of each emitted identifier node.
#[macro_export]
macro_rules! consume_id_list {
  ($s:ident,$err_msg:expr) => {{
    let mut ids = vec![consume_id![$s, $err_msg]?];
    while match_tok![$s, COMMA] {
      ids.push(consume_id![$s, $err_msg]?);
    }
    ids
  }};
}

/// Guard the parser against error tokens present in the tokens list.
#[macro_export]
macro_rules! guard_error_token {
  ($s:ident) => {
    if let ERROR(e) = curr_tk![$s] {
      return Err($s.error_at_prev(e.to_str()));
    }
  };
}

/// The result of parsing a node or part of a node.
pub type NodeResult<T> = Result<T, ErrorReport>;

/// Represents Hinton's Parser, which converts source text into
/// an Abstract Syntax Tree representation of the program.
pub struct Parser<'a> {
  /// The Lexer used in this Parser.
  tokens: &'a TokenList<'a>,
  /// The position of the Parser in the list of tokens.
  current_pos: usize,
  /// The program's AST as an ArenaTree
  pub ast: ASTArena,
  /// Whether the Parser is in error-recovery mode or not.
  is_in_panic: bool,
  /// A list of reported errors generated while parsing.
  errors: Vec<ErrorReport>,
}

impl<'a> Parser<'a> {
  /// Composes an ASTArena from a TokenList
  ///
  /// # Arguments
  ///
  /// * `tokens`: The TokenList containing the lexed tokens.
  ///
  /// # Returns:
  /// ```Parser```
  pub fn parse(tokens: &'a TokenList) -> Parser<'a> {
    let mut parser = Parser {
      tokens,
      current_pos: 0,
      is_in_panic: false,
      errors: vec![],
      ast: ASTArena::default(),
    };

    // Parse the entire list of tokens into an AST
    parser.parse_module();

    // Return the parse
    parser
  }

  /// Gets a reference to the previous token.
  /// NOTE: Boundaries not checked.
  fn prev_tok(&self) -> &Token {
    &self.tokens[self.current_pos - 1]
  }

  /// Gets a reference to the next token.
  /// NOTE: Boundaries not checked.
  fn curr_tok(&self) -> &Token {
    &self.tokens[self.current_pos]
  }

  /// Gets the previous token's kind.
  /// NOTE: Boundaries not checked.
  fn get_prev_tk(&self) -> &TokenKind {
    &self.prev_tok().kind
  }

  /// Gets the current token's kind.
  /// NOTE: Boundaries not checked.
  fn get_curr_tk(&self) -> &TokenKind {
    &self.curr_tok().kind
  }

  /// Gets the next token's kind.
  /// NOTE: Boundaries not checked.
  fn get_next_tk(&self) -> &TokenKind {
    &self.tokens[self.current_pos + 1].kind
  }

  /// Checks that the current token matches the TokenKind provided.
  ///
  /// # Parameters
  /// - `tk` The token Kind we expect to match with the current token.
  ///
  /// # Results
  /// - `bool`: True if the current token matches the given token type false otherwise.
  fn check(&mut self, tk: &TokenKind) -> bool {
    let tt = self.get_curr_tk();
    tt.type_match(tk)
  }

  /// Checks that the current token matches the TokenKind provided.
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

  /// Advances the Parser to the next token.
  fn advance(&mut self) -> bool {
    self.current_pos += 1;
    true
  }

  /// Consumes the current token only if it is of a given type.
  /// If the token is not of the expected kind, returns an ErrorReport.
  ///
  /// # Parameters
  /// - `tk`: The kind of token we expect to consume.
  /// - `message`: The error message used in the ErrorReport if the
  /// current token is not of the given kind.
  fn consume(&mut self, tk: &TokenKind, message: &str) -> NodeResult<TokenIdx> {
    if self.check(tk) {
      self.advance();
      return Ok((self.current_pos - 1).into());
    }

    if let SEMICOLON = tk {
      Err(self.error_at_prev(message))
    } else {
      Err(self.error_at_current(message))
    }
  }

  /// Emit an ASTNodeKind to the parser's ASTArena.
  ///
  /// # Arguments
  ///
  /// * `node`: The AST node to be added to the arena.
  pub fn emit(&mut self, node: ASTNodeKind) -> NodeResult<ASTNodeIdx> {
    Ok(self.ast.push(node))
  }

  /// Emits a syntax error from the current token.
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
  fn error_at_prev(&mut self, message: &str) -> ErrorReport {
    self.error_at_tok((self.current_pos - 1).into(), message)
  }

  /// Emits a compiler error from the given token.
  ///
  /// # Parameters
  /// - `tok`: The token that caused the error.
  /// - `message`: The error message to display.
  fn error_at_tok(&mut self, tok_idx: TokenIdx, message: &str) -> ErrorReport {
    let tok = &self.tokens[tok_idx.0];

    // Construct the error message.
    let msg = format!(
      "\x1b[31;1mSyntaxError\x1b[0m\x1b[1m at [{}:{}]: {}\x1b[0m",
      tok.line_num, tok.span.0, message
    );

    ErrorReport {
      line: tok.line_num,
      column: tok.span.0,
      lexeme_len: tok.span.1 - tok.span.0,
      message: msg,
    }
  }

  /// Gets the list of generated errors.
  pub fn get_errors_list(&self) -> &[ErrorReport] {
    &self.errors
  }

  /// Synchronizes the parser when it has found an error.
  /// This method helps minimize the number of cascading errors the compiler emits
  /// when it finds a parsing error. After an error, it skips tokens until it find
  /// a synchronization point, like the keyword for a statement.
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
