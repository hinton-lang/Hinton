use core::ast::ASTNodeKind::*;
use core::ast::*;
use core::tokens::TokenKind::*;

use crate::{check_tok, consume_id, curr_tk, error_at_tok, guard_error_token, match_tok, ErrMsg, NodeResult, Parser};

pub enum ParsingLevel {
  Module,
  Block,
}

impl<'a> Parser<'a> {
  /// Parses a module.
  ///
  /// ```bnf
  /// MODULE ::= STATEMENT* EOF
  /// ```
  pub(super) fn parse_module(&mut self) {
    while !match_tok![self, EOF] {
      match self.parse_stmt(ParsingLevel::Module) {
        Ok(Some(node)) => self.ast.attach_to_root(node),
        Ok(None) => continue, // Ignore semicolons
        Err(e) => {
          self.errors.push(e);

          // Synchronizes the parser when it has found an error.
          // This method helps minimize the number of cascading errors the parser emits
          // when it finds a parsing error. After an error, it skips tokens until it finds
          // a synchronization point, like the keyword for a statement.
          // NOTE: The condition was split into multiple `check_tok!` macros to fit in less lines.
          while !check_tok![self, EOF | PUB_KW | HASHTAG | WHILE_KW | FOR_KW]
            && !check_tok![self, BREAK_KW | CONTINUE_KW | RETURN_KW | YIELD_KW | WITH_KW]
            && !check_tok![self, TRY_KW | THROW_KW | DEL_KW | IF_KW | MATCH_KW]
            && !check_tok![self, LET_KW | CONST_KW | ENUM_KW | IMPORT_KW | EXPORT_KW]
            && !check_tok![self, FUNC_KW | ASYNC_KW | CLASS_KW | ABSTRACT_KW | INIT_KW]
          {
            // Skip nested semicolons and right-curly braces.
            if check_tok![self, SEMICOLON | R_CURLY] {
              while check_tok![self, SEMICOLON | R_CURLY] {
                self.advance();
              }

              break;
            }

            self.advance();
          }
        }
      }
    }
  }

