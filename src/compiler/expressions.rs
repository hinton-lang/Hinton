use super::{Compiler, CompilerError, SymbolType};
use crate::{ast::*, chunk::OpCode, lexer::tokens::Token, objects::Object};

impl Compiler {
    /// Compiles a literal expression.
    ///
    /// # Arguments
    /// * `expr` – A literal expression node.
    pub(super) fn compile_literal_expr(&mut self, expr: &LiteralExprNode) {
        let obj = expr.value.clone();
        let opr_pos = (expr.token.line_num, expr.token.column_num);

        match obj {
            Object::Bool(x) if x => {
                self.emit_op_code(OpCode::LoadImmTrue, opr_pos);
            }
            Object::Bool(x) if !x => {
                self.emit_op_code(OpCode::LoadImmFalse, opr_pos);
            }
            Object::Null => {
                self.emit_op_code(OpCode::LoadImmNull, opr_pos);
            }
            Object::Int(x) if x == 0i64 => {
                self.emit_op_code(OpCode::LoadImm0I, opr_pos);
            }
            Object::Int(x) if x == 1i64 => {
                self.emit_op_code(OpCode::LoadImm1I, opr_pos);
            }
            Object::Float(x) if x == 0f64 => {
                self.emit_op_code(OpCode::LoadImm0F, opr_pos);
            }
            Object::Float(x) if x == 1f64 => {
                self.emit_op_code(OpCode::LoadImm1F, opr_pos);
            }
            Object::Int(x) if x > 1i64 => {
                if x < 256i64 {
                    self.emit_op_code(OpCode::LoadImmN, opr_pos);
                    self.emit_raw_byte(x as u8, opr_pos);
                } else if x < (u16::MAX as i64) {
                    self.emit_op_code(OpCode::LoadImmNLong, opr_pos);
                    self.emit_short(x as u16, opr_pos);
                } else {
                    // If the number cannot be encoded within two bytes (as an unsigned short),
                    // the we add it to the constant pool.
                    self.add_literal_to_pool(obj, &expr.token)
                }
            }
            _ => self.add_literal_to_pool(obj, &expr.token),
        };
    }

    /// Compiles a unary expression.
    ///
    /// # Arguments
    /// * `expr` – A unary expression node.
    pub(super) fn compile_unary_expr(&mut self, expr: &UnaryExprNode) {
        self.compile_node(&expr.operand);

        let expression_op_code = match expr.opr_type {
            UnaryExprType::ArithmeticNeg => OpCode::Negate,
            UnaryExprType::LogicNeg => OpCode::LogicNot,
            UnaryExprType::BitwiseNeg => OpCode::BitwiseNot,
        };

        self.emit_op_code(expression_op_code, expr.pos);
    }

    /// Compiles a binary expression.
    ///
    /// # Arguments
    /// * `expr` – A binary expression node.
    pub(super) fn compile_binary_expr(&mut self, expr: &BinaryExprNode) {
        // Because logic 'AND' expressions are short-circuit expressions,
        // they are compiled by a separate function.
        if let BinaryExprType::LogicAND = expr.opr_type {
            return self.compile_and_expr(expr);
        }

        // Because logic 'OR' expressions are short-circuit expressions,
        // they are compiled by a separate function.
        if let BinaryExprType::LogicOR = expr.opr_type {
            return self.compile_or_expr(expr);
        }

        // Compiles the binary operators.
        self.compile_node(&expr.left);
        self.compile_node(&expr.right);

        let expression_op_code = match expr.opr_type {
            BinaryExprType::BitwiseAND => OpCode::BitwiseAnd,
            BinaryExprType::BitwiseOR => OpCode::BitwiseOr,
            BinaryExprType::BitwiseShiftLeft => OpCode::BitwiseShiftLeft,
            BinaryExprType::BitwiseShiftRight => OpCode::BitwiseShiftRight,
            BinaryExprType::BitwiseXOR => OpCode::BitwiseXor,
            BinaryExprType::Division => OpCode::Divide,
            BinaryExprType::Expo => OpCode::Expo,
            BinaryExprType::LogicAND => {
                unreachable!("The 'AND' expression should have been compiled by now.")
            }
            BinaryExprType::LogicEQ => OpCode::Equals,
            BinaryExprType::LogicGreaterThan => OpCode::GreaterThan,
            BinaryExprType::LogicGreaterThanEQ => OpCode::GreaterThanEq,
            BinaryExprType::LogicLessThan => OpCode::LessThan,
            BinaryExprType::LogicLessThanEQ => OpCode::LessThanEq,
            BinaryExprType::LogicNotEQ => OpCode::NotEq,
            BinaryExprType::LogicOR => {
                unreachable!("The 'OR' expression should have been compiled by now.")
            }
            BinaryExprType::Minus => OpCode::Subtract,
            BinaryExprType::Modulus => OpCode::Modulus,
            BinaryExprType::Multiplication => OpCode::Multiply,
            BinaryExprType::Nullish => OpCode::NullishCoalescing,
            BinaryExprType::Addition => OpCode::Add,
            BinaryExprType::Range => OpCode::MakeRange,
        };

        self.emit_op_code(
            expression_op_code,
            (expr.opr_token.line_num, expr.opr_token.column_num),
        );
    }

