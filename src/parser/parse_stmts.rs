use crate::errors::ErrorReport;
use crate::lexer::tokens::TokenKind::*;
use crate::parser::ast::ASTNodeKind::*;
use crate::parser::ast::*;
use crate::parser::Parser;
use crate::{curr_tk, match_tok};

impl<'a> Parser<'a> {
  /// Parses a module.
  ///
  /// ```bnf
  /// // Only function, constant, class, and enum declaration statements can be exported.
  /// MODULE ::= ("pub"? STATEMENT)* EOF
  /// ```
  pub(super) fn parse_module(&mut self) {
    while !match_tok![self, EOF] {
      // Public statement
      if match_tok![self, PUB_KW] {
        let pub_stmt = match curr_tk![self] {
          CONST_KW => todo!("Parse const declaration."),
          ENUM_KW => todo!("Parse enum declaration."),
          FUNC_KW => todo!("Parse func declaration."),
          CLASS_KW => todo!("Parse class declaration."),
          _ => {
            let err = self.error_at_previous("Only 'func', 'class', 'const', and 'enum' declarations can be public.");
            self.errors.push(err);
            continue;
          }
        };

        self.ast.attach_to_root(pub_stmt);
        continue;
      }

      // Private statement
      match self.parse_stmt() {
        Ok(node) => self.ast.attach_to_root(node),
        Err(e) => self.errors.push(e),
      }
    }
  }

  /// Parses a general statement.
  ///
  /// ```bnf
  /// STATEMENT ::= BLOCK_STMT | WHILE_STMT | FOR_STMT | LOOP_EXPR_STMT
  ///           | BREAK_STMT | CONTINUE_STMT | RETURN_STMT | YIELD_STMT
  ///           | WITH_AS_STMT | TRY_STMT | THROW_STMT | DEL_STMT | IF_STMT
  ///           | MATCH_STMT | VAR_DECL | CONST_DECL | ENUM_DECL | IMPORT_DECL
  ///           | DECORATOR_STMT* (FUNC_DECL | CLASS_DECL) | EXPR_STMT
  /// ```
  pub(super) fn parse_stmt(&mut self) -> Result<ASTNodeIdx, ErrorReport> {
    match curr_tk![self] {
      L_CURLY if self.advance() => self.parse_block_stmt(),
      WHILE_KW if self.advance() => todo!("Parse while loop."),
      FOR_KW if self.advance() => todo!("Parse for loop."),
      LOOP_KW if self.advance() => self.parse_loop_expr_stmt(false),
      BREAK_KW if self.advance() => self.parse_break_stmt(),
      CONTINUE_KW if self.advance() => self.parse_continue_stmt(),
      RETURN_KW if self.advance() => self.parse_return_stmt(),
      YIELD_KW if self.advance() => self.parse_yield_stmt(),
      WITH_KW if self.advance() => todo!("Parse with statement."),
      TRY_KW if self.advance() => todo!("Parse try statement."),
      THROW_KW if self.advance() => self.parse_throw_stmt(),
      DEL_KW if self.advance() => self.parse_del_stmt(),
      IF_KW if self.advance() => todo!("Parse if block."),
      MATCH_KW if self.advance() => todo!("Parse match block."),
      LET_KW if self.advance() => todo!("Parse let declaration."),
      CONST_KW if self.advance() => todo!("Parse const declaration."),
      ENUM_KW if self.advance() => todo!("Parse enum declaration."),
      IMPORT_KW if self.advance() => todo!("Parse import declaration."),
      AT if self.advance() => todo!("Parse decorator."),
      FUNC_KW if self.advance() => todo!("Parse func declaration."),
      CLASS_KW if self.advance() => todo!("Parse class declaration."),
      PUB_KW if self.advance() => Err(self.error_at_current("Keyword 'pub' not allowed here.")),
      _ => self.parse_expr_stmt(),
    }
  }

  /// Parses an expression statement.
  ///
  /// ```bnf
  /// EXPR_STMT ::= EXPRESSION ";"
  /// ```
  pub(super) fn parse_expr_stmt(&mut self) -> Result<ASTNodeIdx, ErrorReport> {
    let expr = self.parse_expr()?;
    self.consume(&SEMICOLON, "Expected ';' after expression.")?;
    self.emit(ExprStmt(expr))
  }