  /// Parses a general statement.
  ///
  /// ```bnf
  /// STATEMENT ::= BLOCK_STMT | WHILE_LOOP_STMT | FOR_LOOP_STMT | LOOP_EXPR | BREAK_STMT
  ///           | CONTINUE_STMT | RETURN_STMT | YIELD_STMT | WITH_AS_STMT | TRY_STMT
  ///           | THROW_STMT | DEL_STMT  IF_STMT | MATCH_EXPR_STMT | VAR_DECL
  ///           | "pub"? (CONST_DECL | ENUM_DECL) | IMPORT_EXPORT_DECL
  ///           | DECORATOR_STMT* "pub"? (FUNC_DECL | CLASS_DECL) | EXPR_STMT
  ///           | ";" // ignored
  /// ```
  pub(super) fn parse_stmt(&mut self, level: ParsingLevel) -> NodeResult<Option<ASTNodeIdx>> {
    guard_error_token![self];

    // Skip semicolons at the start of new statements.
    if check_tok![self, SEMICOLON] {
      while check_tok![self, SEMICOLON] {
        self.advance();
      }

      return Ok(None);
    }

    let decorators = if match_tok![self, HASHTAG] {
      self.parse_decorator_stmt()?
    } else {
      Vec::with_capacity(0)
    };

    let is_pub = if match_tok![self, PUB_KW] {
      // Verify the statement can be public
      if let ParsingLevel::Module = level {
        if !check_tok![self, FUNC_KW | CLASS_KW | CONST_KW | ENUM_KW] {
          let err_msg = "Expected 'func', 'class', 'const', or 'enum' keyword for public declaration.";
          return Err(self.error_at_current_tok(err_msg, None));
        }
      } else {
        return Err(self.error_at_prev_tok("Keyword 'pub' not allowed here.", None));
      }

      true
    } else {
      false
    };

    // Verify the statement can be decorated
    if !decorators.is_empty() && !check_tok![self, FUNC_KW | CLASS_KW] {
      return Err(self.error_at_current_tok("Expected 'func' or 'class' declaration as decoration target.", None));
    }

    let stmt_idx = match curr_tk![self] {
      L_CURLY if self.advance() => {
        let node = self.parse_block_stmt()?;
        self.emit(BlockStmt(node))?
      }
      WHILE_KW if self.advance() => self.parse_while_loop_stmt()?,
      FOR_KW if self.advance() => self.parse_for_loop_stmt()?,
      BREAK_KW if self.advance() => self.parse_break_stmt()?,
      CONTINUE_KW if self.advance() => self.parse_continue_stmt()?,
      RETURN_KW if self.advance() => self.parse_return_stmt()?,
      YIELD_KW if self.advance() => self.parse_yield_stmt()?,
      WITH_KW if self.advance() => self.parse_with_as_stmt()?,
      TRY_KW if self.advance() => self.parse_try_stmt()?,
      THROW_KW if self.advance() => self.parse_throw_stmt()?,
      DEL_KW if self.advance() => self.parse_del_stmt()?,
      IF_KW if self.advance() => self.parse_if_stmt()?,
      MATCH_KW if self.advance() => todo!("Parse match block."),
      LET_KW if self.advance() => self.parse_var_or_const_decl(false)?,
      CONST_KW if self.advance() => self.parse_var_or_const_decl(true)?,
      ENUM_KW if self.advance() => todo!("Parse enum declaration."),
      IMPORT_KW if self.advance() => self.parse_import_export_decl(false)?,
      EXPORT_KW if self.advance() => self.parse_import_export_decl(true)?,
      FUNC_KW if self.advance() => self.parse_func_stmt(false, decorators)?,
      ASYNC_KW if self.advance() => self.parse_func_stmt(true, decorators)?,
      CLASS_KW if self.advance() => self.parse_class_decl(false, decorators)?,
      ABSTRACT_KW if self.advance() => self.parse_class_decl(true, decorators)?,
      _ => self.parse_expr_stmt()?,
    };

    // Add statement to the list of public members
    if is_pub {
      self.ast.add_pub_to_root(stmt_idx);
    }

    Ok(Some(stmt_idx))
  }

  /// Parses an expression statement.
  ///
  /// ```bnf
  /// EXPR_STMT ::= EXPRESSION ";"
  /// ```
  pub(super) fn parse_expr_stmt(&mut self) -> NodeResult<ASTNodeIdx> {
    let token = self.current_pos;
    let expr = self.parse_expr()?;
    self.consume(&SEMICOLON, "Expected ';' after expression.", None)?;
    self.emit(ExprStmt(ASTExprStmt { token, expr }))
  }

  /// Parses a block statement.
  ///
  /// ```bnf
  /// BLOCK_STMT ::= "{" STATEMENT* "}"
  /// ```
  pub(super) fn parse_block_stmt(&mut self) -> NodeResult<BlockNode> {
    let mut children = vec![];

    while !match_tok![self, R_CURLY] {
      match self.parse_stmt(ParsingLevel::Block)? {
        Some(stmt) => children.push(stmt),
        None => continue, // Ignore semicolons
      }
    }

    let close_token = self.current_pos - 1;
    Ok(BlockNode { close_token, children })
  }

  /// Parses a while-loop statement.
  ///
  /// ```bnf
  /// WHILE_LOOP_STMT ::= "while" ("let" IDENTIFIER "=")? EXPRESSION BLOCK_STMT
  /// ```
  pub(super) fn parse_while_loop_stmt(&mut self) -> NodeResult<ASTNodeIdx> {
    let token = self.current_pos - 1;
    let mut let_id = None;

    if match_tok![self, LET_KW] {
      let_id = Some(consume_id![self, "Expected identifier in while-let declaration.", None]);
      self.consume(&EQUALS, "Expected '=' after while-let declaration identifier.", None)?;
    }

    let cond = self.parse_expr()?;
    self.consume(&L_CURLY, "Expected block as 'while' loop body.", None)?;
    let body = self.parse_block_stmt()?;

    self.emit(WhileLoop(ASTWhileLoopNode { token, let_id, cond, body }))
  }