    /// Compiles a ternary conditional expression.
    /// This is compiled in a similar way to how if statements are compiled.
    ///
    /// # Arguments
    /// * `expr` – A ternary conditional expression node.
    pub(super) fn compile_ternary_conditional_expr(&mut self, expr: &TernaryConditionalNode) {
        self.compile_if_stmt(&IfStmtNode {
            condition: expr.condition.clone(),
            then_token: expr.true_branch_token.clone(),
            then_branch: expr.branch_true.clone(),
            else_branch: Box::new(Some(*expr.branch_false.clone())),
            else_token: Some(expr.false_branch_token.clone()),
        });
    }

    /// Compiles an 'AND' expression.
    ///
    /// # Arguments
    /// * `expr` – A binary expression node.
    pub(super) fn compile_and_expr(&mut self, expr: &BinaryExprNode) {
        match expr.opr_type {
            BinaryExprType::LogicAND => {
                // First compile the lhs of the expression which will leave its value on the stack.
                self.compile_node(&expr.left);

                // For 'AND' expressions, if the lhs is false, then the entire expression must be false.
                // We emit a `JUMP_IF_FALSE_OR_POP` instruction to jump over the rest of this expression
                // if the lhs is falsey.
                let end_jump = self.emit_jump(OpCode::JumpIfFalseOrPop, &expr.opr_token);

                self.compile_node(&expr.right);

                // Patches the `JUMP_IF_FALSE_OR_POP` instruction above so that if the lhs is falsey, it knows
                // where the end of the expression is.
                self.patch_jump(end_jump, &expr.opr_token);
            }
            _ => unreachable!(
                "the `compile_and_expr(...)` function can only compile logical 'AND' expressions."
            ),
        }
    }

    // Compiles an 'OR' expression.
    ///
    /// # Arguments
    /// * `expr` – A binary expression node.
    pub(super) fn compile_or_expr(&mut self, expr: &BinaryExprNode) {
        match expr.opr_type {
            BinaryExprType::LogicOR => {
                // First compile the lhs of the expression which will leave its value on the stack.
                self.compile_node(&expr.left);

                // For 'OR' expressions, if the lhs is true, then the entire expression must be true.
                // We emit a `JUMP_IF_TRUE_OR_POP` instruction to jump over to the next expression
                // if the lhs is falsey.
                let end_jump = self.emit_jump(OpCode::JumpIfTrueOrPop, &expr.opr_token);

                self.compile_node(&expr.right);

                // Patches the `JUMP_IF_TRUE_OR_POP` instruction above so that if the lhs is truthy, it knows
                // where the end of the expression is.
                self.patch_jump(end_jump, &expr.opr_token);
            }
            _ => unreachable!(
                "the `compile_or_expr(...)` function can only compile logical 'OR' expressions."
            ),
        }
    }

    /// Compiles an identifier expression.
    ///
    /// # Arguments
    /// * `expr` – An identifier expression node.
    pub(super) fn compile_identifier_expr(&mut self, expr: &IdentifierExprNode) {
        if let Some(idx) = self.resolve_symbol(&expr.token, false) {
            if idx < 256 {
                self.emit_op_code(OpCode::GetVar, (expr.token.line_num, expr.token.column_num));
                self.emit_raw_byte(idx as u8, (expr.token.line_num, expr.token.column_num));
            } else {
                self.emit_op_code(
                    OpCode::GetVarLong,
                    (expr.token.line_num, expr.token.column_num),
                );
                self.emit_short(idx, (expr.token.line_num, expr.token.column_num));
            }
        }
    }

