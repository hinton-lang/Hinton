use super::Compiler;
use std::rc::Rc;

use crate::{
    chunk::{op_codes::OpCode, ConstantPos},
    intermediate::ast::*,
    lexer::tokens::Token,
    objects::Object,
};

impl Compiler {
    /// Compiles a literal expression.
    ///
    /// # Arguments
    /// * `expr` – A literal expression node.
    pub(super) fn compile_literal(&mut self, expr: &LiteralExprNode) {
        let obj = Rc::clone(&expr.value);
        let opr_pos = (expr.token.line_num, expr.token.column_num);

        match *obj {
            Object::Bool(x) if x => {
                self.emit_op_code(OpCode::OP_TRUE, opr_pos);
            }
            Object::Bool(x) if !x => {
                self.emit_op_code(OpCode::OP_FALSE, opr_pos);
            }
            Object::Null => {
                self.emit_op_code(OpCode::OP_NULL, opr_pos);
            }
            _ => self.add_literal_to_pool(obj, Rc::clone(&expr.token)),
        };
    }

    /// Compiles a unary expression.
    ///
    /// # Arguments
    /// * `expr` – A unary expression node.
    pub(super) fn compile_unary_expr(&mut self, expr: &UnaryExprNode) {
        self.compile_node(&expr.operand);

        let expression_op_code = match expr.opr_type {
            UnaryExprType::ArithmeticNeg => OpCode::OP_NEGATE,
            UnaryExprType::LogicNeg => OpCode::OP_LOGIC_NOT,
            UnaryExprType::BitwiseNeg => OpCode::OP_BITWISE_NOT,
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
            BinaryExprType::BitwiseAND => OpCode::OP_BITWISE_AND,
            BinaryExprType::BitwiseOR => OpCode::OP_BITWISE_OR,
            BinaryExprType::BitwiseShiftLeft => OpCode::OP_BITWISE_L_SHIFT,
            BinaryExprType::BitwiseShiftRight => OpCode::OP_BITWISE_R_SHIFT,
            BinaryExprType::BitwiseXOR => OpCode::OP_BITWISE_XOR,
            BinaryExprType::Division => OpCode::OP_DIVIDE,
            BinaryExprType::Expo => OpCode::OP_EXPO,
            BinaryExprType::LogicAND => unreachable!("The 'AND' expression should have been compiled by now."),
            BinaryExprType::LogicEQ => OpCode::OP_EQUALS,
            BinaryExprType::LogicGreaterThan => OpCode::OP_GREATER_THAN,
            BinaryExprType::LogicGreaterThanEQ => OpCode::OP_GREATER_THAN_EQ,
            BinaryExprType::LogicLessThan => OpCode::OP_LESS_THAN,
            BinaryExprType::LogicLessThanEQ => OpCode::OP_LESS_THAN_EQ,
            BinaryExprType::LogicNotEQ => OpCode::OP_NOT_EQUALS,
            BinaryExprType::LogicOR => unreachable!("The 'OR' expression should have been compiled by now."),
            BinaryExprType::Minus => OpCode::OP_SUBTRACT,
            BinaryExprType::Modulus => OpCode::OP_MODULUS,
            BinaryExprType::Multiplication => OpCode::OP_MULTIPLY,
            BinaryExprType::Nullish => OpCode::OP_NULLISH_COALESCING,
            BinaryExprType::Addition => OpCode::OP_ADD,
            BinaryExprType::Range => OpCode::OP_GENERATE_RANGE,
        };

        self.emit_op_code(expression_op_code, (expr.opr_token.line_num, expr.opr_token.column_num));
    }

