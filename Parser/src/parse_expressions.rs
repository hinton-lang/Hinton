use core::ast::ASTNodeKind::*;
use core::ast::*;
use core::tokens::TokenKind::*;

use crate::{check_tok, consume_id, curr_tk, error_at_tok, guard_error_token, match_tok, ErrMsg, NodeResult, Parser};

macro_rules! append_binary_expr {
  ($s:ident, $l:expr, $r:expr, $k:expr, $tok:expr) => {
    $s.ast.push(BinaryExpr(ASTBinaryExprNode {
      left: $l,
      right: $r,
      kind: $k,
      token: $tok,
    }))
  };
}

impl<'a> Parser<'a> {
  /// Parses a general expression.
  ///
  /// ```bnf
  /// EXPRESSION ::= ASSIGNMENT_EXPR
  /// ```
  pub(super) fn parse_expr(&mut self) -> NodeResult<ASTNodeIdx> {
    guard_error_token![self];
    self.parse_reassignment()
  }

  /// Parses a reassignment expression.
  ///
  /// ```bnf
  /// REASSIGNMENT_EXPR ::= TERNARY_EXPR (ASSIGNMENT_OPR EXPRESSION)?
  /// ```
  pub(super) fn parse_reassignment(&mut self) -> NodeResult<ASTNodeIdx> {
    let target = self.parse_ternary_expr()?;
    let target_tok = self.current_pos - 1;

    if let Some(kind) = ReassignmentKind::try_from_token(self.get_curr_tk()) {
      self.advance(); // Consume the token
      let operator = self.current_pos - 1;

      // Gets the value for assignment
      let value = self.parse_expr()?;

      // Returns the assignment expression of the corresponding type
      return match &self.ast.get(target) {
        // In the compiler, we simply check the kind of target we have
        // to emit the correct set of bytecode instructions.
        IdLiteral(_) | MemberAccess(_) | Indexing(_) => {
          let node = ASTReassignmentNode { target, operator, kind, value };
          self.emit(Reassignment(node))
        }
        _ => Err(error_at_tok(
          target_tok,
          ErrMsg::Syntax("Invalid assignment target.".to_string()),
          Some("Verify this is an identifier, object member, or indexing expression.".to_string()),
        )),
      };
    }

    Ok(target)
  }

  /// Parses a ternary-conditional expression.
  ///
  /// ```bnf
  /// TERNARY_EXPR ::= NONE_COALESCE_EXPR ("?" EXPRESSION ":" EXPRESSION)?
  /// ```
  pub(super) fn parse_ternary_expr(&mut self) -> NodeResult<ASTNodeIdx> {
    let condition = self.parse_nonish_coalescing()?;

    if match_tok![self, QUESTION] {
      let branch_true = self.parse_expr()?;
      let hint = Some("Ternary conditionals require an 'else' branch.");
      self.consume(&COLON, "Expected ':' after the expression.", hint)?;
      let branch_false = self.parse_expr()?;

      let node = ASTTernaryConditionalNode { condition, branch_true, branch_false };

      return self.emit(TernaryConditional(node));
    }

    Ok(condition)
  }

  /// Parses a none-coalescing expression.
  ///
  /// ```bnf
  /// NONE_COALESCE_EXPR ::= LOGIC_OR_EXPR ("??" LOGIC_OR_EXPR)*
  /// ```
  pub(super) fn parse_nonish_coalescing(&mut self) -> NodeResult<ASTNodeIdx> {
    let mut left = self.parse_logic_or()?;

    while match_tok![self, NONISH] {
      let tok = self.current_pos - 1;
      let right = self.parse_logic_or()?;
      left = append_binary_expr![self, left, right, BinaryExprKind::Nonish, tok];
    }

    Ok(left)
  }