  /// Parses a for-loop statement.
  ///
  /// ```bnf
  /// FOR_LOOP_STMT ::= "for" FOR_LOOP_HEAD BLOCK_STMT
  /// ```
  pub(super) fn parse_for_loop_stmt(&mut self) -> NodeResult<ASTNodeIdx> {
    let head = self.parse_for_loop_head()?;
    self.consume(&L_CURLY, "Expected block as 'for' loop body.", None)?;
    let body = self.parse_block_stmt()?;
    self.emit(ForLoop(ASTForLoopNode { head, body }))
  }

  /// Parses a for-loop statement.
  ///
  /// ```bnf
  /// FOR_LOOP_HEAD ::= (IDENTIFIER | UNPACK_PATTERN) "in" EXPRESSION
  /// ```
  pub(super) fn parse_for_loop_head(&mut self) -> NodeResult<ForLoopHead> {
    let id = if match_tok![self, L_PAREN] {
      CompoundIdDecl::Unpack(self.parse_unpack_pattern("'for' loop")?)
    } else {
      CompoundIdDecl::Single(consume_id![self, "Expected identifier in 'for' loop", None])
    };

    self.consume(&IN_KW, "Expected keyword 'in' after for-loop identifiers.", None)?;
    let target = self.parse_expr()?;
    Ok(ForLoopHead { id, target })
  }

  /// Parses a break statement.
  ///
  /// ```bnf
  /// BREAK_STMT ::= "break" EXPRESSION? ";"
  /// ```
  pub(super) fn parse_break_stmt(&mut self) -> NodeResult<ASTNodeIdx> {
    let token = self.current_pos - 1;

    let val = if !match_tok![self, SEMICOLON] {
      let val = Some(self.parse_expr()?);
      self.consume(&SEMICOLON, "Expected ';' after break statement.", None)?;
      val
    } else {
      None
    };

    let node = ASTBreakStmtNode { token, val };
    self.emit(BreakStmt(node))
  }

  /// Parses a continue statement.
  ///
  /// ```bnf
  /// CONTINUE_STMT ::= "continue" ";"
  /// ```
  pub(super) fn parse_continue_stmt(&mut self) -> NodeResult<ASTNodeIdx> {
    let token = self.current_pos - 1;
    self.consume(&SEMICOLON, "Expected ';' after continue statement.", None)?;
    self.emit(ContinueStmt(token))
  }

  /// Parses a return statement.
  ///
  /// ```bnf
  /// RETURN_STMT ::= "return" EXPRESSION ";"
  /// ```
  pub(super) fn parse_return_stmt(&mut self) -> NodeResult<ASTNodeIdx> {
    let token = self.current_pos - 1;
    let val = self.parse_expr()?;
    self.consume(&SEMICOLON, "Expected ';' after return statement.", None)?;
    self.emit(ReturnStmt(ASTReturnStmtNode { token, val }))
  }

  /// Parses a yield statement.
  ///
  /// ```bnf
  /// YIELD_STMT ::= "yield" EXPRESSION ";"
  /// ```
  pub(super) fn parse_yield_stmt(&mut self) -> NodeResult<ASTNodeIdx> {
    let stmt = YieldStmt(self.parse_expr()?);
    self.consume(&SEMICOLON, "Expected ';' after yield statement.", None)?;
    self.emit(stmt)
  }

  /// Parses a throw statement.
  ///
  /// ```bnf
  /// THROW_STMT ::= "throw" EXPRESSION ";"
  /// ```
  pub(super) fn parse_throw_stmt(&mut self) -> NodeResult<ASTNodeIdx> {
    let stmt = ThrowStmt(self.parse_primary()?);
    self.consume(&SEMICOLON, "Expected ';' after throw statement.", None)?;
    self.emit(stmt)
  }