    /// Compiles a variable reassignment expression.
    ///
    /// # Arguments
    /// * `expr` – A variable reassignment expression node.
    pub(super) fn compile_var_reassignment_expr(&mut self, expr: &VarReassignmentExprNode) {
        if let Some(idx) = self.resolve_symbol(&expr.target, true) {
            let line_info = (expr.target.line_num, expr.target.column_num);

            if let ReassignmentType::None = expr.opr_type {
                // Proceed to directly reassign the variable.
                self.compile_node(&expr.value);
            } else {
                // The expression `a /= 2` expands to `a = a / 2`, so we
                // must get the variable's value onto the stack first.
                if idx < 256 {
                    self.emit_op_code(OpCode::GetVar, line_info);
                    self.emit_raw_byte(idx as u8, line_info);
                } else {
                    self.emit_op_code(OpCode::GetVarLong, line_info);
                    self.emit_short(idx, line_info);
                }

                self.compile_node(&expr.value);

                match expr.opr_type {
                    ReassignmentType::Plus => self.emit_op_code(OpCode::Add, line_info),
                    ReassignmentType::Minus => self.emit_op_code(OpCode::Subtract, line_info),
                    ReassignmentType::Div => self.emit_op_code(OpCode::Divide, line_info),
                    ReassignmentType::Mul => self.emit_op_code(OpCode::Multiply, line_info),
                    ReassignmentType::Expo => self.emit_op_code(OpCode::Expo, line_info),
                    ReassignmentType::Mod => self.emit_op_code(OpCode::Modulus, line_info),
                    ReassignmentType::ShiftL => {
                        self.emit_op_code(OpCode::BitwiseShiftLeft, line_info)
                    }
                    ReassignmentType::ShiftR => {
                        self.emit_op_code(OpCode::BitwiseShiftRight, line_info)
                    }
                    ReassignmentType::BitAnd => self.emit_op_code(OpCode::BitwiseAnd, line_info),
                    ReassignmentType::Xor => self.emit_op_code(OpCode::BitwiseXor, line_info),
                    ReassignmentType::BitOr => self.emit_op_code(OpCode::BitwiseOr, line_info),
                    ReassignmentType::None => 0,
                };
            }

            // Sets the new value (which will be on top of the stack)
            if idx < 256 {
                self.emit_op_code(OpCode::SetVar, line_info);
                self.emit_raw_byte(idx as u8, line_info);
            } else {
                self.emit_op_code(OpCode::SetVarLong, line_info);
                self.emit_short(idx, line_info);
            }
        }
    }

    /// Compiles an array literal expression.
    ///
    /// # Arguments
    /// * `expr` – A array expression node.
    pub(super) fn compile_array_expr(&mut self, expr: &ArrayExprNode) {
        if expr.values.len() <= (u16::MAX as usize) {
            // We reverse the list here because at runtime, we pop each value of the stack in the
            // opposite order (because it *is* a stack). Instead of performing that operation during
            // runtime, we execute it once during compile time.
            for node in expr.values.iter().rev() {
                self.compile_node(&node);
            }

            if expr.values.len() < 256 {
                self.emit_op_code(
                    OpCode::MakeArray,
                    (expr.token.line_num, expr.token.column_num),
                );
                self.emit_raw_byte(
                    expr.values.len() as u8,
                    (expr.token.line_num, expr.token.column_num),
                );
            } else {
                self.emit_op_code(
                    OpCode::MakeArrayLong,
                    (expr.token.line_num, expr.token.column_num),
                );
                self.emit_short(
                    expr.values.len() as u16,
                    (expr.token.line_num, expr.token.column_num),
                );
            }
        } else {
            self.error_at_token(
                &expr.token,
                CompilerError::MaxCapacity,
                "Too many values in the array.",
            );
        }
    }

    /// Compiles a tuple literal expression.
    ///
    /// # Arguments
    /// * `expr` – A tuple expression node.
    pub(super) fn compile_tuple_expr(&mut self, expr: &TupleExprNode) {
        if expr.values.len() <= (u16::MAX as usize) {
            // We reverse the list here because at runtime, we pop each value of the stack in the
            // opposite order (because it *is* a stack). Instead of performing that operation during
            // runtime, we execute it once during compile time.
            for node in expr.values.iter().rev() {
                self.compile_node(&node);
            }

            if expr.values.len() < 256 {
                self.emit_op_code(
                    OpCode::MakeTuple,
                    (expr.token.line_num, expr.token.column_num),
                );
                self.emit_raw_byte(
                    expr.values.len() as u8,
                    (expr.token.line_num, expr.token.column_num),
                );
            } else {
                self.emit_op_code(
                    OpCode::MakeTupleLong,
                    (expr.token.line_num, expr.token.column_num),
                );
                self.emit_short(
                    expr.values.len() as u16,
                    (expr.token.line_num, expr.token.column_num),
                );
            }
        } else {
            self.error_at_token(
                &expr.token,
                CompilerError::MaxCapacity,
                "Too many values in the tuple.",
            );
        }
    }

    /// Compiles an array indexing expression.
    ///
    /// # Arguments
    /// * `expr` – A array indexing expression node.
    pub(super) fn compile_array_indexing_expr(&mut self, expr: &ArrayIndexingExprNode) {
        self.compile_node(&expr.target);
        self.compile_node(&expr.index);
        self.emit_op_code(OpCode::Indexing, expr.pos);
    }