  /// Parses a block statement.
  ///
  /// ```bnf
  /// BLOCK_STMT ::= "{" STATEMENT* "}"
  /// ```
  pub(super) fn parse_block_stmt(&mut self) -> Result<ASTNodeIdx, ErrorReport> {
    let mut stmts = vec![];

    while !match_tok![self, R_CURLY] {
      stmts.push(self.parse_stmt()?);
    }

    self.emit(BlockStmt(stmts))
  }

  /// Parses a loop expression or loop statement.
  ///
  /// ```bnf
  /// LOOP_EXPR_STMT ::= "loop" BLOCK_STMT
  /// ```
  pub(super) fn parse_loop_expr_stmt(&mut self, is_expr: bool) -> Result<ASTNodeIdx, ErrorReport> {
    self.consume(&L_CURLY, "Expected '{' after loop keyword.")?;
    let body = self.parse_block_stmt()?;
    self.emit(LoopExprStmt(ASTLoopExprStmtNode { body, is_expr }))
  }

  /// Parses a break statement.
  ///
  /// ```bnf
  /// BREAK_STMT ::= "break" EXPRESSION? ";"
  /// ```
  pub(super) fn parse_break_stmt(&mut self) -> Result<ASTNodeIdx, ErrorReport> {
    let value = if !match_tok![self, SEMICOLON] {
      let val = Some(self.parse_expr()?);
      self.consume(&SEMICOLON, "Expected ';' after break statement.")?;
      val
    } else {
      None
    };

    self.emit(BreakStmt(value))
  }

  /// Parses a continue statement.
  ///
  /// ```bnf
  /// CONTINUE_STMT ::= "continue" ";"
  /// ```
  pub(super) fn parse_continue_stmt(&mut self) -> Result<ASTNodeIdx, ErrorReport> {
    self.consume(&SEMICOLON, "Expected ';' after continue statement.")?;
    self.emit(ContinueStmt)
  }

  /// Parses a return statement.
  ///
  /// ```bnf
  /// RETURN_STMT ::= "return" EXPRESSION ";"
  /// ```
  pub(super) fn parse_return_stmt(&mut self) -> Result<ASTNodeIdx, ErrorReport> {
    let stmt = ReturnStmt(self.parse_expr()?);
    self.consume(&SEMICOLON, "Expected ';' after return statement.")?;
    self.emit(stmt)
  }

  /// Parses a yield statement.
  ///
  /// ```bnf
  /// YIELD_STMT ::= "yield" EXPRESSION ";"
  /// ```
  pub(super) fn parse_yield_stmt(&mut self) -> Result<ASTNodeIdx, ErrorReport> {
    let stmt = YieldStmt(self.parse_expr()?);
    self.consume(&SEMICOLON, "Expected ';' after yield statement.")?;
    self.emit(stmt)
  }

  /// Parses a throw statement.
  ///
  /// ```bnf
  /// THROW_STMT ::= "throw" EXPRESSION ";"
  /// ```
  pub(super) fn parse_throw_stmt(&mut self) -> Result<ASTNodeIdx, ErrorReport> {
    let stmt = ThrowStmt(self.parse_primary()?);
    self.consume(&SEMICOLON, "Expected ';' after throw statement.")?;
    self.emit(stmt)
  }

  /// Parses a del statement.
  ///
  /// ```bnf
  /// DEL_STMT ::= "del" EXPRESSION ";"
  /// ```
  pub(super) fn parse_del_stmt(&mut self) -> Result<ASTNodeIdx, ErrorReport> {
    let target = self.parse_expr()?;
    // TODO: Implement node span resolution and get the span of the target instead.
    let target_tok = self.current_pos - 1;

    // Only IDENTIFIER, INDEXING_EXPR, or MEMBER_ACCESS_EXPR can be deleted.
    let stmt = match &self.ast.get(target).kind {
      Identifier(_) | Indexing(_) | MemberAccess(_) => target,
      _ => return Err(self.error_at_tok(target_tok, "Invalid del target.")),
    };

    self.consume(&SEMICOLON, "Expected ';' after del statement.")?;
    self.emit(DelStmt(stmt))
  }
}
