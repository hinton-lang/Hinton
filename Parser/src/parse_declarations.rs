use core::ast::ASTNodeKind::*;
use core::ast::*;
use core::tokens::TokenKind::*;

use crate::{check_tok, consume_id, curr_tk, match_tok, NodeResult, Parser};

impl<'a> Parser<'a> {
  /// Parses a variable or constant declaration statement.
  ///
  /// ```bnf
  /// VAR_CONST_DECL ::= ("let" | "const") (IDENTIFIER | DESTRUCT_PATTERN) "=" EXPRESSION ";"
  /// ```
  pub(super) fn parse_var_or_const_decl(&mut self, is_const: bool) -> NodeResult<ASTNodeIdx> {
    let decl_name = if is_const { "const" } else { "let" };

    let id = if match_tok![self, L_PAREN] {
      self.parse_destructing_pattern(&format!("'{}' declaration", decl_name))?
    } else {
      let err_msg = &format!("Expected identifier for '{}' declaration.", decl_name);
      consume_id![self, err_msg]?
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
  pub(super) fn parse_destructing_pattern(&mut self, msg: &str) -> NodeResult<ASTNodeIdx> {
    let mut patterns = vec![];
    let mut has_rest = false;

    loop {
      let pattern = match curr_tk![self] {
        TRIPLE_DOT if has_rest => {
          return Err(self.error_at_current("Can only have one wildcard expression in destructing pattern."));
        }
        TRIPLE_DOT if self.advance() => {
          has_rest = true;

          let node = if match_tok![self, IDENTIFIER] {
            Some(ASTNodeIdx::from(self.current_pos - 1))
          } else {
            None
          };

          self.emit(DestructingWildCard(node))?
        }
        _ => {
          let err_msg = &format!("Expected identifier for destructing pattern in {}.", msg);
          consume_id![self, err_msg]?
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

  /// Parses a function declaration statement.
  ///
  /// ```bnf
  /// FUNC_DECL ::= "async"? "func" "*"? IDENTIFIER "(" PARAMETERS ")" BLOCK_STMT
  /// ```
  pub(super) fn parse_func_stmt(&mut self, is_async: bool, decor: Vec<Decorator>) -> NodeResult<ASTNodeIdx> {
    if is_async {
      self.consume(&FUNC_KW, "Expected 'func' keyword for async function declaration.")?;
    }

    let is_gen = match_tok![self, STAR];
    let name = consume_id![self, "Expected identifier for function name."]?;

    self.consume(&L_PAREN, "Expected '(' after function name.")?;
    let (min_arity, max_arity, params) = self.parse_func_params(false)?;
    self.consume(&R_PAREN, "Expected ')' after function parameter list.")?;

    self.consume(&L_CURLY, "Expected '{' for the function body.")?;
    let body = self.parse_block_stmt()?;

    self.emit(FuncDecl(ASTFuncDeclNode {
      decor,
      is_async,
      is_gen,
      name,
      params,
      min_arity,
      max_arity,
      body,
    }))
  }

  /// Parses a parameter list for a function declaration statement or lambda expression.
  ///
  /// ```bnf
  /// PARAMETERS      ::= IDENTIFIER_LIST? NON_REQ_PARAMS? REST_PARAM?
  /// NON_REQ_PARAMS  ::= IDENTIFIER NON_REQ_BODY ("," IDENTIFIER NON_REQ_BODY)*
  /// NON_REQ_BODY    ::= "?" | (":=" EXPRESSION)
  /// ```
  pub(super) fn parse_func_params(&mut self, is_lambda: bool) -> NodeResult<(u8, u8, Vec<SingleParam>)> {
    let closing_tok = if is_lambda { VERT_BAR } else { R_PAREN };
    let mut params: Vec<SingleParam> = vec![];
    let mut min_arity: u8 = 0;
    let mut has_rest_param = false;

    while !check_tok![self, closing_tok] {
      if params.len() >= 255 {
        return Err(self.error_at_current("Can't have more than 255 parameters."));
      }

      let param = self.parse_single_param(&params)?;
      has_rest_param = has_rest_param || matches![param.kind, SingleParamKind::Rest];
      min_arity += matches![param.kind, SingleParamKind::Required] as u8;
      params.push(param);

      // Optional trailing comma
      if !check_tok![self, COMMA] || (match_tok![self, COMMA] && check_tok![self, closing_tok]) {
        break;
      }
    }

    let max_arity = if has_rest_param { 255 } else { params.len() as u8 };
    Ok((min_arity, max_arity, params))
  }

  /// Parses a single parameter (whether required, optional, named, or rest).
  ///
  /// # Arguments
  ///
  /// * `params`: A list of previously parsed parameters.
  ///
  /// # Returns:
  /// ```Result<FuncParam, ErrorReport>```
  pub(super) fn parse_single_param<P>(&mut self, params: &[P]) -> NodeResult<SingleParam>
  where
    P: SingleParamLike,
  {
    let is_spread = match_tok![self, TRIPLE_DOT];
    let name = consume_id![self, "Expected a parameter name."]?;

    let param = if is_spread {
      let kind = SingleParamKind::Rest;
      SingleParam { name, kind }
    } else {
      let kind = match curr_tk![self] {
        QUESTION if self.advance() => SingleParamKind::Optional,
        COLON_EQUALS if self.advance() => SingleParamKind::Named(self.parse_expr()?),
        _ => SingleParamKind::Required,
      };

      SingleParam { name, kind }
    };

    // TODO: Should use node span resolution, and emit errors at the appropriate nodes.
    if !params.is_empty() {
      let prev_kind = &params.last().unwrap().get_kind();

      if let SingleParamKind::Rest = prev_kind {
        return Err(self.error_at_prev("Rest parameter must be last in parameter list."));
      }

      if let SingleParamKind::Required = param.kind {
        if !matches![prev_kind, SingleParamKind::Required] {
          return Err(self.error_at_current("Required parameters cannot follow optional or named parameters."));
        }
      }
    }

    Ok(param)
  }
}