  /// Parses a del statement.
  ///
  /// ```bnf
  /// DEL_STMT ::= "del" EXPRESSION ";"
  /// ```
  pub(super) fn parse_del_stmt(&mut self) -> NodeResult<ASTNodeIdx> {
    let target = self.parse_expr()?;
    // TODO: Implement node span resolution and get the span of the target instead.
    let target_tok = self.current_pos - 1;

    // Only IDENTIFIER, INDEXING_EXPR, or MEMBER_ACCESS_EXPR can be deleted.
    let stmt = match &self.ast.get(target) {
      IdLiteral(_) | Indexing(_) | MemberAccess(_) => target,
      _ => {
        let err_msg = ErrMsg::Syntax("Invalid del target.".to_string());
        return Err(error_at_tok(target_tok, err_msg, None));
      }
    };

    self.consume(&SEMICOLON, "Expected ';' after del statement.", None)?;
    self.emit(DelStmt(stmt))
  }

  /// Parses an if statement.
  ///
  /// ```bnf
  /// IF_STMT ::= "if" EXPRESSION BLOCK_STMT ("else" (BLOCK_STMT | IF_STMT))?
  /// ```
  pub(super) fn parse_if_stmt(&mut self) -> NodeResult<ASTNodeIdx> {
    let token = self.current_pos - 1;
    let cond = self.parse_expr()?;
    self.consume(&L_CURLY, "Expected block as 'if' statement body", None)?;
    let true_branch = self.parse_block_stmt()?;
    let mut else_branch = ElseBranch::None;

    if match_tok![self, ELSE_KW] {
      else_branch = match curr_tk![self] {
        IF_KW if self.advance() => ElseBranch::IfStmt(self.parse_if_stmt()?),
        L_CURLY if self.advance() => ElseBranch::Block(self.parse_block_stmt()?),
        _ => return Err(self.error_at_current_tok("Expected block or 'if' statement after 'else' keyword.", None)),
      }
    }

    let node = ASTIfStmtNode {
      token,
      cond,
      true_branch,
      else_branch,
    };
    self.emit(IfStmt(node))
  }

  /// Parses a with-as statement.
  ///
  /// ```bnf
  /// WITH_AS_STMT ::= "with" WITH_STMT_HEAD ("," WITH_STMT_HEAD)* BLOCK_STMT
  /// ```
  pub(super) fn parse_with_as_stmt(&mut self) -> NodeResult<ASTNodeIdx> {
    let mut heads = vec![self.parse_with_stmt_head()?];

    while match_tok![self, COMMA] {
      heads.push(self.parse_with_stmt_head()?);
    }

    self.consume(&L_CURLY, "Expected block as 'with' statement body.", None)?;
    let body = self.parse_block_stmt()?;
    self.emit(WithStmt(ASTWithStmtNode { heads, body }))
  }

  /// Parses a single with-as statement head.
  ///
  /// ```bnf
  /// WITH_STMT_HEAD ::= EXPRESSION "as" IDENTIFIER
  /// ```
  pub(super) fn parse_with_stmt_head(&mut self) -> NodeResult<WithStmtHead> {
    let expr = self.parse_expr()?;
    self.consume(&AS_KW, "Expected 'as' keyword in 'with' statement head.", None)?;
    let id = consume_id![self, "Expected identifier in 'with' statement head.", None];
    Ok(WithStmtHead { expr, id })
  }