  /// Parses a logic-or expression.
  ///
  /// ```bnf
  /// LOGIC_OR_EXPR ::= LOGIC_AND_EXPR (("||" | "or") LOGIC_AND_EXPR)*
  /// ```
  pub(super) fn parse_logic_or(&mut self) -> NodeResult<ASTNodeIdx> {
    let mut left = self.parse_logic_and()?;

    while match_tok![self, DOUBLE_VERT_BAR | OR_KW] {
      let tok = self.current_pos - 1;
      let right = self.parse_logic_and()?;
      left = append_binary_expr![self, left, right, BinaryExprKind::LogicOR, tok];
    }

    Ok(left)
  }

  /// Parses a logic-and expression.
  ///
  /// ```bnf
  /// LOGIC_AND_EXPR ::= BITWISE_OR_EXPR (("&&" | "and") BITWISE_OR_EXPR)*
  /// ```
  pub(super) fn parse_logic_and(&mut self) -> NodeResult<ASTNodeIdx> {
    let mut left = self.parse_bit_or()?;

    while match_tok![self, DOUBLE_AMPERSAND | AND_KW] {
      let tok = self.current_pos - 1;
      let right = self.parse_bit_or()?;
      left = append_binary_expr![self, left, right, BinaryExprKind::LogicAND, tok];
    }

    Ok(left)
  }

  /// Parses a bitwise-or expression.
  ///
  /// ```bnf
  /// BITWISE_OR_EXPR ::= BITWISE_XOR_EXPR ("|" BITWISE_XOR_EXPR)*
  /// ```
  pub(super) fn parse_bit_or(&mut self) -> NodeResult<ASTNodeIdx> {
    let mut left = self.parse_bit_xor()?;

    while match_tok![self, VERT_BAR] {
      let tok = self.current_pos - 1;
      let right = self.parse_bit_xor()?;
      left = append_binary_expr![self, left, right, BinaryExprKind::BitOR, tok];
    }

    Ok(left)
  }

  /// Parses a bitwise-xor expression.
  ///
  /// ```bnf
  /// BITWISE_XOR_EXPR ::= BITWISE_AND_EXPR ("^" BITWISE_AND_EXPR)*
  /// ```
  pub(super) fn parse_bit_xor(&mut self) -> NodeResult<ASTNodeIdx> {
    let mut left = self.parse_bit_and()?;

    while match_tok![self, BIT_XOR] {
      let tok = self.current_pos - 1;
      let right = self.parse_bit_and()?;
      left = append_binary_expr![self, left, right, BinaryExprKind::BitXOR, tok];
    }

    Ok(left)
  }

  /// Parses a bitwise-and expression.
  ///
  /// ```bnf
  /// BITWISE_AND_EXPR ::= EQUALITY_EXPR ("&" EQUALITY_EXPR)*
  /// ```
  pub(super) fn parse_bit_and(&mut self) -> NodeResult<ASTNodeIdx> {
    let mut left = self.parse_equality()?;

    while match_tok![self, AMPERSAND] {
      let tok = self.current_pos - 1;
      let right = self.parse_equality()?;
      left = append_binary_expr![self, left, right, BinaryExprKind::BitAND, tok];
    }

    Ok(left)
  }

  /// Parses an equality expression.
  ///
  /// ```bnf
  /// EQUALITY_EXPR ::= RELATION_EXPR (("!=" | "==") RELATION_EXPR)*
  /// ```
  pub(super) fn parse_equality(&mut self) -> NodeResult<ASTNodeIdx> {
    let mut left = self.parse_relation()?;

    // ==, !=
    while let Some(eq) = BinaryExprKind::try_equality(self.get_curr_tk()) {
      self.advance(); // Consume the token
      let tok = self.current_pos - 1;
      let right = self.parse_relation()?;
      left = append_binary_expr![self, left, right, eq, tok];
    }

    Ok(left)
  }