    /// Compiles a ternary conditional expression.
    /// This is compiled in a similar way to how if statements are compiled.
    ///
    /// # Arguments
    /// * `expr` – A ternary conditional expression node.
    pub(super) fn compile_ternary_conditional(&mut self, expr: &TernaryConditionalNode) {
        self.compile_node(&expr.condition);

        // Compile the `true` branch of the ternary.
        let then_jump = self.emit_jump(OpCode::OP_JUMP_IF_FALSE, Rc::clone(&expr.true_branch_token));
        self.emit_op_code(OpCode::OP_POP_STACK, (expr.true_branch_token.line_num, expr.true_branch_token.column_num));
        self.compile_node(&expr.branch_true);

        // At this point, if the condition is true, this instruction makes sure we jump
        // over all the instructions related to the `false` branch of the ternary.
        let else_jump = self.emit_jump(OpCode::OP_JUMP, Rc::clone(&expr.false_branch_token));

        // Patches the `then_jump`, so that if the condition is false, the `OP_JUMP_IF_FALSE`
        // instruction above knows where the `false` branch of the ternary starts.
        self.patch_jump(then_jump, Rc::clone(&expr.true_branch_token));

        // Compiles the `false` branch of the ternary
        self.emit_op_code(OpCode::OP_POP_STACK, (expr.true_branch_token.line_num, expr.true_branch_token.column_num));
        self.compile_node(&expr.branch_false);

        // Patches the `then_jump`, so that if the condition is true, the `OP_JUMP`
        // instruction above knows where the end of the ternary expression is.
        self.patch_jump(else_jump, Rc::clone(&expr.false_branch_token));
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
                // We emit a `OP_JUMP_IF_FALSE` instruction to jump over the rest of this expression
                // if the lhs is falsey.
                let end_jump = self.emit_jump(OpCode::OP_JUMP_IF_FALSE, Rc::clone(&expr.opr_token));

                // If the lhs is not false, the we pop that value off the stack, and continue to execute the
                // expressions in the rhs.
                self.emit_op_code(OpCode::OP_POP_STACK, (expr.opr_token.line_num, expr.opr_token.column_num));
                self.compile_node(&expr.right);

                // Patches the `OP_JUMP_IF_FALSE` instruction above so that if the lhs is falsey, it knows
                // where the end of the expression is.
                self.patch_jump(end_jump, Rc::clone(&expr.opr_token));
            }
            _ => unreachable!("the `compile_and_expr(...)` function can only compile logical 'AND' expressions."),
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
                // We emit a `OP_JUMP_IF_FALSE` instruction to jump over to the next expression if the lhs is falsey.
                let else_jump = self.emit_jump(OpCode::OP_JUMP_IF_FALSE, Rc::clone(&expr.opr_token));

                // If the lhs is truthy, then we skip over the rest of this expression.
                let end_jump = self.emit_jump(OpCode::OP_JUMP, Rc::clone(&expr.opr_token));

                // Patches the 'else_jump' so that is the lhs is falsey, the `OP_JUMP_IF_FALSE` instruction
                // above knows where the starts of the next expression is.
                self.patch_jump(else_jump, Rc::clone(&expr.opr_token));
                self.emit_op_code(OpCode::OP_POP_STACK, (expr.opr_token.line_num, expr.opr_token.column_num));
                self.compile_node(&expr.right);