  /// Parses a single with-as statement head.
  ///
  /// ```bnf
  /// TRY_STMT            ::= "try" BLOCK_STMT NAMED_CATCH_PART+
  ///                     | "try" BLOCK_STMT NAMED_CATCH_PART* (DEFAULT_CATCH_PART | FINALLY_PART)
  ///                     | "try" BLOCK_STMT NAMED_CATCH_PART+ DEFAULT_CATCH_PART FINALLY_PART
  /// NAMED_CATCH_PART    ::= "catch" IDENTIFIER ("as" IDENTIFIER)? BLOCK_STMT
  /// DEFAULT_CATCH_PART  ::= "catch" BLOCK_STMT
  /// FINALLY_PART        ::= "finally" BLOCK_STMT
  /// ```
  pub(super) fn parse_try_stmt(&mut self) -> NodeResult<ASTNodeIdx> {
    self.consume(&L_CURLY, "Expected block as 'try' body.", None)?;
    let body = self.parse_block_stmt()?;
    let mut has_default_catch = false;

    let mut catchers = vec![];
    let mut finally = None;

    loop {
      match curr_tk![self] {
        FINALLY_KW if self.advance() => {
          if finally.is_some() {
            return Err(
              self.error_at_prev_tok("A try-catch-finally statement can only have one 'finally' block.", None),
            );
          }

          self.consume(&L_CURLY, "Expected block as 'finally' body.", None)?;
          finally = Some(self.parse_block_stmt()?);
        }
        CATCH_KW if self.advance() => {
          if finally.is_some() {
            return Err(self.error_at_prev_tok("A 'catch' block cannot follow a 'finally' block.", None));
          }

          let catch_part = self.parse_catch_part(has_default_catch)?;
          has_default_catch = catch_part.target.is_none();
          catchers.push(catch_part);
        }
        _ if finally.is_none() && catchers.is_empty() => {
          return Err(self.error_at_current_tok("Expected 'catch' or 'finally' block after 'try' block.", None))
        }
        _ => break,
      }
    }

    self.emit(TryCatchFinally(ASTTryCatchFinallyNode { body, catchers, finally }))
  }

  /// Parses a single with-as statement head.
  ///
  /// ```bnf
  /// NAMED_CATCH_PART    ::= "catch" IDENTIFIER ("as" IDENTIFIER)? BLOCK_STMT
  /// DEFAULT_CATCH_PART  ::= "catch" BLOCK_STMT
  /// ```
  pub(super) fn parse_catch_part(&mut self, has_default_catch: bool) -> NodeResult<CatchPart> {
    if match_tok![self, L_CURLY] {
      if has_default_catch {
        return Err(self.error_at_prev_tok(
          "A try-catch-finally statement can only have one default 'catch' block.",
          None,
        ));
      }

      let body = self.parse_block_stmt()?;
      Ok(CatchPart { body, target: None })
    } else {
      if has_default_catch {
        return Err(self.error_at_prev_tok("Non-default 'catch' block cannot follow default 'catch' block.", None));
      }

      let error_class = consume_id![self, "Expected error name in 'catch' block.", None];
      let error_result = if match_tok![self, AS_KW] {
        Some(consume_id![self, "Expected error class name in 'catch' block.", None])
      } else {
        None
      };

      let target = Some(CatchTarget { error_class, error_result });
      self.consume(&L_CURLY, "Expected block as 'catch' body.", None)?;
      let body = self.parse_block_stmt()?;

      Ok(CatchPart { body, target })
    }
  }

