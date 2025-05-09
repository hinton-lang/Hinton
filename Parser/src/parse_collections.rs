use core::ast::ASTNodeKind::*;
use core::ast::*;
use core::tokens::TokenKind::*;

use crate::{check_tok, curr_tk, match_tok, NodeResult, Parser, TokenIdx};

impl<'a> Parser<'a> {
  /// Parses an array literal expression.
  ///
  /// ```bnf
  /// ARRAY_LITERAL ::= "[" ARR_TPL_BODY? "]"
  /// ARR_TPL_BODY  ::= ARR_TPL_LIST | ARR_TPL_REPEAT | COMPACT_FOR_LOOP
  /// ```
  pub(super) fn parse_array_literal(&mut self) -> NodeResult<ASTNodeIdx> {
    let token = self.current_pos - 1;
    // If we match a for-keyword at the start the array literal, then we
    // know we have an array comprehension so we can go ahead and parse it.
    if match_tok![self, FOR_KW] {
      let expr = self.parse_compact_arr_or_tpl(false)?;
      self.consume(&R_BRACKET, "Expected matching ']' for array literal.", None)?;
      return Ok(expr);
    }

    // Initialize the list of values for array literal
    let mut values = vec![];

    if !match_tok![self, R_BRACKET] {
      // Get the first value of the array
      values.push(match curr_tk![self] {
        TRIPLE_DOT if self.advance() => self.parse_single_spread_expr()?,
        _ => {
          let value = self.parse_expr()?;

          // If there is a semicolon after the first value, then
          // we know this must be an array-repeat expression.
          if match_tok![self, SEMICOLON] {
            return self.parse_repeat_arr_or_tpl(token, value, false);
          }

          value
        }
      });

      // Parse the rest of the array's body
      values.append(&mut self.parse_array_or_tuple_list(false)?);
      self.consume(&R_BRACKET, "Expected matching ']' for array literal.", None)?;
    }

    let node = ASTArrayLiteralNode { token, values };
    self.emit(ArrayLiteral(node))
  }

  /// Parses either a tuple literal expression or a grouping expression.
  ///
  /// ```bnf
  /// TUPLE_LITERAL ::= "(" ARR_TPL_BODY? ")"
  /// ARR_TPL_BODY  ::= ARR_TPL_LIST | ARR_TPL_REPEAT | COMPACT_FOR_LOOP
  /// ```
  pub(super) fn parse_tuple_literal_or_grouping_expr(&mut self) -> NodeResult<ASTNodeIdx> {
    let token = self.current_pos - 1;

    // If we match a for-keyword at the start the tuple literal, then we
    // know we have a tuple comprehension so we can go ahead and parse it.
    if match_tok![self, FOR_KW] {
      let expr = self.parse_compact_arr_or_tpl(false)?;
      self.consume(&R_PAREN, "Expected matching ')' for tuple literal.", None)?;
      return Ok(expr);
    }

    // Initialize the list of values for tuple literal
    let mut values: Vec<ASTNodeIdx> = vec![];

    // If we *do* match a right parenthesis immediately, then we can
    // skip all this code and return an empty tuple like in Python.
    if !match_tok![self, R_PAREN] {
      // Get the first value of the array
      values.push(match curr_tk![self] {
        TRIPLE_DOT if self.advance() => self.parse_single_spread_expr()?,
        _ => {
          let value = self.parse_expr()?;

          // If there is a semicolon after the first value, then
          // we know this must be an array-repeat expression.
          if match_tok![self, SEMICOLON] {
            return self.parse_repeat_arr_or_tpl(token, value, true);
          }

          // If we do not find a comma after the first expression,
          // then we are safe to assume this is a grouping expression.
          if !check_tok![self, COMMA] {
            self.consume(&R_PAREN, "Expected matching ')' for grouped expression.", None)?;
            return Ok(value);
          }

          value
        }
      });

      // Parse the rest of the tuple's body
      values.append(&mut self.parse_array_or_tuple_list(true)?);
      self.consume(&R_PAREN, "Expected matching ')' for tuple literal.", None)?;
    }

    let node = ASTTupleLiteralNode { token, values };
    self.emit(TupleLiteral(node))
  }