                // Patches the 'end_jump' so that if the lhs is truthy, then the `OP_JUMP` instruction above
                // knows where the end of the entire expression is.
                self.patch_jump(end_jump, Rc::clone(&expr.opr_token));
            }
            _ => unreachable!("the `compile_or_expr(...)` function can only compile logical 'AND' expressions."),
        }
    }

    /// Compiles an identifier expression.
    ///
    /// # Arguments
    /// * `expr` – An identifier expression node.
    pub(super) fn compile_identifier_expr(&mut self, expr: &IdentifierExprNode) {
        match self.resolve_variable(Rc::clone(&expr.token), false) {
            Some(idx) => {
                self.emit_op_code(OpCode::OP_GET_VAR, (expr.token.line_num, expr.token.column_num));
                self.emit_short(idx, (expr.token.line_num, expr.token.column_num));
            }
            None => {}
        }
    }

    /// Compiles a variable reassignment expression.
    ///
    /// # Arguments
    /// * `expr` – A variable reassignment expression node.
    pub(super) fn compile_var_reassignment_expr(&mut self, expr: &VarReassignmentExprNode) {
        self.compile_node(&expr.value);

        match self.resolve_variable(Rc::clone(&expr.target), true) {
            Some(idx) => {
                self.emit_op_code(OpCode::OP_SET_VAR, (expr.target.line_num, expr.target.column_num));
                self.emit_short(idx, (expr.target.line_num, expr.target.column_num));
            }
            None => {}
        }
    }

    /// Compiles a post-increment expression.
    ///
    /// # Arguments
    /// * `expr` – A post-increment expression node.
    pub(super) fn compile_post_increment_expr(&mut self, expr: &PostIncrementExprNode) {
        match self.resolve_variable(Rc::clone(&expr.target), false) {
            Some(idx) => {
                self.emit_op_code(OpCode::OP_POST_INCREMENT, (expr.token.line_num, expr.token.column_num));
                self.emit_short(idx, (expr.token.line_num, expr.token.column_num));
            }
            None => {}
        }
    }

        /// Compiles a post-decrement expression.
    ///
    /// # Arguments
    /// * `expr` – A post-decrement expression node.
    pub(super) fn compile_post_decrement_expr(&mut self, expr: &PostDecrementExprNode) {
        match self.resolve_variable(Rc::clone(&expr.target), false) {
            Some(idx) => {
                self.emit_op_code(OpCode::OP_POST_DECREMENT, (expr.token.line_num, expr.token.column_num));
                self.emit_short(idx, (expr.token.line_num, expr.token.column_num));
            }
            None => {}
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

            self.emit_op_code(OpCode::OP_ARRAY, (expr.token.line_num, expr.token.column_num));
            self.emit_short(expr.values.len() as u16, (expr.token.line_num, expr.token.column_num));
        } else {
            self.error_at_token(Rc::clone(&expr.token), "Too many values in the array.");
        }
    }

    /// Looks for a variable with the given token name in the list of variables.
    ///
    /// ## Arguments
    /// * `token` – A reference to the token (variable name) related to the variable.
    /// * `for_reassign` – Wether of not we are resolving the variable for the purpose of reassignment.
    ///
    /// ## Returns
    /// * `Option<u16>` – If there were no errors with resolving the variable, it returns the position
    /// of the variable in the variables array.
    fn resolve_variable(&mut self, token: Rc<Token>, for_reassign: bool) -> Option<u16> {
        // Look for the variable in the locals array starting from the back.
        // We loop backwards because we want to first check if the variable
        // exists in the current scope, then in any of the parent scopes, etc..
        for (index, var) in self.variables.iter_mut().enumerate().rev() {
            if var.name.lexeme == token.lexeme {
                if !var.is_initialized {
                    if !var.is_const {
                        self.error_at_token(token, "Cannot read variable in its own initializer.");
                    } else {
                        self.error_at_token(token, "Cannot read constant in its own initializer.");
                    }
                    return None;
                }

                if for_reassign && var.is_const {
                    self.error_at_token(token, "Cannot reassign to constant.");
                    return None;
                }

                var.is_used = true;
                return Some(index as u16);
            }
        }

        // The variable doesn't exist
        self.error_at_token(token, "Use of undeclared identifer.");
        None
    }

    /// Emits a constant instruction and adds the related object to the constant pool
    ///
    /// # Arguments
    /// * `obj` – A reference to the literal object being added to the pool.
    /// * `token` – The object's original token.
    pub(super) fn add_literal_to_pool(&mut self, obj: Rc<Object>, token: Rc<Token>) {
        let constant_pos = self.chunk.add_constant(obj);
        let opr_pos = (token.line_num, token.column_num);

        match constant_pos {
            ConstantPos::Pos(idx) => {
                self.emit_op_code(OpCode::OP_LOAD_VALUE, opr_pos);
                self.emit_short(idx, opr_pos);
            }
            ConstantPos::Error => {
                self.error_at_token(Rc::clone(&token), "Too many constants in one chunk.");
            }
        }
    }
}