    pub(super) fn compile_function_call_expr(&mut self, expr: &FunctionCallExprNode) {
        // Compile the call's identifier
        self.compile_node(&expr.target);

        // Compile call's arguments
        for arg in expr.args.iter() {
            self.compile_node(&arg.value);
        }

        // Call the function at runtime
        self.emit_op_code(OpCode::FuncCall, expr.pos);
        self.emit_raw_byte(expr.args.len() as u8, expr.pos);
    }

    /// Looks for a symbol with the given token name in the symbol table.
    ///
    /// ## Arguments
    /// * `token` – A reference to the token (symbol name) related to the symbol.
    /// * `for_reassign` – Wether of not we are resolving the symbol for the purpose of reassignment.
    ///
    /// ## Returns
    /// * `Option<u16>` – If there were no errors with resolving the symbol, it returns the position
    /// of the symbol in the symbol table.
    fn resolve_symbol(&mut self, token: &Token, for_reassign: bool) -> Option<u16> {
        // Look for the symbol in the symbol table starting from the back.
        // We loop backwards because we want to first check if the symbol
        // exists in the current scope, then in any of the parent scopes, etc..
        for (index, symbol) in self.symbol_table.iter_mut().enumerate().rev() {
            if symbol.name == token.lexeme {
                if !symbol.is_initialized {
                    match symbol.symbol_type {
                        SymbolType::Variable => self.error_at_token(
                            &token,
                            CompilerError::Reference,
                            &format!(
                                "Cannot reference variable '{}' before it has been defined.",
                                token.lexeme
                            ),
                        ),
                        SymbolType::Constant => self.error_at_token(
                            &token,
                            CompilerError::Reference,
                            &format!(
                                "Cannot reference constant '{}' before it has been defined. ",
                                token.lexeme
                            ),
                        ),
                        SymbolType::Function => self.error_at_token(
                            &token,
                            CompilerError::Reference,
                            &format!(
                                "Cannot reference function '{}' before it has been defined.",
                                token.lexeme
                            ),
                        ),
                        // Classes, Parameters, and Enums are initialized upon declaration. Hence, unreachable here.
                        _ => unreachable!("Symbol should have been initialized by now."),
                    }

                    // Return None here because a symbol should be referenced
                    // unless it has been initialized.
                    return None;
                }

                if for_reassign {
                    match &symbol.symbol_type {
                        SymbolType::Constant => {
                            self.error_at_token(
                                token,
                                CompilerError::Reassignment,
                                "Constants are immutable.",
                            );
                            return None;
                        }
                        SymbolType::Function => {
                            self.error_at_token(
                                token,
                                CompilerError::Reassignment,
                                "Functions are immutable.",
                            );
                            return None;
                        }
                        SymbolType::Class => {
                            self.error_at_token(
                                token,
                                CompilerError::Reassignment,
                                "Classes are immutable.",
                            );
                            return None;
                        }
                        SymbolType::Enum => {
                            self.error_at_token(
                                token,
                                CompilerError::Reassignment,
                                "Enums are immutable.",
                            );
                            return None;
                        }
                        // Only variables & parameters are reassignable
                        SymbolType::Variable | SymbolType::Parameter => {}
                    }
                }

                symbol.is_used = true;
                return Some(index as u16);
            }
        }

        // Look for the identifier in the natives
        if self.natives.contains(&token.lexeme) {
            if for_reassign {
                self.error_at_token(
                    token,
                    CompilerError::Reassignment,
                    &format!("Cannot modify native function '{}'.", token.lexeme),
                );
            } else {
                self.add_literal_to_pool(Object::String(token.lexeme.clone()), token);
                self.emit_op_code(OpCode::LoadNative, (token.line_num, token.column_num));
            }

            return None;
        }

        // The symbol doesn't exist
        self.error_at_token(
            token,
            CompilerError::Reference,
            &format!("Use of undeclared identifier '{}'.", token.lexeme),
        );
        None
    }

    /// Emits a constant instruction and adds the related object to the constant pool
    ///
    /// # Arguments
    /// * `obj` – A reference to the literal object being added to the pool.
    /// * `token` – The object's original token.
    pub(super) fn add_literal_to_pool(&mut self, obj: Object, token: &Token) {
        let constant_pos = self.function.chunk.add_constant(obj);
        let opr_pos = (token.line_num, token.column_num);

        match constant_pos {
            Ok(idx) => {
                if idx < 256 {
                    self.emit_op_code(OpCode::LoadConstant, opr_pos);
                    self.emit_raw_byte(idx as u8, opr_pos);
                } else {
                    self.emit_op_code(OpCode::LoadConstantLong, opr_pos);
                    self.emit_short(idx, opr_pos);
                }
            }
            Err(_) => {
                self.error_at_token(
                    token,
                    CompilerError::MaxCapacity,
                    "Too many constants in one chunk.",
                );
            }
        }
    }
}