  /// Parses a relation expression.
  ///
  /// ```bnf
  /// RELATION_EXPR ::= BITWISE_SHIFT ((">" | ">=" | "<" | "<=" | "in" | "instof") BITWISE_SHIFT)*
  /// ```
  pub(super) fn parse_relation(&mut self) -> NodeResult<ASTNodeIdx> {
    let mut left = self.parse_bit_shift()?;

    // >, >=, <, <=, in, instof
    while let Some(eq) = BinaryExprKind::try_relation(self.get_curr_tk()) {
      self.advance(); // Consume the token
      let tok = self.current_pos - 1;
      let right = self.parse_bit_shift()?;
      left = append_binary_expr![self, left, right, eq, tok];
    }

    Ok(left)
  }

  /// Parses a bitwise-shift expression.
  ///
  /// ```bnf
  /// BITWISE_SHIFT ::= RANGE_EXPR (("<<" | ">>") RANGE_EXPR)*
  /// ```
  pub(super) fn parse_bit_shift(&mut self) -> NodeResult<ASTNodeIdx> {
    let mut left = self.parse_range()?;

    // >>, <<
    while let Some(eq) = BinaryExprKind::try_bit_shift(self.get_curr_tk()) {
      self.advance(); // Consume the token
      let tok = self.current_pos - 1;
      let right = self.parse_range()?;
      left = append_binary_expr![self, left, right, eq, tok];
    }

    Ok(left)
  }

  /// Parses a range expression.
  ///
  /// ```bnf
  /// RANGE_EXPR ::= TERM_EXPR ((".." | "..=") TERM_EXPR)?
  /// ```
  pub(super) fn parse_range(&mut self) -> NodeResult<ASTNodeIdx> {
    let mut left = self.parse_term()?;

    // .., ..=
    while let Some(eq) = BinaryExprKind::try_range(self.get_curr_tk()) {
      self.advance(); // Consume the token
      let tok = self.current_pos - 1;
      let right = self.parse_term()?;
      left = append_binary_expr![self, left, right, eq, tok];
    }

    Ok(left)
  }

  /// Parses a term expression.
  ///
  /// ```bnf
  /// TERM_EXPR ::= FACTOR_EXPR (( "-" | "+") FACTOR_EXPR)*
  /// ```
  pub(super) fn parse_term(&mut self) -> NodeResult<ASTNodeIdx> {
    let mut left = self.parse_factor()?;

    // +, -
    while let Some(eq) = BinaryExprKind::try_term(self.get_curr_tk()) {
      self.advance(); // Consume the token
      let tok = self.current_pos - 1;
      let right = self.parse_factor()?;
      left = append_binary_expr![self, left, right, eq, tok];
    }

    Ok(left)
  }

  /// Parses a factor expression.
  ///
  /// ```bnf
  /// FACTOR_EXPR ::= POW_EXPR (( "/" | "*" | "%" | "mod" | "@") POW_EXPR)*
  /// ```
  pub(super) fn parse_factor(&mut self) -> NodeResult<ASTNodeIdx> {
    let mut left = self.parse_pow()?;

    // *, %, mod, /, @
    while let Some(eq) = BinaryExprKind::try_factor(self.get_curr_tk()) {
      self.advance(); // Consume the token
      let tok = self.current_pos - 1;
      let right = self.parse_pow()?;
      left = append_binary_expr![self, left, right, eq, tok];
    }

    Ok(left)
  }

  /// Parses a power expression.
  ///
  /// ```bnf
  /// POW_EXPR ::= PIPE_EXPR ("**" PIPE_EXPR)*
  /// ```
  pub(super) fn parse_pow(&mut self) -> NodeResult<ASTNodeIdx> {
    let mut left = self.parse_pipe()?;

    while match_tok![self, POW] {
      let tok = self.current_pos - 1;
      let right = self.parse_pipe()?;
      left = append_binary_expr![self, left, right, BinaryExprKind::Pow, tok];
    }

    Ok(left)
  }

