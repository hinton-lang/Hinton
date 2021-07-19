use crate::core::ast::ASTNode;
use crate::core::ast::ASTNode::*;
use crate::core::ast::*;
use crate::core::tokens::TokenType::*;
use crate::objects::Object;
use crate::parser::Parser;

impl Parser {
   /// Parses a declaration.
   pub(super) fn parse_declaration(&mut self) -> Option<ASTNode> {
      let decl = if self.matches(&VAR_KW) {
         self.parse_var_declaration().map(VariableDecl)
      } else if self.matches(&CONST_KW) {
         self.parse_const_declaration().map(ConstantDecl)
      } else if self.matches(&FUNC_KW) {
         self.parse_func_declaration().map(FunctionDecl)
      } else if self.matches(&CLASS_KW) {
         self.parse_class_declaration()
      } else {
         self.parse_statement()
      };

      if self.is_in_panic {
         self.synchronize();
      }

      decl
   }

   /// Parses a statement.
   fn parse_statement(&mut self) -> Option<ASTNode> {
      if self.matches(&L_CURLY) {
         self.parse_block()
      } else if self.matches(&IF_KW) {
         self.parse_if_statement()
      } else if self.matches(&WHILE_KW) {
         self.parse_while_statement()
      } else if self.matches(&FOR_KW) {
         self.parse_for_statement()
      } else if self.matches(&BREAK_KW) {
         let tok = self.previous.clone();
         self.consume(&SEMICOLON, "Expected a ';' after the 'break' keyword.")?;

         Some(LoopBranch(LoopBranchStmtNode {
            token: tok,
            is_break: true,
         }))
      } else if self.matches(&CONTINUE_KW) {
         let tok = self.previous.clone();
         self.consume(&SEMICOLON, "Expected a ';' after the 'continue' keyword.")?;

         Some(LoopBranch(LoopBranchStmtNode {
            token: tok,
            is_break: false,
         }))
      } else if self.matches(&RETURN_KW) {
         self.parse_return_stmt()
      } else {
         self.parse_expression_statement()
      }
   }

   /// Parses an expression statement.
   fn parse_expression_statement(&mut self) -> Option<ASTNode> {
      let opr = self.previous.clone();
      let expr = Box::new(self.parse_expression()?);

      self.consume(&SEMICOLON, "Expected a ';' after the expression.")?;

      Some(ExpressionStmt(ExpressionStmtNode {
         child: expr,
         pos: (opr.line_num, opr.column_start),
      }))
   }

   /// Parses a block statement.
   pub(super) fn parse_block(&mut self) -> Option<ASTNode> {
      let mut body: Vec<ASTNode> = vec![];

      while !self.check(&R_CURLY) && !self.check(&EOF) {
         body.push(self.parse_declaration()?);
      }

      self.consume(&R_CURLY, "Expected a matching '}' for the block statement.");

      Some(BlockStmt(BlockNode {
         body: body.into_boxed_slice(),
         end_of_block: self.previous.clone(),
      }))
   }

   /// Parses a variable declaration.
   fn parse_var_declaration(&mut self) -> Option<VariableDeclNode> {
      // Gets at least one variable name, or a list of names separated by a comma
      let mut declarations = vec![self.consume(&IDENTIFIER, "Expected variable name.")?];
      while self.matches(&COMMA) {
         declarations.push(self.consume(&IDENTIFIER, "Expected variable name.")?);
      }

      // Gets the variable's value.
      let initializer = if self.matches(&EQUALS) {
         self.parse_expression()?
      } else {
         ASTNode::Literal(LiteralExprNode {
            value: Object::Null,
            token: self.previous.clone(),
         })
      };

      // Requires a semicolon at the end of the declaration if the declaration
      // was not a block (e.g., when assigning a lambda function to a variable).
      if !self.previous.token_type.type_match(&R_CURLY) {
         self.consume(&SEMICOLON, "Expected a ';' after the variable declaration.")?;
      }

      // However, if there is a semicolon after a curly brace, then we consume it
      if self.previous.token_type.type_match(&R_CURLY) && self.check(&SEMICOLON) {
         self.advance();
      }

      Some(VariableDeclNode {
         identifiers: declarations.into_boxed_slice(),
         value: Box::new(initializer),
      })
   }