  /// Parses an import statement or an export statement.
  ///
  /// ```bnf
  /// IMPORT_EXPORT_DECL ::= ("import" | "export") ((GRANULAR_IMPORT | "..." IDENTIFIER) "from")? STRING_LITERAL ";"
  ///                    | ("import" | "export") GRANULAR_IMPORT "," "..." IDENTIFIER "from" STRING_LITERAL ";"
  ///                    | ("import" | "export") "..." IDENTIFIER "," GRANULAR_IMPORT "from" STRING_LITERAL ";"
  /// ```
  pub(super) fn parse_import_export_decl(&mut self, is_export: bool) -> NodeResult<ASTNodeIdx> {
    let decl_name = if is_export { "export" } else { "import" };

    let mut members = vec![];
    let mut wildcard = None;

    if !check_tok![self, STR_LIT] {
      // Match first wildcard or granular import
      match curr_tk![self] {
        L_CURLY if self.advance() => members = self.parse_granular_import(decl_name)?,
        TRIPLE_DOT if self.advance() => {
          let err_msg = &format!("Expected identifier for wildcard {}.", decl_name);
          wildcard = Some(consume_id![self, err_msg, None])
        }
        _ => return Err(self.error_at_current_tok(&format!("Expected {} declaration body.", decl_name), None)),
      }

      // Then, if next is a comma, match another wildcard or granular import
      if match_tok![self, COMMA] {
        if !members.is_empty() && !check_tok![self, TRIPLE_DOT] {
          let err_msg = &format!("Expected wildcard {} after granular {}.", decl_name, decl_name);
          return Err(self.error_at_current_tok(err_msg, None));
        } else if wildcard.is_some() && !check_tok![self, L_CURLY] {
          let err_msg = &format!("Expected granular {} after wildcard {}.", decl_name, decl_name);
          return Err(self.error_at_current_tok(err_msg, None));
        }

        if match_tok![self, L_CURLY] {
          members = self.parse_granular_import(decl_name)?;
        } else if match_tok![self, TRIPLE_DOT] {
          let err_msg = &format!("Expected identifier for wildcard {}.", decl_name);
          wildcard = Some(consume_id![self, err_msg, None])
        }
      }

      // Finally, consume the "from" keyword
      let err_msg = &format!("Expected keyword 'from' for {} declaration.", decl_name);
      self.consume(&FROM_KW, err_msg, None)?;
    }

    let err_msg = &format!("Expected module path for {} declaration.", decl_name);
    self.consume(&STR_LIT, err_msg, None)?;
    let path = self.emit(StringLiteral(self.current_pos - 1))?;

    self.consume(
      &SEMICOLON,
      &format!("Expected ';' after {} declaration.", decl_name),
      None,
    )?;

    let node = ASTImportExportNode { members, wildcard, path };
    self.emit(if is_export { ExportDecl(node) } else { ImportDecl(node) })
  }

  /// Parses the body of a granular import or export.
  ///
  /// ```bnf
  /// GRANULAR_IMPORT ::= "{" IDENTIFIER ("as" IDENTIFIER)? ("," IDENTIFIER ("as" IDENTIFIER)?)* ","? "}"
  /// ```
  pub(super) fn parse_granular_import(&mut self, decl_name: &str) -> NodeResult<Vec<ImportExportMember>> {
    let mut members = vec![];

    loop {
      let member = consume_id![self, &format!("Expected identifier to {}.", decl_name), None];

      // Maybe parse alias
      let alias = if match_tok![self, AS_KW] {
        let err_msg = &format!("Expected identifier for {} alias.", decl_name);
        Some(consume_id![self, err_msg, None])
      } else {
        None
      };

      members.push(ImportExportMember { member, alias });

      // Optional trailing comma
      if (match_tok![self, COMMA] && match_tok![self, R_CURLY]) || match_tok![self, R_CURLY] {
        break;
      }
    }

    Ok(members)
  }

  /// Parses a decorator statement.
  ///
  /// ```bnf
  /// DECORATOR_STMT ::= "#" (DECORATOR_BDY | "[" DECORATOR_BDY ("," DECORATOR_BDY)* ","? "]")
  /// ```
  pub(super) fn parse_decorator_stmt(&mut self) -> NodeResult<Vec<Decorator>> {
    let mut decorators = vec![];

    loop {
      if match_tok![self, L_BRACKET] {
        loop {
          decorators.push(self.parse_decorator_body()?);

          // Optional trailing comma
          if (match_tok![self, COMMA] && match_tok![self, R_BRACKET]) || match_tok![self, R_BRACKET] {
            break;
          }
        }
      } else {
        decorators.push(self.parse_decorator_body()?);
      }

      if !match_tok![self, HASHTAG] {
        break;
      }
    }

    // Match a function or class declaration after the decorators.
    Ok(decorators)
  }

  /// Parses a decorator body.
  ///
  /// ```bnf
  /// DECORATOR_BDY ::= IDENTIFIER | CALL_EXPR
  /// ```
  pub(super) fn parse_decorator_body(&mut self) -> NodeResult<Decorator> {
    let expr = self.parse_expr()?;

    let decorator = match self.ast.get(expr) {
      IdLiteral(_) | CallExpr(_) => expr,
      // TODO: Implement node span resolution and get the span of the target instead.
      _ => return Err(self.error_at_prev_tok("Expected identifier or function call as decorator.", None)),
    };

    Ok(Decorator(decorator))
  }
}
