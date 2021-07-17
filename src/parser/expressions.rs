// ========= Precedence Table for Expressions (Low to High) =========
// Assignment:      <identifier> (=, +=, -=, /=, etc...) <expr>
// Ternary:         <expr> ? <expr> : <expr>
// Nullish:         <expr> ?? <expr>
// Logic Or:        (<expr> || <expr>), (<expr> or <expr>)
// Logic And:       (<expr> && <expr>), (<expr> and <expr>)
// Bitwise Or:      <expr> | <expr>
// Bitwise Xor:     <expr> ^ <expr>
// Bitwise And:     <expr> & <expr>
// Equality:        (<expr> == <expr>), (<expr> equals <expr>)
// Comparison:      <expr> (<, <=, >, >=) <expr>
// Range:           <expr> .. <expr>
// Bitwise Shift:   <expr> (<<, >>) <expr>
// Term:            <expr> (+, -) <expr>
// Factor:          <expr> (*, /, %, mod) <expr>
// Expo:            <expr> ** <expr>
// Unary:           (~, !, -, not) <expr>
// ==================================================================

use crate::core::ast::ASTNode::*;
use crate::core::ast::*;
use crate::core::ast::{ASTNode, ReassignmentType};
use crate::core::tokens::Token;
use crate::core::tokens::TokenType::*;
use crate::core::tokens::TokenType::{LOGIC_NOT_EQ, MINUS};
use crate::objects::Object;
use crate::parser::Parser;

