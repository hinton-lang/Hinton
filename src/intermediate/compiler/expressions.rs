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
    pub(super) fn compile_literal(&mut self, expr: LiteralExprNode) {
        let obj = Rc::clone(&expr.value);
        let opr_pos = (expr.token.line_num, expr.token.column_num);

        match *obj {
            Object::Bool(x) if x => self.emit_op_code(OpCode::OP_TRUE, opr_pos),
            Object::Bool(x) if !x => self.emit_op_code(OpCode::OP_FALSE, opr_pos),
            Object::Null => self.emit_op_code(OpCode::OP_NULL, opr_pos),
            _ => self.add_literal_to_pool(obj, expr.token),
        }
    }

    /// Compiles a unary expression.
    ///
    /// # Arguments
    /// * `expr` – A unary expression node.
    pub(super) fn compile_unary_expr(&mut self, expr: UnaryExprNode) {
        self.compile_node(*expr.operand);

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
    pub(super) fn compile_binary_expr(&mut self, expr: BinaryExprNode) {
        self.compile_node(*expr.left);
        self.compile_node(*expr.right);

        let expression_op_code = match expr.opr_type {
            BinaryExprType::BitwiseAND => OpCode::OP_BITWISE_AND,
            BinaryExprType::BitwiseOR => OpCode::OP_BITWISE_OR,
            BinaryExprType::BitwiseShiftLeft => OpCode::OP_BITWISE_L_SHIFT,
            BinaryExprType::BitwiseShiftRight => OpCode::OP_BITWISE_R_SHIFT,
            BinaryExprType::BitwiseXOR => OpCode::OP_BITWISE_XOR,
            BinaryExprType::Division => OpCode::OP_DIVIDE,
            BinaryExprType::Expo => OpCode::OP_EXPO,
            BinaryExprType::LogicAND => return (),
            BinaryExprType::LogicEQ => OpCode::OP_EQUALS,
            BinaryExprType::LogicGreaterThan => OpCode::OP_GREATER_THAN,
            BinaryExprType::LogicGreaterThanEQ => OpCode::OP_GREATER_THAN_EQ,
            BinaryExprType::LogicLessThan => OpCode::OP_LESS_THAN,
            BinaryExprType::LogicLessThanEQ => OpCode::OP_LESS_THAN_EQ,
            BinaryExprType::LogicNotEQ => OpCode::OP_NOT_EQUALS,
            BinaryExprType::LogicOR => return (),
            BinaryExprType::Minus => OpCode::OP_SUBTRACT,
            BinaryExprType::Modulus => OpCode::OP_MODULUS,
            BinaryExprType::Multiplication => OpCode::OP_MULTIPLY,
            BinaryExprType::Nullish => OpCode::OP_NULLISH_COALESCING,
            BinaryExprType::Addition => OpCode::OP_ADD,
            BinaryExprType::Range => OpCode::OP_GENERATE_RANGE,
        };

        self.emit_op_code(expression_op_code, expr.pos);
    }

    /// Compiles a ternary conditional expression.
    ///
    /// # Arguments
    /// * `expr` – A ternary conditional expression node.
    pub(super) fn compile_ternary_conditional(&mut self, expr: TernaryConditionalNode) {
        self.compile_node(*expr.condition);
        self.compile_node(*expr.branch_true);
        self.compile_node(*expr.branch_false);

        self.emit_op_code(OpCode::OP_TERNARY, expr.pos);
    }

    /// Compiles an identifier expression.
    ///
    /// # Arguments
    /// * `expr` – An identifier expression node.
    pub(super) fn compile_identifier_expr(&mut self, expr: IdentifierExprNode) {
        let arg = self.add_identifier_to_pool(Rc::clone(&expr.token));

        match arg {
            ConstantPos::Pos(x) => {
                self.emit_op_code(OpCode::OP_GET_GLOBAL_VAR, (expr.token.line_num, expr.token.column_num));
                self.emit_short(x, (expr.token.line_num, expr.token.column_num));
            }
            ConstantPos::Error => {
                self.error_at_token(expr.token, "Could not add variable name to constant pool.");
            }
        }
    }

    /// Compiles a variable reassignment expression.
    ///
    /// # Arguments
    /// * `expr` – A variable reassignment expression node.
    pub(super) fn compile_var_reassignment_expr(&mut self, expr: VarReassignmentExprNode) {
        self.compile_node(*expr.value);

        let arg = self.add_identifier_to_pool(Rc::clone(&expr.target));

        match arg {
            ConstantPos::Pos(x) => {
                self.emit_op_code(OpCode::OP_SET_GLOBAL_VAR, (expr.target.line_num, expr.target.column_num));
                self.emit_short(x, (expr.target.line_num, expr.target.column_num));
            }
            ConstantPos::Error => {
                self.error_at_token(expr.target, "Could not add variable name to constant pool.");
            }
        }
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
                self.emit_op_code(OpCode::OP_CONSTANT, opr_pos);
                self.emit_short(idx, opr_pos);
            }
            ConstantPos::Error => {
                self.error_at_token(Rc::clone(&token), "Too many constants in one chunk.");
            }
        }
    }
}