   /// Parses a constant declaration.
   fn parse_const_declaration(&mut self) -> Option<ConstantDeclNode> {
      let name = self.consume(&IDENTIFIER, "Expected a name for the constant declaration.")?;
      self.consume(&EQUALS, "Constants must be initialized upon declaration.")?;
      let initializer = self.parse_expression()?;

      // Requires a semicolon at the end of the declaration if the declaration
      // was not a block (e.g., when assigning a lambda function to a constant).
      if !self.previous.token_type.type_match(&R_CURLY) {
         self.consume(&SEMICOLON, "Expected a ';' after the constant declaration.")?;
      }

      // However, if there is a semicolon after a curly brace, then we consume it
      if self.previous.token_type.type_match(&R_CURLY) && self.check(&SEMICOLON) {
         self.advance();
      }

      Some(ConstantDeclNode {
         name,
         value: Box::new(initializer),
      })
   }

   /// Parses an `if` statement.
   fn parse_if_statement(&mut self) -> Option<ASTNode> {
      let then_tok = self.previous.clone();
      let condition = self.parse_expression()?;

      let then_branch = if let R_PAREN = self.previous.token_type {
         self.parse_statement()?
      } else {
         self.consume(&L_CURLY, "Expected '{' after the 'if' condition.")?;
         self.parse_block()?
      };

      let mut else_branch = None;
      let mut else_tok = None;

      if self.matches(&ELSE_KW) {
         else_tok = Some(self.previous.clone());
         else_branch = Some(self.parse_statement()?);
      }

      Some(IfStmt(IfStmtNode {
         condition: Box::new(condition),
         then_token: then_tok,
         then_branch: Box::new(then_branch),
         else_branch: Box::new(else_branch),
         else_token: else_tok,
      }))
   }

   /// Parses a `while` statement.
   fn parse_while_statement(&mut self) -> Option<ASTNode> {
      let tok = self.previous.clone();
      let condition = self.parse_expression()?;

      let body = if let R_PAREN = self.previous.token_type {
         self.parse_statement()?
      } else {
         self.consume(&L_CURLY, "Expected '{' after the 'while' condition.");
         self.parse_block()?
      };

      Some(WhileStmt(WhileStmtNode {
         token: tok,
         condition: Box::new(condition),
         body: Box::new(body),
      }))
   }

   /// Parses a `for-in` statement.
   fn parse_for_statement(&mut self) -> Option<ASTNode> {
      let token = self.previous.clone();
      let has_parenthesis = self.matches(&L_PAREN);

      // For-loops must have either the `var` or `await` keyword before the loop's variable, but
      // not both. Here, in the future, we would check which keyword it is and define the type
      // of for-loop we are parsing based on which keyword is present.
      self.consume(&VAR_KW, "Expected the 'var' keyword before the identifier.");

      let id = if let ASTNode::Identifier(id) = self.parse_primary()? {
         id
      } else {
         self.error_at_current("Expected an identifier name.");
         return None;
      };

      self.consume(&IN_KW, "Expected the 'in' keyword after the identifier.");

      let iterator = Box::new(self.parse_expression()?);

      let body = if has_parenthesis {
         self.consume(&R_PAREN, "Expected a matching ')' after the 'for-in' iterator.");

         match self.parse_statement()? {
            ASTNode::BlockStmt(block) => block.body,
            _any => vec![_any].into_boxed_slice(),
         }
      } else {
         self.consume(&L_CURLY, "Expected '{' after the 'for-in' iterator.");

         match self.parse_block()? {
            ASTNode::BlockStmt(block) => block.body,
            _ => unreachable!("Should have parsed a block."),
         }
      };

      Some(ForStmt(ForStmtNode {
         token,
         id,
         iterator,
         body,
      }))
   }