impl<'a> Parser {
   /// Parses an expression.
   pub(super) fn parse_expression(&mut self) -> Option<ASTNode> {
      self.parse_assignment()
   }

   /// Parses an assignment expression.
   fn parse_assignment(&mut self) -> Option<ASTNode> {
      let expr = self.parse_ternary_conditional();
      let expr_tok = self.previous.clone();

      if self.matches(&EQUALS)
         || self.matches(&PLUS_EQ)
         || self.matches(&MINUS_EQ)
         || self.matches(&STAR_EQ)
         || self.matches(&SLASH_EQ)
         || self.matches(&EXPO_EQUALS)
         || self.matches(&MOD_EQ)
         || self.matches(&BIT_L_SHIFT_EQ)
         || self.matches(&BIT_R_SHIFT_EQ)
         || self.matches(&BIT_AND_EQ)
         || self.matches(&BIT_XOR_EQ)
         || self.matches(&BIT_OR_EQ)
      {
         let opr = self.previous.clone();

         // Gets the type of reassignment
         let opr_type = match opr.token_type {
            PLUS_EQ => ReassignmentType::Plus,
            MINUS_EQ => ReassignmentType::Minus,
            STAR_EQ => ReassignmentType::Mul,
            SLASH_EQ => ReassignmentType::Div,
            EXPO_EQUALS => ReassignmentType::Expo,
            MOD_EQ => ReassignmentType::Mod,
            BIT_L_SHIFT_EQ => ReassignmentType::ShiftL,
            BIT_R_SHIFT_EQ => ReassignmentType::ShiftR,
            BIT_AND_EQ => ReassignmentType::BitAnd,
            BIT_XOR_EQ => ReassignmentType::Xor,
            BIT_OR_EQ => ReassignmentType::BitOr,
            // Regular re-assignment
            _ => ReassignmentType::Assign,
         };

         // Gets the value for assignment
         let rhs = self.parse_expression()?;

         // Returns the assignment expression of the corresponding type
         return match expr {
            Some(node) => match node {
               // Variable re-assignment.
               Identifier(id) => Some(VarReassignment(VarReassignmentExprNode {
                  target: id.token,
                  opr_type,
                  value: Box::new(rhs),
                  pos: (opr.line_num, opr.column_start),
               })),

               // Object setter `object.property = value`.
               PropGetExpr(getter) => Some(PropSetExpr(PropSetExprNode {
                  target: getter.target,
                  setter: getter.getter,
                  value: Box::new(rhs),
                  opr_type,
               })),

               // Subscript assignment `a[expression] = value`
               Subscript(sub) => Some(SubscriptAssignment(SubscriptAssignExprNode {
                  target: sub.target,
                  index: sub.index,
                  value: Box::new(rhs),
                  pos: (opr.line_num, opr.column_start),
                  opr_type,
               })),

               // The assignment target is not valid
               _ => {
                  self.error_at_token(&expr_tok, "Invalid assignment target.");
                  None
               }
            },

            // Could not parse lhs of expression
            None => None,
         };
      }

      expr
   }

   /// Parses a ternary conditional expression.
   fn parse_ternary_conditional(&mut self) -> Option<ASTNode> {
      let mut expr = self.parse_nullish_coalescing();

      if self.matches(&QUESTION) {
         let true_branch_opr = self.previous.clone();
         let branch_true = self.parse_expression()?;

         self.consume(&COLON, "Expected ':' after the expression.")?;

         let false_branch_opr = self.previous.clone();
         let branch_false = self.parse_expression()?;

         expr = Some(TernaryConditional(TernaryConditionalNode {
            condition: Box::new(expr?),
            true_branch_token: true_branch_opr,
            branch_true: Box::new(branch_true),
            branch_false: Box::new(branch_false),
            false_branch_token: false_branch_opr,
         }));
      }

      expr
   }

   /// Parses an '??' (nullish coalescing) expression.
   fn parse_nullish_coalescing(&mut self) -> Option<ASTNode> {
      let mut expr = self.parse_logic_or();

      while self.matches(&NULLISH) {
         let opr = self.previous.clone();

         expr = Some(Binary(BinaryExprNode {
            left: Box::new(expr?),
            right: Box::new(self.parse_logic_or()?),
            opr_token: opr,
            opr_type: BinaryExprType::Nullish,
         }));
      }

      expr
   }

   /// Parses an 'OR' expression.
   fn parse_logic_or(&mut self) -> Option<ASTNode> {
      let mut expr = self.parse_logic_and();

      while self.matches(&LOGIC_OR) {
         let opr = self.previous.clone();

         expr = Some(Binary(BinaryExprNode {
            left: Box::new(expr?),
            right: Box::new(self.parse_logic_and()?),
            opr_token: opr,
            opr_type: BinaryExprType::LogicOR,
         }));
      }

      expr
   }

   /// Parses an 'AND' expression.
   fn parse_logic_and(&mut self) -> Option<ASTNode> {
      let mut expr = self.parse_bitwise_or();

      while self.matches(&LOGIC_AND) {
         let opr = self.previous.clone();

         expr = Some(Binary(BinaryExprNode {
            left: Box::new(expr?),
            right: Box::new(self.parse_bitwise_or()?),
            opr_token: opr,
            opr_type: BinaryExprType::LogicAND,
         }));
      }

      expr
   }

   /// Parses a 'BITWISE OR' expression.
   fn parse_bitwise_or(&mut self) -> Option<ASTNode> {
      let mut expr = self.parse_bitwise_xor();

      while self.matches(&BIT_OR) {
         let opr = self.previous.clone();

         expr = Some(Binary(BinaryExprNode {
            left: Box::new(expr?),
            right: Box::new(self.parse_bitwise_xor()?),
            opr_token: opr,
            opr_type: BinaryExprType::BitwiseOR,
         }));
      }

      expr
   }

   /// Parses a 'BITWISE XOR' expression.
   fn parse_bitwise_xor(&mut self) -> Option<ASTNode> {
      let mut expr = self.parse_bitwise_and();

      while self.matches(&BIT_XOR) {
         let opr = self.previous.clone();

         expr = Some(Binary(BinaryExprNode {
            left: Box::new(expr?),
            right: Box::new(self.parse_bitwise_and()?),
            opr_token: opr,
            opr_type: BinaryExprType::BitwiseXOR,
         }));
      }

      expr
   }

   /// Parses a 'BITWISE AND' expression.
   fn parse_bitwise_and(&mut self) -> Option<ASTNode> {
      let mut expr = self.parse_equality();

      while self.matches(&BIT_AND) {
         let opr = self.previous.clone();

         expr = Some(Binary(BinaryExprNode {
            left: Box::new(expr?),
            right: Box::new(self.parse_equality()?),
            opr_token: opr,
            opr_type: BinaryExprType::BitwiseAND,
         }));
      }

      expr
   }

   /// Parses an equality expression.
   fn parse_equality(&mut self) -> Option<ASTNode> {
      let mut expr = self.parse_comparison();

      while self.matches(&LOGIC_EQ) || self.matches(&LOGIC_NOT_EQ) {
         let opr = self.previous.clone();

         let opr_type = if let LOGIC_EQ = opr.token_type {
            BinaryExprType::LogicEQ
         } else {
            BinaryExprType::LogicNotEQ
         };

         expr = Some(Binary(BinaryExprNode {
            left: Box::new(expr?),
            right: Box::new(self.parse_comparison()?),
            opr_token: opr,
            opr_type,
         }));
      }

      expr
   }

   /// Parses a comparison expression.
   fn parse_comparison(&mut self) -> Option<ASTNode> {
      let mut expr = self.parse_range();

      while self.matches(&LESS_THAN)
         || self.matches(&LESS_THAN_EQ)
         || self.matches(&GREATER_THAN)
         || self.matches(&GREATER_THAN_EQ)
      {
         let opr = self.previous.clone();

         let opr_type = if let LESS_THAN = opr.token_type {
            BinaryExprType::LogicLessThan
         } else if let LESS_THAN_EQ = opr.token_type {
            BinaryExprType::LogicLessThanEQ
         } else if let GREATER_THAN = opr.token_type {
            BinaryExprType::LogicGreaterThan
         } else {
            BinaryExprType::LogicGreaterThanEQ
         };

         expr = Some(Binary(BinaryExprNode {
            left: Box::new(expr?),
            right: Box::new(self.parse_range()?),
            opr_token: opr,
            opr_type,
         }));
      }

      expr
   }

   /// Parses a range expression.
   fn parse_range(&mut self) -> Option<ASTNode> {
      let mut expr = self.parse_bitwise_shift();

      if self.matches(&RANGE_OPR) {
         let opr = self.previous.clone();

         expr = Some(Binary(BinaryExprNode {
            left: Box::new(expr?),
            right: Box::new(self.parse_bitwise_shift()?),
            opr_token: opr,
            opr_type: BinaryExprType::Range,
         }));
      }

      expr
   }

   /// Parses a 'BITWISE SHIFT'.
   fn parse_bitwise_shift(&mut self) -> Option<ASTNode> {
      let mut expr = self.parse_term();

      while self.matches(&BIT_L_SHIFT) || self.matches(&BIT_R_SHIFT) {
         let opr = self.previous.clone();

         let opr_type = if let BIT_L_SHIFT = opr.token_type {
            BinaryExprType::BitwiseShiftLeft
         } else {
            BinaryExprType::BitwiseShiftRight
         };

         expr = Some(Binary(BinaryExprNode {
            left: Box::new(expr?),
            right: Box::new(self.parse_term()?),
            opr_token: opr,
            opr_type,
         }));
      }

      expr
   }

   /// Parses a term expression.
   fn parse_term(&mut self) -> Option<ASTNode> {
      let mut expr = self.parse_factor();

      while self.matches(&PLUS) || self.matches(&MINUS) {
         let opr = self.previous.clone();

         let opr_type = if let PLUS = opr.token_type {
            BinaryExprType::Addition
         } else {
            BinaryExprType::Minus
         };

         expr = Some(Binary(BinaryExprNode {
            left: Box::new(expr?),
            right: Box::new(self.parse_factor()?),
            opr_token: opr,
            opr_type,
         }));
      }

      expr
   }

   /// Parses a factor expression.
   fn parse_factor(&mut self) -> Option<ASTNode> {
      let mut expr = self.parse_expo();

      while self.matches(&SLASH) || self.matches(&STAR) || self.matches(&MODULUS) {
         let opr = self.previous.clone();

         let opr_type = if let SLASH = opr.token_type {
            BinaryExprType::Division
         } else if let STAR = opr.token_type {
            BinaryExprType::Multiplication
         } else {
            BinaryExprType::Modulus
         };

         expr = Some(Binary(BinaryExprNode {
            left: Box::new(expr?),
            right: Box::new(self.parse_expo()?),
            opr_token: opr,
            opr_type,
         }));
      }

      expr
   }

   /// Parses an exponentiation expression.
   fn parse_expo(&mut self) -> Option<ASTNode> {
      let mut expr = self.parse_unary();

      while self.matches(&EXPO) {
         let opr = self.previous.clone();

         expr = Some(Binary(BinaryExprNode {
            left: Box::new(expr?),
            right: Box::new(self.parse_unary()?),
            opr_token: opr,
            opr_type: BinaryExprType::Expo,
         }));
      }

      expr
   }

   /// Parses a unary expression.
   fn parse_unary(&mut self) -> Option<ASTNode> {
      if self.matches(&LOGIC_NOT) || self.matches(&MINUS) || self.matches(&BIT_NOT) {
         let opr = self.previous.clone();

         let opr_type = if let LOGIC_NOT = opr.token_type {
            UnaryExprType::LogicNeg
         } else if let BIT_NOT = opr.token_type {
            UnaryExprType::BitwiseNeg
         } else {
            UnaryExprType::ArithmeticNeg
         };

         Some(Unary(UnaryExprNode {
            operand: Box::new(self.parse_expression()?),
            pos: (opr.line_num, opr.column_start),
            opr_type,
         }))
      } else {
         let mut expr = self.parse_primary();

         loop {
            if self.matches(&L_BRACKET) {
               // Parse array indexing
               expr = Some(Subscript(SubscriptExprNode {
                  target: Box::new(expr?),
                  index: Box::new(self.parse_expression()?),
                  pos: (self.previous.line_num, self.previous.column_start),
               }));

               self.consume(&R_BRACKET, "Expected matching ']' for array indexing expression.")?;
            } else if self.matches(&L_PAREN) {
               let call_pos = (self.previous.line_num, self.previous.column_start);

               // Parse function call
               expr = Some(FunctionCall(FunctionCallExprNode {
                  target: Box::new(expr?),
                  args: self.parse_arguments_list()?,
                  pos: call_pos,
               }))
            } else if self.matches(&DOT) {
               // Property access
               expr = Some(PropGetExpr(PropGetExprNode {
                  target: Box::new(expr?),
                  getter: self.consume(&IDENTIFIER, "Expected property name after the dot.")?,
               }))
            } else {
               break;
            }
         }

         expr
      }
   }

   /// Parses a primary (literal) expression.
   pub(super) fn parse_primary(&mut self) -> Option<ASTNode> {
      self.advance();
      let literal_token = self.previous.clone();

      let literal_value = match self.get_previous_tok_type() {
         STRING => self.compile_string(),
         TRUE => Object::Bool(true),
         FALSE => Object::Bool(false),
         NULL => Object::Null,
         L_BRACKET => return self.construct_array(),
         INTEGER => self.compile_integer().ok()?,
         FLOAT => self.compile_float().ok()?,
         BINARY => self.compile_int_from_base(2).ok()?,
         OCTAL => self.compile_int_from_base(8).ok()?,
         HEXADECIMAL => self.compile_int_from_base(16).ok()?,
         L_PAREN => {
            let start_token = self.previous.clone();

            // If the parenthesis are empty, then we parse this as an empty tuple.
            return if self.matches(&R_PAREN) {
               Some(Tuple(TupleExprNode {
                  values: vec![].into_boxed_slice(),
                  token: start_token,
               }))
            } else {
               let expr = self.parse_expression();

               // If there is a comma after the first expression, then this becomes a tuple.
               if self.matches(&COMMA) {
                  self.parse_tuple(start_token, expr)
               } else {
                  self.consume(&R_PAREN, "Expected closing ')'.")?;
                  // For grouping expression, we don't wrap the inner expression inside an extra node.
                  // Instead, we return the actual expression that was enclosed in the parenthesis.
                  expr
               }
            };
         }
         L_CURLY => {
            return self.parse_dictionary();
         }
         IDENTIFIER => {
            return Some(Identifier(IdentifierExprNode {
               token: self.previous.clone(),
            }))
         }
         SELF_KW => {
            return Some(SelfExpr(SelfExprNode {
               token: self.previous.clone(),
            }))
         }
         NEW_KW => {
            // For class instances, we parse a unary after the "new" keyword so that the instance can
            // be parsed and compiled as a regular function call. The only purpose of the "new" keyword
            // is to differentiate between a function call, and a class instance in Hinton code.
            return match self.parse_unary()? {
               ASTNode::FunctionCall(call) => Some(Instance(call)),
               _ => {
                  self.error_at_current("Expected class instance.");
                  None
               }
            };
         }
         FN_LAMBDA_KW => {
            let fn_keyword = self.previous.clone();

            self.consume(&L_PAREN, "Expected '(' before lambda expression parameters.")?;
            let params = self.parse_parameters()?;
            self.consume(&L_CURLY, "Expected '{' for the function body.")?;

            let min_arity = params.0;
            let max_arity = params.1.len() as u8;

            return Some(Lambda(FunctionDeclNode {
               name: fn_keyword,
               params: params.1,
               arity: (min_arity, max_arity),
               body: match self.parse_block()? {
                  BlockStmt(b) => b.body,
                  _ => unreachable!("Should have parsed a block statement."),
               },
            }));
         }
         _ => {
            self.error_at_previous("Unexpected token.");
            return None;
         }
      };

      Some(Literal(LiteralExprNode {
         value: literal_value,
         token: literal_token,
      }))
   }

   /// Compiles a string token to a Hinton String.
   ///
   /// # Returns
   /// `Object`: The Hinton string object.
   fn compile_string(&mut self) -> Object {
      let lexeme = self.previous.lexeme.clone();

      // Remove outer quotes from the source string
      let lexeme = &lexeme[1..(lexeme.len() - 1)];

      // Replace escaped characters with the actual representations
      let lexeme = lexeme
         .replace("\\n", "\n")
         .replace("\\t", "\t")
         .replace("\\r", "\r")
         .replace("\\\\", "\\")
         .replace("\\\"", "\"");

      // Emits the constant instruction
      Object::from(lexeme)
   }

   /// Compiles an integer token to a Hinton Int.
   ///
   /// # Returns
   /// `Result<Object, ()>`: The Hinton number object.
   fn compile_integer(&mut self) -> Result<Object, ()> {
      let lexeme = self.previous.lexeme.clone();
      // Removes the underscores from the lexeme
      let lexeme = lexeme.replace('_', "");

      // Parses the lexeme into a float
      match lexeme.parse::<i64>() {
         Ok(x) => Ok(Object::Int(x)),
         Err(_) => {
            // The lexeme could not be converted to an i64.
            self.error_at_previous("Unexpected token.");
            Err(())
         }
      }
   }

   /// Compiles a float token to a Hinton Float.
   ///
   /// # Returns
   /// `Rc<Object>`: The Hinton number object.
   fn compile_float(&mut self) -> Result<Object, ()> {
      let lexeme = self.previous.lexeme.clone();
      // Removes the underscores from the lexeme
      let lexeme = lexeme.replace('_', "");

      // Parses the lexeme into a float
      match lexeme.parse::<f64>() {
         Ok(x) => Ok(Object::Float(x)),
         Err(_) => {
            // The lexeme could not be converted to a f64.
            self.error_at_previous("Unexpected token.");
            Err(())
         }
      }
   }

   /// Compiles a binary, octal, or hexadecimal number token to a Hinton Number.
   ///
   /// # Returns
   /// `Result<Object, ()>`: If there was no error converting the lexeme to an integer
   /// of the specified base, returns the Hinton number object. Otherwise, returns an empty error.
   fn compile_int_from_base(&mut self, radix: u32) -> Result<Object, ()> {
      let lexeme = self.previous.lexeme.clone();
      // Removes the underscores from the lexeme
      let lexeme = lexeme.replace('_', "");

      // Parses the lexeme into an integer
      match i64::from_str_radix(&lexeme[2..], radix) {
         Ok(x) => Ok(Object::Int(x)),
         Err(_) => {
            // The lexeme could not be converted to an i64.
            self.error_at_previous("Unexpected token.");
            Err(())
         }
      }
   }

   /// Parses an array expression.
   fn construct_array(&mut self) -> Option<ASTNode> {
      let start_token = self.previous.clone();
      let mut values: Vec<ASTNode> = vec![];

      if !self.matches(&R_BRACKET) {
         loop {
            values.push(self.parse_expression()?);

            if self.matches(&COMMA) {
               continue;
            }

            self.consume(&R_BRACKET, "Expected matching ']' for array literal.");
            break;
         }
      }

      Some(Array(ArrayExprNode {
         values: values.into_boxed_slice(),
         token: start_token,
      }))
   }

   /// Parses a tuple literal expression.
   fn parse_tuple(&mut self, start_token: Token, first: Option<ASTNode>) -> Option<ASTNode> {
      let mut values: Vec<ASTNode> = vec![first?];

      if !self.matches(&R_PAREN) {
         loop {
            values.push(self.parse_expression()?);

            if self.matches(&COMMA) {
               continue;
            }

            self.consume(&R_PAREN, "Expected matching ')' for tuple declaration.");
            break;
         }
      }

      Some(Tuple(TupleExprNode {
         values: values.into_boxed_slice(),
         token: start_token,
      }))
   }

   /// Parses a dictionary literal expression.
   fn parse_dictionary(&mut self) -> Option<ASTNode> {
      let token = self.previous.clone();
      let mut keys: Vec<Token> = vec![];
      let mut values: Vec<ASTNode> = vec![];

      if !self.matches(&R_CURLY) {
         loop {
            // Parses the key
            match self.parse_primary() {
               Some(Identifier(id)) => keys.push(id.token),
               Some(Literal(lit)) => match lit.value {
                  Object::String(_) => keys.push(lit.token),
                  _ => {
                     self.error_at_previous("Expected an identifier, or a string, for dictionary key.");
                     return None;
                  }
               },
               _ => {
                  self.error_at_previous("Expected an identifier, or a string, for dictionary key.");
                  return None;
               }
            }

            // Consumes the colon
            self.consume(&COLON, "Expected ':' after dictionary key.")?;

            // Consumes the value
            values.push(self.parse_expression()?);

            // If matches a comma, consume next
            if self.matches(&COMMA) {
               // If there is a closing curly brace after the comma, we assume it
               // is the end of the dictionary.
               if self.check(&R_CURLY) {
                  self.advance();
                  break;
               }

               continue;
            }

            self.consume(&R_CURLY, "Expected matching '}' for dictionary literal.");
            break;
         }
      }

      Some(Dictionary(DictionaryExprNode {
         keys: keys.into_boxed_slice(),
         values: values.into_boxed_slice(),
         token,
      }))
   }

   /// Parses a function argument expression.
   fn parse_arguments_list(&mut self) -> Option<Box<[Argument]>> {
      let mut args: Vec<Argument> = vec![];

      while !self.matches(&R_PAREN) {
         if args.len() >= 256 {
            self.error_at_current("Can't have more than 255 arguments.");
            return None;
         }

         args.push(Argument {
            name: None,
            value: Box::new(self.parse_expression()?),
         });

         if self.matches(&R_PAREN) {
            break;
         }

         self.consume(&COMMA, "Expected a comma after the argument.");
      }

      Some(args.into_boxed_slice())
   }
}
