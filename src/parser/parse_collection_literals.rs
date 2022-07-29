use crate::errors::ErrorReport;
use crate::lexer::tokens::TokenKind::*;
use crate::parser::ast::ASTNodeKind::*;
use crate::parser::ast::*;
use crate::parser::Parser;
use crate::{check_tok, curr_tk, match_tok};

impl<'a> Parser<'a> {
  /// Parses an array literal expression.
  ///
  /// ```bnf
  /// ARRAY_LITERAL ::= "[" ARR_TPL_BODY? "]"
  /// ARR_TPL_BODY  ::= ARR_TPL_LIST | ARR_TPL_REPEAT | COMPACT_FOR_LOOP
  /// ```
  pub(super) fn parse_array_literal(&mut self) -> Result<ASTNodeIdx, ErrorReport> {
    // If we match a for-keyword at the start the array literal, then we
    // know we have an array comprehension so we can go ahead and parse it.
    if match_tok![self, FOR_KW] {
      todo!("Implement array comprehension");
      // self.consume(&R_BRACKET, "Expected matching ']' for array literal.")?;
    }

    // Initialize the list of values for array literal
    let mut values: Vec<ASTNodeIdx> = vec![];

    if !match_tok![self, R_BRACKET] {
      // Get the first value of the array
      values.push(match curr_tk![self] {
        SPREAD if self.advance() => self.parse_single_spread_expr()?,
        _ => {
          let value = self.parse_expr()?;

          // If there is a semicolon after the first value, then
          // we know this must be an array-repeat expression.
          if match_tok![self, SEMICOLON] {
            return self.parse_repeat_arr_or_tpl(value, false);
          }

          value
        }
      });

      // Parse the rest of the array's body
      values.append(&mut self.parse_array_or_tuple_list(false)?);
      self.consume(&R_BRACKET, "Expected matching ']' for array literal.")?;
    }

    Ok(self.ast.append(ArrayLiteral(values)))
  }

  /// Parses either a tuple literal expression or a grouping expression.
  ///
  /// ```bnf
  /// TUPLE_LITERAL ::= "(" ARR_TPL_BODY? ")"
  /// ARR_TPL_BODY  ::= ARR_TPL_LIST | ARR_TPL_REPEAT | COMPACT_FOR_LOOP
  /// ```
  pub(super) fn parse_tuple_literal_or_grouping_expr(&mut self) -> Result<ASTNodeIdx, ErrorReport> {
    // If we match a for-keyword at the start the tuple literal, then we
    // know we have a tuple comprehension so we can go ahead and parse it.
    if match_tok![self, FOR_KW] {
      todo!("Implement tuple comprehension");
      // self.consume(&R_PAREN, "Expected matching ')' for tuple literal.")?;
    }

    // Initialize the list of values for tuple literal
    let mut values: Vec<ASTNodeIdx> = vec![];

    // If we *do* match a right parenthesis immediately, then we can
    // skip all this code and return an empty tuple like in Python.
    if !match_tok![self, R_PAREN] {
      // Get the first value of the array
      values.push(match curr_tk![self] {
        SPREAD if self.advance() => self.parse_single_spread_expr()?,
        _ => {
          let value = self.parse_expr()?;

          // If there is a semicolon after the first value, then
          // we know this must be an array-repeat expression.
          if match_tok![self, SEMICOLON] {
            return self.parse_repeat_arr_or_tpl(value, true);
          }

          // If we do not find a comma after the first expression,
          // then we are safe to assume this is a grouping expression.
          if !check_tok![self, COMMA] {
            self.consume(&R_PAREN, "Expected matching ')' for grouped expression.")?;
            return Ok(value);
          }

          value
        }
      });

      // Parse the rest of the tuple's body
      values.append(&mut self.parse_array_or_tuple_list(true)?);
      self.consume(&R_PAREN, "Expected matching ')' for tuple literal.")?;
    }

    Ok(self.ast.append(TupleLiteral(values)))
  }

  /// Parses either a single spread expression
  ///
  /// ```bnf
  /// SINGLE_SPREAD_EXPR ::= "..." EXPRESSION
  /// ```
  pub(super) fn parse_single_spread_expr(&mut self) -> Result<ASTNodeIdx, ErrorReport> {
    let expr = self.parse_expr()?;
    Ok(self.ast.append(SpreadExpr(expr)))
  }

