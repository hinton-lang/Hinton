use crate::{check_tok, consume_id, consume_id_list, curr_tk, match_tok, NodeResult, Parser};
use core::ast::ASTNodeKind::*;
use core::ast::*;
use core::tokens::TokenKind::*;

impl<'a> Parser<'a> {
  /// Parses a class declaration.
  ///
  /// ```bnf
  /// CLASS_DECL  ::= "abstract"? "class" IDENTIFIER CLS_PARAMS? CLS_EXTEND? CLS_IMPL? "{" CLS_MEMBER* "}"
  /// CLS_EXTEND  ::= "->" IDENTIFIER_LIST
  /// CLS_IMPL    ::= "impl" IDENTIFIER_LIST
  /// ```
  pub(super) fn parse_class_decl(&mut self, is_abstract: bool, decor: Vec<Decorator>) -> NodeResult<ASTNodeIdx> {
    if is_abstract {
      let err_msg = "Expected 'class' keyword for abstract class declaration.";
      self.consume(&CLASS_KW, err_msg, None)?;
    }

    // Parse the class name
    let name = consume_id![self, "Expected identifier for class name.", None];

    // Parse class parameters
    let (min_arity, max_arity, params) = if match_tok![self, L_PAREN] {
      self.parse_class_params()?
    } else {
      (0, 0, Vec::with_capacity(0))
    };

    // Parse class extensions
    let extends = if match_tok![self, THIN_ARROW] {
      consume_id_list![self, "Expected identifier for class extend.", None]
    } else {
      vec![]
    };

    // Parse class implementations
    let impls = if match_tok![self, IMPL_KW] {
      consume_id_list![self, "Expected identifier for class extend.", None]
    } else {
      vec![]
    };

    self.consume(&L_CURLY, "Expected '{' at start of class body.", None)?;

    let mut init = None;
    let mut members = vec![];

    // Parse the class's body
    while !match_tok![self, R_CURLY] {
      match curr_tk![self] {
        INIT_KW if self.advance() => {
          if init.is_some() {
            return Err(self.error_at_prev_tok("Can only have one 'init' block per class.", None));
          }
          self.consume(&L_CURLY, "Expected block as 'init' body.", None)?;
          init = Some(self.parse_block_stmt()?)
        }
        _ => members.push(self.parse_class_member()?),
      }
    }

    let class_idx = self.ast.push_class(ASTClassDeclNode {
      decor,
      is_abstract,
      name,
      params,
      min_arity,
      max_arity,
      extends,
      impls,
      init,
      members,
    });

    self.emit(ClassDecl(class_idx))
  }

  /// Parses a class parameter.
  ///
  /// ```bnf
  /// CLS_PARAMS          ::= "(" CLS_PARAM_ID_LIST? CLS_NON_REQ_PARAMS? REST_PARAM? ")"
  /// CLS_PARAM_ID_LIST   ::= CLS_PARAM_MODE? IDENTIFIER ("," CLS_PARAM_MODE? IDENTIFIER)*
  /// CLS_NON_REQ_PARAMS  ::= CLS_PARAM_MODE? IDENTIFIER NON_REQ_BODY ("," CLS_PARAM_MODE? IDENTIFIER NON_REQ_BODY)*
  /// CLS_PARAM_MODE      ::= DECORATOR_STMT* "pub"? "const"?
  /// ```
  pub(super) fn parse_class_params(&mut self) -> NodeResult<(u8, u8, Vec<ClassParam>)> {
    let mut params: Vec<ClassParam> = vec![];
    let mut min_arity: u8 = 0;
    let mut has_rest_param = false;

    while !check_tok![self, R_PAREN] {
      if params.len() >= 255 {
        return Err(self.error_at_current_tok("Can't have more than 255 parameters.", None));
      }

      let decor = if match_tok![self, HASHTAG] {
        self.parse_decorator_stmt()?
      } else {
        Vec::with_capacity(0)
      };

      let is_pub = match_tok![self, PUB_KW];
      let is_const = match_tok![self, CONST_KW];

      let param = self.parse_single_param(&params)?;
      has_rest_param = has_rest_param || matches![param.kind, SingleParamKind::Rest];
      min_arity += matches![param.kind, SingleParamKind::Required] as u8;
      params.push(ClassParam { decor, is_pub, is_const, param });

      // Optional trailing comma
      if !check_tok![self, COMMA] || (match_tok![self, COMMA] && check_tok![self, R_PAREN]) {
        break;
      }
    }

    self.consume(&R_PAREN, "Expected ')' after class parameter list.", None)?;
    let max_arity = if has_rest_param { 255 } else { params.len() as u8 };
    Ok((min_arity, max_arity, params))
  }

  /// Parses a class member, except the init block.
  ///
  /// ```bnf
  /// CLS_MEMBER  ::= DECORATOR_STMT* "pub"? "override"? "static"? (VAR_DECL | CONST_DECL | FUNC_DECL)
  ///             | "init" BLOCK_STMT // only one init block per class
  /// ```
  fn parse_class_member(&mut self) -> NodeResult<ClassMember> {
    let decorators = if match_tok![self, HASHTAG] {
      self.parse_decorator_stmt()?
    } else {
      Vec::with_capacity(0)
    };

    let mode = self.parse_class_member_mode()?;

    let member = match curr_tk![self] {
      LET_KW if self.advance() => ClassMemberKind::Var(decorators, self.parse_var_or_const_decl(false)?),
      CONST_KW if self.advance() => ClassMemberKind::Const(decorators, self.parse_var_or_const_decl(true)?),
      FUNC_KW if self.advance() => ClassMemberKind::Func(self.parse_func_stmt(false, decorators)?),
      ASYNC_KW if self.advance() => ClassMemberKind::Func(self.parse_func_stmt(true, decorators)?),
      _ => return Err(self.error_at_current_tok("Expected 'let', 'const', or 'func' declaration.", None)),
    };

    Ok(ClassMember { mode, member })
  }

  /// Parses the mode of a class member.
  ///
  /// ```bnf
  /// CLS_MEMBER  ::= DECORATOR_STMT* "pub"? "override"? "static"? (VAR_DECL | CONST_DECL | FUNC_DECL)
  ///             | "init" BLOCK_STMT // only one init block per class
  /// ```
  fn parse_class_member_mode(&mut self) -> NodeResult<ClassMemberMode> {
    let mut is_public = false;
    let mut is_override = false;
    let mut is_static = false;

    loop {
      match curr_tk![self] {
        PUB_KW if self.advance() => {
          if is_public {
            return Err(self.error_at_prev_tok("Class member already marked as public.", None));
          } else if is_override {
            return Err(self.error_at_prev_tok("Keyword 'pub' must precede the 'override' keyword.", None));
          } else if is_static {
            return Err(self.error_at_prev_tok("Keyword 'pub' must precede the 'static' keyword.", None));
          } else {
            is_public = true
          }
        }
        OVERRIDE_KW if self.advance() => {
          if is_override {
            return Err(self.error_at_prev_tok("Class member already marked as override.", None));
          } else if is_static {
            return Err(self.error_at_prev_tok("Keyword 'override' must precede the 'static' keyword.", None));
          } else {
            is_override = true
          }
        }
        STATIC_KW if self.advance() => {
          if is_static {
            return Err(self.error_at_prev_tok("Class member already marked as static.", None));
          } else {
            is_static = true
          }
        }
        _ => break,
      };
    }

    Ok(ClassMemberMode {
      is_public,
      is_override,
      is_static,
    })
  }
}
