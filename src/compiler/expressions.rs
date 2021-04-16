use std::rc::Rc;

use crate::{
    chunk::{op_codes::OpCode, ConstantPos},
    objects::Object,
    lexer::tokens::TokenType,
};

use super::{
    precedence::{self, ParseFn, Precedence},
    Compiler,
};

impl<'a> Compiler<'a> {
    /// Compiles an expression.
    pub(super) fn expression(&mut self) {
        self.parse_with_precedence(Precedence::PREC_ASSIGNMENT);
    }

    /// Compiles an expression with a certain parse precedence.
    /// It parses all sub-expressions in the current expression with
    /// the same precedence or higher.
    ///
    /// ## Arguments
    /// * `prec` – The minimum parsing precedence
    pub(super) fn parse_with_precedence(&mut self, prec: Precedence) {
        self.advance();

        let prev_prefix_rule = precedence::get_rule(self.get_previous_tok_type()).prefix;

        if let ParseFn::NONE = prev_prefix_rule {
            self.error_at_previous("Expected an expression");
            return ();
        }

        let can_assign = (prec.clone() as u8) <= (Precedence::PREC_ASSIGNMENT as u8);
        self.execute_rule(prev_prefix_rule, can_assign);

        while (prec.clone() as u8) <= (precedence::get_rule(self.get_current_tok_type()).precedence as u8) {
            self.advance();
            let prev_infix_rule = precedence::get_rule(self.get_previous_tok_type()).infix;
            self.execute_rule(prev_infix_rule, can_assign);
        }

        if can_assign && self.matches(TokenType::EQUALS_SIGN) {
            self.error_at_previous("Invalid assignment target.");
        }
    }

    /// Executes a parsing function from a given parser rule.
    ///
    /// ## Arguments
    /// * `rule` – The parser rule to execute
    /// * `can_assign` – Wether or not this is an assignment expression.
    pub(super) fn execute_rule(&mut self, rule: ParseFn, _can_assign: bool) {
        match rule {
            ParseFn::CompileBinaryExpr => self.compile_binary_expr(),
            ParseFn::CompileBinaryNum => self.compile_int_from_base(2),
            ParseFn::CompileGrouping => {
                self.expression();
                self.consume(TokenType::RIGHT_PARENTHESIS, "Expected ')' after expression.")
            }
            ParseFn::CompileHexNum => self.compile_int_from_base(16),
            ParseFn::CompileLiteral => self.compile_literal(),
            ParseFn::CompileLogicAnd => self.logic_and(),
            ParseFn::CompileLogicOr => self.logic_or(),
            ParseFn::CompileNumeric => self.compile_number(),
            ParseFn::CompileOctalNum => self.compile_int_from_base(8),
            ParseFn::CompileString => self.compile_string(),
            ParseFn::CompileUnary => self.compile_unary(),
            ParseFn::CompileTernary => self.compile_ternary_expression(),
            ParseFn::CompileVariable => self.consume_variable_identifier(),
            ParseFn::NONE => return (),
        }
    }

    /// Tries to add an object to the constant pool: if the object was successfully
    /// added to the pool, also adds an `OpCode::CONSTANT(u16)` instruction to the chunk.
    /// Otherwise, if it could add the constant to the pool, reports an error.
    ///
    /// ## Arguments
    /// * `obj` – The object to be added to the pool
    /// * `add_const` – Wether or not the compiler should also emit an `OpCode::OP_CONSTANT`
    /// instruction before the other two bytes.
    ///
    /// ## Returns
    /// * `Option<u16>` – The position of the constant in the pool if it was added successfully.
    pub(super) fn emit_constant_instruction(&mut self, obj: Rc<Object<'a>>, add_const: bool) -> Option<u16> {
        let pos = self.chunk.add_constant(obj);

        match pos {
            ConstantPos::Pos(x) => {
                if add_const {
                    self.emit_op_code(OpCode::OP_CONSTANT);
                }

                // Emit the index of the constant in the pool
                self.emit_short(x);
                Some(x)
            }
            ConstantPos::Error => {
                self.error_at_previous("Too many constants in one chunk.");
                None
            }
        }
    }

    /// Compiles a string.
    pub(super) fn compile_string(&mut self) {
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
        self.emit_constant_instruction(Rc::new(Object::String(lexeme)), true);
    }

    /// Compiles a number.
    pub(super) fn compile_number(&mut self) {
        let lexeme = self.previous.lexeme.clone();
        // Removes the underscores from the lexeme
        let lexeme = lexeme.replace('_', "");
        // Parses the lexeme into a float
        let num = lexeme.parse::<f64>();

        // If the lexeme could successfully be converted to `isize` integer
        // then we proceed to save it in the constant pool and emit the
        // instruction. Otherwise, we indicate that there was a compilation error.
        match num {
            Ok(x) => {
                self.emit_constant_instruction(Rc::new(Object::Number(x)), true);
            }
            Err(_) => todo!("Throw a meaningful error message if the lexeme could not be converted to a Rust float."),
        }
    }

    /// Compiles a binary, octal, or hexadecimal number.
    pub(super) fn compile_int_from_base(&mut self, radix: u32) {
        let lexeme = self.previous.lexeme.clone();
        // Removes the underscores from the lexeme
        let lexeme = lexeme.replace('_', "");
        // Parses the lexeme into an integer
        let num = isize::from_str_radix(&lexeme, radix);

        // If the lexeme could successfully be converted to `isize` integer
        // then we proceed to save it in the constant pool and emit the
        // instruction. Otherwise, we indicate that there was a compilation error.
        match num {
            Ok(x) => {
                self.emit_constant_instruction(Rc::new(Object::Number(x as f64)), true);
            }
            Err(_) => todo!("Throw a meaningful error message if the lexeme could not be converted to a Rust float."),
        }
    }