   /// Parses a function declaration.
   fn parse_func_declaration(&mut self) -> Option<FunctionDeclNode> {
      let name = self.consume(
         &IDENTIFIER,
         "Expected an identifier for the function declaration.",
      )?;

      self.consume(&L_PAREN, "Expected '(' after function name.")?;
      let params = self.parse_parameters()?;
      self.consume(&L_CURLY, "Expected '{' for the function body.")?;

      let min_arity = params.0;
      let max_arity = params.1.len() as u8;

      return Some(FunctionDeclNode {
         name,
         params: params.1,
         arity: (min_arity, max_arity),
         body: match self.parse_block()? {
            BlockStmt(b) => b.body,
            _ => unreachable!("Should have parsed a block statement."),
         },
      });
   }

   /// Parses a parameter declaration.
   pub(super) fn parse_parameters(&mut self) -> Option<(u8, Box<[Parameter]>)> {
      let mut params: Vec<Parameter> = vec![];
      let mut min_arity: u8 = 0;

      while !self.matches(&R_PAREN) {
         if params.len() >= 255 {
            self.error_at_current("Can't have more than 255 parameters.");
            return None;
         }

         let name = self.consume(&IDENTIFIER, "Expected a parameter name.")?;

         let param = if self.matches(&QUESTION) {
            Parameter {
               name,
               is_optional: true,
               default: None,
            }
         } else if self.matches(&COLON_EQUALS) {
            Parameter {
               name,
               is_optional: true,
               default: Some(Box::new(self.parse_expression()?)),
            }
         } else {
            Parameter {
               name,
               is_optional: false,
               default: None,
            }
         };

         if !params.is_empty() && !param.is_optional && params.last().unwrap().is_optional {
            self.error_at_token(
               &params.last().unwrap().name.clone(),
               "Optional and named parameters must be declared after all required parameters.",
            );
            return None;
         }

         if !param.is_optional {
            min_arity += 1
         }

         params.push(param);

         if !self.matches(&R_PAREN) {
            self.consume(&COMMA, "Expected a ',' between the parameter declarations.")?;
         } else {
            break;
         }
      }

      Some((min_arity, params.into_boxed_slice()))
   }

   /// Parses a `return` statement.
   fn parse_return_stmt(&mut self) -> Option<ASTNode> {
      let tok = self.previous.clone();

      // Compiles the return expression
      if !self.matches(&SEMICOLON) {
         let expr = self.parse_expression()?;
         self.consume(&SEMICOLON, "Expected a ';' after the expression.")?;

         return Some(ReturnStmt(ReturnStmtNode {
            token: tok,
            value: Some(Box::new(expr)),
         }));
      }

      Some(ReturnStmt(ReturnStmtNode {
         token: tok,
         value: None,
      }))
   }

   /// Parses a `class` declaration statement.
   fn parse_class_declaration(&mut self) -> Option<ASTNode> {
      let name = self.consume(&IDENTIFIER, "Expected an identifier for the class declaration.")?;

      self.consume(&L_CURLY, "Expected '{' for the class body.")?;
      let mut members: Vec<ClassMemberDeclNode> = vec![];

      while !self.matches(&R_CURLY) {
         let mut mode = self.capture_field_mode()?;

         let member_type = if self.matches(&FUNC_KW) {
            match self.parse_func_declaration() {
               Some(decl) => {
                  if decl.name.lexeme == "init" {
                     if (mode & 0b_0000_0100) == 4 {
                        self.error_at_token(&decl.name, "Class initializer cannot be static.");
                        return None;
                     } else if (mode & 0b_0000_0010) == 2 {
                        self.error_at_token(&decl.name, "Cannot override class initializer.");
                        return None;
                     }
                  }

                  ClassMemberDecl::Method(decl)
               }
               None => return None, // Could not parse method
            }
         } else if self.matches(&VAR_KW) {
            self.parse_var_declaration().map(ClassMemberDecl::Var)?
         } else if self.matches(&CONST_KW) {
            mode |= 0b_0000_0001; // Sets the "constant" mode bit.
            self.parse_const_declaration().map(ClassMemberDecl::Const)?
         } else {
            if self.check(&EOF) {
               self.error_at_current("Unexpected end of file while parsing class body.")
            } else {
               self.error_at_current("Unexpected token.");
            }
            return None;
         };

         members.push(ClassMemberDeclNode { member_type, mode });
      }

      Some(ClassDecl(ClassDeclNode {
         name,
         members: members.into_boxed_slice(),
      }))
   }

