use crate::errors::ErrorReport;
use crate::lexer::tokens::TokenKind::*;
use crate::parser::ast::ASTNodeKind::*;
use crate::parser::ast::*;
use crate::parser::Parser;
use crate::{curr_tk, match_tok};

impl<'a> Parser<'a> {
  /// Parses a variable or constant declaration statement.
  ///
  /// ```bnf
  /// VAR_CONST_DECL ::= ("let" | "const") (IDENTIFIER | DESTRUCT_PATTERN) "=" EXPRESSION ";"
  /// ```
  pub(super) fn parse_var_or_const_decl(&mut self, is_const: bool) -> Result<ASTNodeIdx, ErrorReport> {
    let decl_name = if is_const { "const" } else { "let" };

    let id = if match_tok![self, L_PAREN] {
      self.parse_destructing_pattern(&format!("'{}' declaration", decl_name))?
    } else {
      let id = self.consume(
        &IDENTIFIER,
        &format!("Expected identifier for '{}' declaration.", decl_name),
      )?;
      self.emit(Identifier(id))?
    };

    self.consume(&EQUALS, &format!("Expected '=' for '{}' declaration.", decl_name))?;
    let val = self.parse_expr()?;
    self.consume(&SEMICOLON, &format!("Expected ';' after '{}' declaration.", decl_name))?;

    self.emit(VarConstDecl(ASTVarConsDeclNode { is_const, id, val }))
  }

  /// Parses a destructing pattern to be used in a variable or constant declaration, or in a for-loop statement.
  ///
  /// ```bnf
  /// IDENTIFIER_LIST  ::= IDENTIFIER ("," IDENTIFIER)*
  /// DESTRUCT_PATTERN ::= "(" IDENTIFIER_LIST ")" // no wildcard
  ///                  |   "(" IDENTIFIER_LIST "," "..." IDENTIFIER? ")" // tail wildcard
  ///                  |   "(" IDENTIFIER_LIST "," "..." IDENTIFIER? "," IDENTIFIER_LIST ")" // middle wildcard
  ///                  |   "(" "..." IDENTIFIER? "," IDENTIFIER_LIST ")" // head wildcard
  /// ```
  pub(super) fn parse_destructing_pattern(&mut self, msg: &str) -> Result<ASTNodeIdx, ErrorReport> {
    let mut patterns = vec![];
    let mut has_rest = false;

    loop {
      let pattern = match curr_tk![self] {
        SPREAD if has_rest => {
          return Err(self.error_at_current("Can only have one wildcard expression in destructing pattern."));
        }
        SPREAD if self.advance() => {
          has_rest = true;

          let node = if match_tok![self, IDENTIFIER] {
            Some(self.current_pos - 1)
          } else {
            None
          };

          self.emit(DestructingWildCard(node))?
        }
        _ => {
          let node = self.consume(
            &IDENTIFIER,
            &format!("Expected identifier for destructing pattern in {}.", msg),
          )?;
          self.emit(Identifier(node))?
        }
      };

      patterns.push(pattern);

      if !match_tok![self, COMMA] {
        break;
      }
    }

    if has_rest && patterns.len() == 1 {
      return Err(self.error_at_current("Cannot have destructing pattern with only a wildcard expression."));
    }

    self.consume(&R_PAREN, "Expected ')' after destructing pattern.")?;
    self.emit(DestructingPattern(ASTDestructingPatternNode { patterns }))
  }
}