  /// Parses a pipe expression.
  ///
  /// ```bnf
  /// PIPE_EXPR ::= UNARY_EXPR ("|>" UNARY_EXPR)*
  /// ```
  pub(super) fn parse_pipe(&mut self) -> NodeResult<ASTNodeIdx> {
    let mut left = self.parse_unary()?;

    while match_tok![self, PIPE] {
      let tok = self.current_pos - 1;
      let right = self.parse_unary()?;
      left = append_binary_expr![self, left, right, BinaryExprKind::Pipe, tok];
    }

    Ok(left)
  }

  /// Parses a unary expression.
  ///
  /// ```bnf
  /// UNARY_EXPR ::= (UNARY_OPR | "new" | "await" | "typeof") UNARY_EXPR
  ///            | PRIMARY_EXPR
  /// ```
  pub(super) fn parse_unary(&mut self) -> NodeResult<ASTNodeIdx> {
    let token = self.current_pos;
    // !, ~, -, new, typeof, await
    if let Some(kind) = UnaryExprKind::try_from_token(self.get_curr_tk()) {
      self.advance(); // Consume the token

      let operand = self.parse_unary()?;
      let node = ASTUnaryExprNode { token, kind, operand };
      self.emit(UnaryExpr(node))
    } else {
      self.parse_primary()
    }
  }

  /// Parses a primary expression.
  ///
  /// ```bnf
  /// PRIMARY_EXPR ::= LAMBDA_EXPR
  ///              | LARGE_EXPR (INDEXING_EXPR | CALL_EXPR | MEMBER_ACCESS_EXPR)*
  /// ```
  pub(super) fn parse_primary(&mut self) -> NodeResult<ASTNodeIdx> {
    if match_tok![self, ASYNC_KW] {
      return self.parse_lambda_literal(true);
    } else if match_tok![self, VERT_BAR] || match_tok![self, DOUBLE_VERT_BAR] {
      return self.parse_lambda_literal(false);
    }

    let mut expr = match curr_tk![self] {
      MATCH_KW if self.advance() => todo!("Parse `match` expression."),
      _ => self.parse_literal()?,
    };

    loop {
      expr = match curr_tk![self] {
        L_BRACKET if self.advance() => self.parse_indexing_expr(expr)?,
        L_PAREN if self.advance() => self.parse_call_expr(expr)?,
        DOT | SAFE_ACCESS if self.advance() => self.parse_member_access_expr(expr)?,
        _ => break,
      }
    }

    Ok(expr)
  }

  /// Parses a lambda literal expression.
  ///
  /// ```bnf
  /// LAMBDA_EXPR ::= "async"? "|" PARAMETERS? "|" (EXPRESSION | BLOCK_STMT)
  /// ```
  pub(super) fn parse_lambda_literal(&mut self, is_async: bool) -> NodeResult<ASTNodeIdx> {
    let should_parse_params = if is_async {
      if match_tok![self, DOUBLE_VERT_BAR] {
        false
      } else if match_tok![self, VERT_BAR] {
        true
      } else {
        return Err(self.error_at_current_tok("Expected '|' for async lambda parameters.", None));
      }
    } else {
      matches!(self.get_prev_tk(), VERT_BAR)
    };

    let (min_arity, max_arity, params) = if should_parse_params {
      let params = self.parse_func_params(true)?;
      self.consume(&VERT_BAR, "Expected '|' after lambda parameter list.", None)?;
      params
    } else {
      (0u16, Some(0u16), vec![])
    };

    let body = if match_tok![self, L_CURLY] {
      LambdaBody::Block(self.parse_block_stmt()?)
    } else {
      LambdaBody::Expr(self.parse_expr()?)
    };

    self.emit(Lambda(ASTLambdaNode { is_async, params, min_arity, max_arity, body }))
  }

  /// Parses an indexing expression.
  ///
  /// ```bnf
  /// INDEXING_EXPR ::= "[" INDEXER ("," INDEXER)* "]"
  /// ```
  pub(super) fn parse_indexing_expr(&mut self, target: ASTNodeIdx) -> NodeResult<ASTNodeIdx> {
    let mut indexers = vec![self.parse_indexer()?];

    while match_tok![self, COMMA] {
      indexers.push(self.parse_indexer()?);
    }

    self.consume(&R_BRACKET, "Expected matching ']' for indexing expression.", None)?;
    let node = ASTIndexingNode { target, indexers };
    self.emit(Indexing(node))
  }

