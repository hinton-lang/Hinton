pub mod ast;
pub mod expressions;
pub mod parser;

use std::rc::Rc;

use crate::{
    chunk::{op_codes::OpCode, Chunk, ConstantPos},
    lexer::tokens::{Token, TokenType},
    objects::Object,
};

use self::{
    ast::{ASTNode, BinaryExprNode, BinaryExprType, LiteralExprNode, UnaryExprNode},
    parser::parse,
};

/// Represents a compiler and its internal state.
pub struct Compiler<'a> {
    had_error: bool,
    is_in_panic: bool,
    pub chunk: Chunk<'a>,
}

impl<'a> Compiler<'a> {
    pub fn compile(src: &'a str) {
        let tree: Vec<ASTNode<'a>> = parse(src);

        let mut c = Compiler {
            had_error: false,
            is_in_panic: false,
            chunk: Chunk::new(),
        };

        // iterate over the tree
        for item in tree.iter() {
            item.print(0);
            c.compile_node(item);

            if c.had_error {
                break;
            }
        }

        c.chunk.disassemble("<Script>");
    }

    /// Compiles an AST node.
    pub fn compile_node(&mut self, node: &'a ASTNode) {
        return match node {
            ASTNode::Literal(x) => self.compile_literal(x),
            ASTNode::Binary(x) => self.compile_binary_expr(x),
            ASTNode::Unary(x) => self.compile_unary_expr(x),
        };
    }

    /// Compiles a binary expression
    pub fn compile_binary_expr(&mut self, expr: &'a BinaryExprNode) {
        self.compile_node(&expr.left);
        self.compile_node(&expr.right);
        let opr_pos = (expr.token.line_num, expr.token.column_num);

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

        self.emit_op_code(expression_op_code, opr_pos);
    }

    pub fn compile_unary_expr(&mut self, expr: &'a UnaryExprNode) {
        self.compile_node(&expr.operand);
        let opr_pos = (expr.token.line_num, expr.token.column_num);

        let expression_op_code = match expr.opr_type {
            ast::UnaryExprType::ArithmeticNeg => OpCode::OP_NEGATE,
            ast::UnaryExprType::LogicNeg => OpCode::OP_LOGIC_NOT,
            ast::UnaryExprType::BitwiseNeg => OpCode::OP_BITWISE_NOT,
        };

        self.emit_op_code(expression_op_code, opr_pos);
    }

    /// Compiles a literal expression
    pub fn compile_literal(&mut self, expr: &'a LiteralExprNode) {
        let obj = expr.value.clone();
        self.emit_constant_instruction(obj, expr.token.clone());
    }

    /// Emits a constant instruction and adds the related object to the constant pool
    pub fn emit_constant_instruction(&mut self, obj: Rc<Object<'a>>, token: Rc<Token<'a>>) {
        let constant_pos = self.chunk.add_constant(obj);

        match constant_pos {
            ConstantPos::Pos(pos) => {
                self.emit_op_code(OpCode::OP_CONSTANT, (token.line_num, token.column_num));
                self.emit_short(pos, (token.line_num, token.column_num));
            }
            ConstantPos::Error => {
                self.error_at_token(token.clone(), "Too many constants in one chunk.");
            }
        }
    }

    /// Emits a byte instruction from an OpCode into the chunk's instruction list.
    ///
    /// ## Arguments
    /// * `instr` – The OpCode instruction to added to the chunk.
    pub fn emit_op_code(&mut self, instr: OpCode, pos: (usize, usize)) {
        self.chunk.codes.push_byte(instr as u8);
        self.chunk.locations.push(pos);
    }

    /// Emits a short instruction from a 16-bit integer into the chunk's instruction list.
    ///
    /// ## Arguments
    /// * `instr` – The 16-bit short instruction to added to the chunk.
    pub fn emit_short(&mut self, instr: u16, pos: (usize, usize)) {
        self.chunk.codes.push_short(instr);
        self.chunk.locations.push(pos);
    }

    /// Emits a compiler error from the given token.
    ///
    /// ## Arguments
    /// *  `tok` – The token that caused the error.
    /// * `message` – The error message to display.
    pub(super) fn error_at_token(&mut self, tok: Rc<Token<'a>>, message: &str) {
        if self.is_in_panic {
            return ();
        }
        self.is_in_panic = true;

        print!("SyntaxError [{}:{}]", tok.line_num, tok.column_num);

        if let TokenType::EOF = tok.token_type {
            println!(" – At the end of the program.");
        } else if let TokenType::ERROR = tok.token_type {
            // Nothing...
        } else {
            print!(" at '{}' – ", tok.lexeme);
        }

        println!("{}", message);
        self.had_error = true;
    }
}