   /// Computes the modifier settings, or "mode", of a class field.
   fn capture_field_mode(&mut self) -> Option<u8> {
      let mut is_public = false;
      let mut is_static = false;
      let mut is_override = false;

      // [public, static, override, constant]
      // [0, 0, 0, 0] = 0  -> (private,    non-static,    non-override,    non-constant)
      // [0, 0, 0, 1] = 1  -> (private,    non-static,    non-override,    constant)
      // [0, 0, 1, 0] = 2  -> (private,    non-static,    override,        non-constant)
      // [0, 0, 1, 1] = 3  -> (private,    non-static,    override,        constant)
      // [0, 1, 0, 0] = 4  -> (private,    static,        non-override,    non-constant)
      // [0, 1, 0, 1] = 5  -> (private,    static,        non-override,    constant)
      // [0, 1, 1, 0] = 6  -> (private,    static,        override,        non-constant)
      // [0, 1, 1, 1] = 7  -> (private,    static,        override,        constant)
      // [1, 0, 0, 0] = 8  -> (public,     non-static,    non-override,    non-constant)
      // [1, 0, 0, 1] = 9  -> (public,     non-static,    non-override,    constant)
      // [1, 0, 1, 0] = 10 -> (public,     non-static,    override,        non-constant)
      // [1, 0, 1, 1] = 11 -> (public,     non-static,    override,        constant)
      // [1, 1, 0, 0] = 12 -> (public,     static,        non-override,    non-constant)
      // [1, 1, 0, 1] = 13 -> (public,     static,        non-override,    constant)
      // [1, 1, 1, 0] = 14 -> (public,     static,        override,        non-constant)
      // [1, 1, 1, 1] = 15 -> (public,     static,        override,        constant)
      // The `parse_class_declaration` function adds the `constant` mode bit.
      let mut mode: u8 = 0;

      while self.matches(&PUBLIC_KW) || self.matches(&STATIC_KW) || self.matches(&OVERRIDE_KW) {
         let tok_type = &self.previous.token_type;

         if matches!(tok_type, PUBLIC_KW) {
            if is_public {
               self.error_at_previous("Class field already marked as public.");
               return None;
            } else if is_static {
               self.error_at_current("The 'pub' keyword must precede the 'static' keyword.");
               return None;
            } else if is_override {
               self.error_at_current("The 'pub' keyword must precede the 'override' keyword.");
               return None;
            }

            is_public = true;
            mode |= 0b_0000_1000; // Sets the "public" mode bit.
            continue;
         }

         if matches!(tok_type, STATIC_KW) {
            if is_static {
               self.error_at_previous("Class field already marked as static.");
               return None;
            } else if is_override {
               self.error_at_current("The 'static' keyword must precede the 'override' keyword.");
               return None;
            }

            is_static = true;
            mode |= 0b_0000_0100; // Sets the "static" mode bit.
            continue;
         }

         if matches!(tok_type, OVERRIDE_KW) {
            if is_override {
               self.error_at_previous("Class field already marked as override.");
               return None;
            }

            is_override = true;
            mode |= 0b_0000_0010; // Sets the "override" mode bit.
            continue;
         }
      }

      Some(mode)
   }
}