  /// Parses the indexer of an indexing expression.
  ///
  /// ```bnf
  /// INDEXER ::= EXPRESSION | SLICE
  /// ```
  pub(super) fn parse_indexer(&mut self) -> NodeResult<ASTNodeIdx> {
    if match_tok![self, COLON] {
      // The indexer is a slice with no lower bound
      self.parse_slice(None)
    } else {
      let expr = self.parse_expr()?;

      if match_tok![self, COLON] {
        // The indexer is a slice with a lower bound
        self.parse_slice(Some(expr))
      } else {
        // The indexer is an expression of the form `[a]`
        Ok(expr)
      }
    }
  }

  /// Parses a slice in an indexing expression.
  ///
  /// ```bnf
  /// SLICE ::= EXPRESSION? ":" EXPRESSION? (":" EXPRESSION?)?
  /// ```
  pub(super) fn parse_slice(&mut self, lower: Option<ASTNodeIdx>) -> NodeResult<ASTNodeIdx> {
    let upper = if !check_tok![self, COMMA | R_BRACKET] {
      Some(self.parse_expr()?)
    } else {
      None
    };

    let step = if match_tok![self, COLON] && !check_tok![self, COMMA | R_BRACKET] {
      Some(self.parse_expr()?)
    } else {
      None
    };

    self.emit(ArraySlice(ASTArraySliceNode { lower, upper, step }))
  }

  /// Parses a function call expression.
  ///
  /// ```bnf
  /// CALL_EXPR     ::= "(" ((NON_VAL_ARGS | (EXPRESSION ("," EXPRESSION)*)) ("," NON_VAL_ARGS)*)? ")"
  /// NON_VAL_ARGS  ::= SINGLE_SPREAD_EXPR | NAMED_ARGS
  /// NAMED_ARGS    ::= IDENTIFIER ":=" EXPRESSION
  /// ```
  pub(super) fn parse_call_expr(&mut self, target: ASTNodeIdx) -> NodeResult<ASTNodeIdx> {
    let token = self.current_pos - 1;

    if match_tok![self, R_PAREN] {
      return self.emit(CallExpr(ASTCallExprNode { token, target, args: vec![] }));
    }

    let mut has_non_val_arg = false;
    let mut args = vec![];

    loop {
      args.push(match curr_tk![self] {
        TRIPLE_DOT if self.advance() => {
          has_non_val_arg = true;
          CallArg::Rest(self.parse_single_spread_expr()?)
        }
        IDENTIFIER if matches![self.get_next_tk(), COLON_EQUALS] => {
          let tok_id = consume_id![self, "Expected identifier for named argument.", None];
          self.consume(&COLON_EQUALS, "Expected ':=' for named argument.", None)?;
          has_non_val_arg = true;
          CallArg::Named { name: tok_id, value: self.parse_expr()? }
        }
        _ => {
          let arg = self.parse_expr()?;

          if has_non_val_arg {
            return Err(self.error_at_prev_tok("Value arguments cannot follow named or rest arguments.", None));
          }

          CallArg::Val(arg)
        }
      });

      // Optional trailing comma
      if !check_tok![self, COMMA] || (match_tok![self, COMMA] && check_tok![self, R_PAREN]) {
        break;
      }
    }

    self.consume(&R_PAREN, "Expected ')' for function call.", None)?;
    self.emit(CallExpr(ASTCallExprNode { token, target, args }))
  }