  /// Parses either a single spread expression
  ///
  /// ```bnf
  /// SINGLE_SPREAD_EXPR ::= "..." EXPRESSION
  /// ```
  pub(super) fn parse_single_spread_expr(&mut self) -> NodeResult<ASTNodeIdx> {
    let expr = self.parse_expr()?;
    self.emit(SpreadExpr(expr))
  }

  /// Parses either an array or tuple repeat.
  ///
  /// ```bnf
  /// ARR_TPL_REPEAT ::= EXPRESSION ";" EXPRESSION
  /// ```
  pub(super) fn parse_repeat_arr_or_tpl(
    &mut self,
    token: TokenIdx,
    value: ASTNodeIdx,
    is_tup: bool,
  ) -> NodeResult<ASTNodeIdx> {
    let count = self.parse_expr()?;

    let kind = if is_tup {
      self.consume(&R_PAREN, "Expected matching ')' for repeated tuple literal.", None)?;
      RepeatLiteralKind::Tuple
    } else {
      self.consume(&R_BRACKET, "Expected matching ']' for repeated array literal.", None)?;
      RepeatLiteralKind::Array
    };

    let node = ASTRepeatLiteralNode { token, kind, value, count };
    self.emit(RepeatLiteral(node))
  }

  /// Parses the expression list in an array or tuple body.
  ///
  /// ```bnf
  /// ARR_TPL_LIST ::= (EXPRESSION | SINGLE_SPREAD_EXPR) ("," (EXPRESSION | SINGLE_SPREAD_EXPR))*
  /// ```
  pub(super) fn parse_array_or_tuple_list(&mut self, is_tpl: bool) -> NodeResult<Vec<ASTNodeIdx>> {
    let mut values: Vec<ASTNodeIdx> = vec![];

    while match_tok![self, COMMA] {
      let val = match curr_tk![self] {
        R_PAREN if is_tpl => break,
        R_BRACKET if !is_tpl => break,
        TRIPLE_DOT if self.advance() => self.parse_single_spread_expr()?,
        _ => self.parse_expr()?,
      };

      values.push(val);
    }

    Ok(values)
  }

  /// Parses a compact array or tuple.
  ///
  /// ```bnf
  /// COMPACT_ARR_TPL ::= COMPACT_FOR_LOOP+ (EXPRESSION | SINGLE_SPREAD_EXPR)
  /// ```
  pub(super) fn parse_compact_arr_or_tpl(&mut self, is_tpl: bool) -> NodeResult<ASTNodeIdx> {
    let mut heads = vec![];

    loop {
      heads.push(self.parse_compact_for_loop()?);

      if !match_tok![self, FOR_KW] {
        break;
      }
    }

    let body = match curr_tk![self] {
      TRIPLE_DOT if self.advance() => self.parse_single_spread_expr()?,
      _ => self.parse_expr()?,
    };

    self.emit(CompactArrOrTpl(ASTCompactArrOrTplNode { heads, body, is_tpl }))
  }

  /// Parses a compact for-loop for an array, tuple, or dict comprehension
  ///
  /// ```bnf
  /// COMPACT_FOR_LOOP ::= "for" "(" FOR_LOOP_HEAD ")" ("if" "(" EXPRESSION ")")?
  /// ```
  pub(super) fn parse_compact_for_loop(&mut self) -> NodeResult<CompactForLoop> {
    self.consume(&L_PAREN, "Expected '(' after 'for' keyword in compact for-loop.", None)?;
    let head = self.parse_for_loop_head()?;
    self.consume(&R_PAREN, "Expected ')' after loop head in compact for-loop.", None)?;

    let mut cond = None;
    if match_tok![self, IF_KW] {
      self.consume(&L_PAREN, "Expected '(' after 'if' keyword in compact for-loop.", None)?;
      cond = Some(self.parse_expr()?);
      self.consume(&R_PAREN, "Expected ')' after 'if' head in compact for-loop.", None)?;
    }

    Ok(CompactForLoop { head, cond })
  }