    /// Compiles a literal value (true, false, and null).
    pub(super) fn compile_literal(&mut self) {
        match self.get_previous_tok_type() {
            TokenType::TRUE_LITERAL => self.emit_op_code(OpCode::OP_TRUE),
            TokenType::FALSE_LITERAL => self.emit_op_code(OpCode::OP_FALSE),
            TokenType::NULL_LITERAL => self.emit_op_code(OpCode::OP_NULL),
            _ => return (), // Unreachable.
        }
    }

    /// Compiles a unary expression.
    pub(super) fn compile_unary(&mut self) {
        let operator_type = self.get_previous_tok_type();

        // Compile the operand.
        self.parse_with_precedence(Precedence::PREC_UNARY);

        // Emit the operator instruction.
        match operator_type {
            TokenType::LOGICAL_NOT => self.emit_op_code(OpCode::OP_LOGIC_NOT),
            TokenType::BITWISE_NOT => self.emit_op_code(OpCode::OP_BITWISE_NOT),
            TokenType::MINUS => self.emit_op_code(OpCode::OP_NEGATE),
            _ => return (), // Unreachable.
        }
    }

    // Compiles a binary expression.
    pub(super) fn compile_binary_expr(&mut self) {
        // Remember the operator.
        let operator_type = self.get_previous_tok_type();

        // Compile the right operand.
        let rule = precedence::get_rule(self.get_previous_tok_type());
        // We use one higher level of precedence for the right operand because
        // the binary operators are left-associative. Given a series of the same
        // operator, like: `1 + 2 + 3 + 4` We want to parse it like: `((1 + 2) + 3)
        // + 4` Thus, when parsing the right-hand operand to the first +, we want to
        // consume the 2, but not the rest, so we use one level above +’s
        // precedence. But if our operator was right-associative, this would be
        // wrong. Given: `a = b = c = d` Since assignment is right-associative, we
        // want to parse it as: `a = (b = (c = d))` To enable that, we would call
        // parsePrecedence() with the same precedence as the current operator.
        self.parse_with_precedence(Precedence::get_by_val((rule.precedence as u8) + 1));

        // Emit the operator instruction.
        match operator_type {
            TokenType::PLUS => self.emit_op_code(OpCode::OP_ADD),
            TokenType::MINUS => self.emit_op_code(OpCode::OP_SUBTRACT),
            TokenType::STAR => self.emit_op_code(OpCode::OP_MULTIPLY),
            TokenType::SLASH => self.emit_op_code(OpCode::OP_DIVIDE),
            TokenType::MODULUS => self.emit_op_code(OpCode::OP_MODULUS),
            TokenType::EXPO => self.emit_op_code(OpCode::OP_EXPO),
            TokenType::BITWISE_OR => self.emit_op_code(OpCode::OP_BITWISE_OR),
            TokenType::BITWISE_XOR => self.emit_op_code(OpCode::OP_BITWISE_XOR),
            TokenType::BITWISE_AND => self.emit_op_code(OpCode::OP_BITWISE_AND),
            TokenType::BITWISE_LEFT_SHIFT => self.emit_op_code(OpCode::OP_BITWISE_L_SHIFT),
            TokenType::BITWISE_RIGHT_SHIFT => self.emit_op_code(OpCode::OP_BITWISE_R_SHIFT),
            TokenType::LOGICAL_EQ => self.emit_op_code(OpCode::OP_EQUALS),
            TokenType::LOGICAL_NOT_EQ => self.emit_op_code(OpCode::OP_NOT_EQUALS),
            TokenType::GREATER_THAN => self.emit_op_code(OpCode::OP_GREATER_THAN),
            TokenType::GREATER_THAN_EQ => self.emit_op_code(OpCode::OP_GREATER_THAN_EQ),
            TokenType::LESS_THAN => self.emit_op_code(OpCode::OP_LESS_THAN),
            TokenType::LESS_THAN_EQ => self.emit_op_code(OpCode::OP_LESS_THAN_EQ),
            TokenType::RANGE_OPERATOR => self.emit_op_code(OpCode::OP_GENERATE_RANGE),
            TokenType::NULLISH_COALESCING => self.emit_op_code(OpCode::OP_NULLISH_COALESCING),
            _ => return (), // Unreachable.
        }
    }

    /// Compiles a ternary expression.
    pub(super) fn compile_ternary_expression(&mut self) {
        match self.get_previous_tok_type() {
            TokenType::QUESTION_MARK => {
                // Ternary expressions are right-associative, so we parse the operands
                // with the same level of precedence as another ternary expression.
                self.parse_with_precedence(Precedence::PREC_TERNARY);
                self.consume(TokenType::COLON_SEPARATOR, "Expected ':' for ternary expression.");
                self.parse_with_precedence(Precedence::PREC_TERNARY);

                // Add the ternary OpCode
                self.emit_op_code(OpCode::OP_TERNARY);
            }
            // TODO: Allowing ternary conditional expressions of the form
            // `a if x else b` is a design choice. Should ternary expressions
            // of this form also be allowed? Should we chose one or the other?
            // What are the benefits? and What Hinton programmers prefer?
            _ => return (), // Unreachable.
        }
    }

    /// Compiles a logic-and expression
    pub(super) fn logic_and(&mut self) {
        todo!("To be implemented");
    }

    /// Compiles a logic-or expression
    pub(super) fn logic_or(&mut self) {
        todo!("To be implemented");
    }
}