  /// Parses either an array or tuple repeat.
  ///
  /// ```bnf
  /// ARR_TPL_REPEAT ::= EXPRESSION ";" EXPRESSION
  /// ```
  pub(super) fn parse_repeat_arr_or_tpl(&mut self, value: ASTNodeIdx, is_tup: bool) -> Result<ASTNodeIdx, ErrorReport> {
    let count = self.parse_expr()?;

    let kind = if is_tup {
      self.consume(&R_PAREN, "Expected matching ')' for repeated tuple literal.")?;
      RepeatLiteralKind::Tuple
    } else {
      self.consume(&R_BRACKET, "Expected matching ']' for repeated array literal.")?;
      RepeatLiteralKind::Array
    };

    let node = ASTRepeatLiteralNode { kind, value, count };
    Ok(self.ast.append(RepeatLiteral(node)))
  }

  /// Parses the expression list in an array or tuple body.
  ///
  /// ```bnf
  /// ARR_TPL_LIST ::= (EXPRESSION | SINGLE_SPREAD_EXPR) ("," (EXPRESSION | SINGLE_SPREAD_EXPR))*
  /// ```
  pub(super) fn parse_array_or_tuple_list(&mut self, is_tpl: bool) -> Result<Vec<ASTNodeIdx>, ErrorReport> {
    let mut values: Vec<ASTNodeIdx> = vec![];

    while match_tok![self, COMMA] {
      let val = match curr_tk![self] {
        R_PAREN if is_tpl => break,
        R_BRACKET if !is_tpl => break,
        SPREAD if self.advance() => self.parse_single_spread_expr()?,
        _ => self.parse_expr()?,
      };

      values.push(val);
    }

    Ok(values)
  }

  /// Parses a dict literal expression.
  ///
  /// ```bnf
  /// DICT_LITERAL  ::= "{" DICT_BODY? "}"
  /// DICT_BODY     ::= (KEY_VAL_PAR | SINGLE_SPREAD_EXPR) ("," (KEY_VAL_PAR | SINGLE_SPREAD_EXPR))*
  ///               | COMPACT_DICT_LOOP
  /// ```
  pub(super) fn parse_dict_literal(&mut self) -> Result<ASTNodeIdx, ErrorReport> {
    // If we match a for-keyword at the start the dict literal, then we
    // know we have a dict comprehension so we can go ahead and parse it.
    if match_tok![self, FOR_KW] {
      todo!("Implement dict comprehension");
      // self.consume(&R_CURLY, "Expected matching '}' for dict literal.")?;
    }

    // Initialize the list of key-val-pairs for dict literal
    let mut key_val_pairs: Vec<ASTNodeIdx> = vec![];

    if !match_tok![self, R_CURLY] {
      loop {
        key_val_pairs.push(self.parse_dict_key_val_pair()?);

        if !match_tok![self, COMMA] {
          break;
        }
      }

      self.consume(&R_CURLY, "Expected matching '}' for dict literal.")?;
    }

    Ok(self.ast.append(DictLiteral(key_val_pairs)))
  }

  /// Parses a single key-value pair for a dict literal.
  ///
  /// ```bnf
  /// KEY_VAL_PAR ::= (IDENTIFIER | STRING_LITERAL | INTEGER_LITERAL | HEX_LITERAL | OCT_LITERAL | BINARY_LITERAL | TUPLE_LITERAL) ":" EXPRESSION
  /// ```
  pub(super) fn parse_dict_key_val_pair(&mut self) -> Result<ASTNodeIdx, ErrorReport> {
    // If we find the spread operator, then simply return a spread expression.
    if match_tok![self, SPREAD] {
      return self.parse_single_spread_expr();
    }

    let ky = if match_tok![self, L_PAREN] {
      self.parse_tuple_literal_or_grouping_expr()?
    } else {
      let literal_or_ident = match curr_tk![self] {
        IDENTIFIER if self.advance() => Identifier(self.current_pos),
        STR_LIT if self.advance() => StringLiteral(self.current_pos),
        INT_LIT if self.advance() => Literal(ASTLiteralNode {
          value: self.parse_integer()?,
          token_idx: self.current_pos,
        }),
        HEX_LIT if self.advance() => Literal(ASTLiteralNode {
          value: self.parse_int_from_base(16)?,
          token_idx: self.current_pos,
        }),
        OCTAL_LIT if self.advance() => Literal(ASTLiteralNode {
          value: self.parse_int_from_base(8)?,
          token_idx: self.current_pos,
        }),
        BINARY_LIT if self.advance() => Literal(ASTLiteralNode {
          value: self.parse_int_from_base(2)?,
          token_idx: self.current_pos,
        }),
        _ => return Err(self.error_at_current("Invalid key for dict literal.")),
      };

      self.ast.append(literal_or_ident)
    };

    self.consume(&COLON, "Expected colon after dictionary literal.")?;
    let val = self.parse_expr()?;
    Ok(self.ast.append(DictKeyValPair((ky, val))))
  }
}