  /// Parses a dict literal expression.
  ///
  /// ```bnf
  /// DICT_LITERAL  ::= "{" DICT_BODY? "}"
  /// DICT_BODY     ::= (KEY_VAL_PAR | SINGLE_SPREAD_EXPR) ("," (KEY_VAL_PAR | SINGLE_SPREAD_EXPR))*
  ///               | COMPACT_DICT_LOOP
  /// ```
  pub(super) fn parse_dict_literal(&mut self) -> NodeResult<ASTNodeIdx> {
    // If we match a for-keyword at the start the dict literal, then we
    // know we have a dict comprehension so we can go ahead and parse it.
    if match_tok![self, FOR_KW] {
      let expr = self.parse_compact_dict()?;
      self.consume(&R_CURLY, "Expected matching '}' for dict literal.", None)?;
      return Ok(expr);
    }

    // Initialize the list of key-val-pairs for dict literal
    let mut key_val_pairs: Vec<ASTNodeIdx> = vec![];

    if !match_tok![self, R_CURLY] {
      key_val_pairs.push(self.parse_dict_key_val_pair()?);

      while match_tok![self, COMMA] {
        if let R_CURLY = curr_tk![self] {
          break;
        } else {
          key_val_pairs.push(self.parse_dict_key_val_pair()?);
        }
      }

      self.consume(&R_CURLY, "Expected matching '}' for dict literal.", None)?;
    }

    self.emit(DictLiteral(key_val_pairs))
  }

  /// Parses a compact dict.
  ///
  /// ```bnf
  /// COMPACT_DICT ::= COMPACT_FOR_LOOP+ (KEY_VAL_PAR | SINGLE_SPREAD_EXPR)
  /// ```
  pub(super) fn parse_compact_dict(&mut self) -> NodeResult<ASTNodeIdx> {
    let mut heads = vec![];

    loop {
      heads.push(self.parse_compact_for_loop()?);

      if !match_tok![self, FOR_KW] {
        break;
      }
    }

    let body = match curr_tk![self] {
      TRIPLE_DOT if self.advance() => self.parse_single_spread_expr()?,
      _ => self.parse_dict_key_val_pair()?,
    };

    self.emit(CompactDict(ASTCompactDictNode { heads, body }))
  }

  /// Parses a single key-value pair for a dict literal.
  ///
  /// ```bnf
  /// KEY_VAL_PAR ::= (("[" EXPRESSION "]") | IDENTIFIER | STRING_LITERAL | INTEGER_LITERAL
  ///             | HEX_LITERAL | OCT_LITERAL | BINARY_LITERAL | TUPLE_LITERAL |) ":" EXPRESSION
  /// ```
  pub(super) fn parse_dict_key_val_pair(&mut self) -> NodeResult<ASTNodeIdx> {
    // If we find the spread operator, then simply return a spread expression.
    if match_tok![self, TRIPLE_DOT] {
      return self.parse_single_spread_expr();
    }

    let ky = if match_tok![self, L_PAREN] {
      self.parse_tuple_literal_or_grouping_expr()?
    } else {
      let literal_or_ident = match curr_tk![self] {
        IDENTIFIER if self.advance() => IdLiteral(self.current_pos - 1),
        STR_LIT if self.advance() => StringLiteral(self.current_pos - 1),
        INT_LIT | HEX_LIT | OCTAL_LIT | BINARY_LIT if self.advance() => NumLiteral(self.current_pos - 1),
        L_BRACKET if self.advance() => {
          let expr = self.parse_expr()?;
          self.consume(&R_BRACKET, "Expected ']' for evaluated dict key name.", None)?;
          EvaluatedDictKey(expr)
        }
        _ => return Err(self.error_at_current_tok("Invalid key for dict literal.", None)),
      };

      self.ast.push(literal_or_ident)
    };

    self.consume(&COLON, "Expected ':' in dict key-value pair.", None)?;
    let val = self.parse_expr()?;
    self.emit(DictKeyValPair((ky, val)))
  }
}