  /// Parses a member access expression.
  ///
  /// ```bnf
  /// MEMBER_ACCESS_EXPR ::= ("." | "?.") IDENTIFIER
  /// ```
  pub(super) fn parse_member_access_expr(&mut self, target: ASTNodeIdx) -> NodeResult<ASTNodeIdx> {
    let is_safe = match self.get_prev_tk() {
      DOT => false,
      SAFE_ACCESS => true,
      _ => unreachable!("Should have parsed either a `.` or `?.` by now."),
    };

    let member = consume_id![self, "Expected member name after the dot.", None];
    self.emit(MemberAccess(ASTMemberAccessNode { is_safe, target, member }))
  }

  /// Parses a literal expression.
  ///
  /// ```bnf
  /// LITERAL_EXPR  ::= IDENTIFIER | INTEGER_LITERAL | FLOAT_LITERAL | SCIENTIFIC_LITERAL
  ///               | HEX_LITERAL | OCT_LITERAL | BINARY_LITERAL | STRING_LITERAL | ARRAY_LITERAL
  ///               | TUPLE_LITERAL | DICT_LITERAL | TRUE_LITERAL | FALSE_LITERAL | NONE_LITERAL
  ///               | SELF_LITERAL | SUPER_LITERAL | "(" EXPRESSION ")"
  /// ```
  pub(super) fn parse_literal(&mut self) -> NodeResult<ASTNodeIdx> {
    match curr_tk![self] {
      // Numeric literals
      INT_LIT | FLOAT_LIT | HEX_LIT | OCTAL_LIT | BINARY_LIT | SCIENTIFIC_LIT if self.advance() => {
        self.emit(NumLiteral(self.current_pos - 1))
      }

      // String literals
      STR_LIT if self.advance() => self.emit(StringLiteral(self.current_pos - 1)),
      START_INTERPOL_STR if self.advance() => self.parse_str_interpolation(),

      // Atomic literals
      TRUE_LIT if self.advance() => self.emit(TrueLiteral(self.current_pos - 1)),
      FALSE_LIT if self.advance() => self.emit(FalseLiteral(self.current_pos - 1)),
      NONE_LIT if self.advance() => self.emit(NoneLiteral(self.current_pos - 1)),

      // Symbolic reference literals
      IDENTIFIER if self.advance() => self.emit(IdLiteral(self.current_pos - 1)),
      SELF_KW if self.advance() => self.emit(SelfLiteral(self.current_pos - 1)),
      SUPER_KW if self.advance() => self.emit(SuperLiteral(self.current_pos - 1)),

      // Collection literals
      L_BRACKET if self.advance() => self.parse_array_literal(),
      L_PAREN if self.advance() => self.parse_tuple_literal_or_grouping_expr(),
      L_CURLY if self.advance() => self.parse_dict_literal(),

      // Unknown expression
      _ => Err(self.error_at_current_tok("Unexpected token.", None)),
    }
  }

  /// Parses a string interpolation expression.
  ///
  /// ```bnf
  /// STRING_LITERAL    ::= ('"' STRING_SEQUENCE* '"') | ("'" STRING_SEQUENCE* "'")
  /// STRING_SEQUENCE   ::= ALMOST_ANY_CHAR
  ///                   | ESCAPED_CHAR
  ///                   | "$" "{" EXPRESSION "}"
  /// ```
  fn parse_str_interpolation(&mut self) -> NodeResult<ASTNodeIdx> {
    let token = self.current_pos - 1;
    let mut parts = vec![];

    while !match_tok![self, END_INTERPOL_STR] {
      let interpol_part = match curr_tk![self] {
        STR_LIT if self.advance() => self.emit(StringLiteral(self.current_pos - 1))?,
        START_INTERPOL_EXPR if self.advance() => {
          let expr = self.parse_expr()?;
          self.consume(&END_INTERPOL_EXPR, "Expected '}' after string interpolation.", None)?;
          expr
        }
        _ => return Err(self.error_at_current_tok("Invalid interpolation part.", None)),
      };

      parts.push(interpol_part);
    }

    let node = ASTStringInterpol { parts, token };
    self.emit(StringInterpol(node))
  }
}
