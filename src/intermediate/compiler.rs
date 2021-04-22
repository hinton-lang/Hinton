use std::rc::Rc;

use crate::{
    chunk::{op_codes::OpCode, Chunk, ConstantPos},
    lexer::tokens::{Token, TokenType},
    objects::{FunctionObject, Object},
    virtual_machine::InterpretResult,
};

use super::ast;
use super::ast::*;

/// Represents a compiler and its internal state.
pub struct Compiler<'a> {
    had_error: bool,
    is_in_panic: bool,
    pub chunk: Chunk<'a>,
}

impl<'a> Compiler<'a> {
    /// Compiles an Abstract Syntax Tree into ByteCode.
    /// While the AST is used to perform static analysis like type checking
    /// and error detection, the ByteCode is used to execute the program faster.
    ///
    /// ## Arguments
    /// * `program` – The root node of the AST for a particular program. This contains all the statements
    /// and declarations in a program.
    ///
    /// ## Returns
    /// `Result<Chunk<'a>, InterpretResult>` – If the program had no compile-time errors, returns
    /// a compiled chunk. Otherwise returns an InterpretResult::INTERPRET_COMPILE_ERROR.
    pub fn compile(program: Rc<ModuleNode<'a>>) -> Result<FunctionObject<'a>, InterpretResult> {
        let mut c = Compiler {
            had_error: false,
            is_in_panic: false,
            chunk: Chunk::new(),
        };

        for node in program.body.iter() {
            if c.had_error {
                return Err(InterpretResult::INTERPRET_COMPILE_ERROR);
            }

            // Prints this node's AST
            // node.print(0);

            // TODO: What can we do so that cloning each node is no longer necessary?
            // Cloning each node is a very expensive operation because some of the nodes
            // could have an arbitrarily big amount of data. Fox example, large bodies
            // of literal text could drastically slow down the performance of the compiler
            // when those strings have to be cloned.
            c.compile_node(node.clone());
        }

        // Shows the chunk.
        // c.chunk.disassemble("<script>");
        // c.chunk.print_raw("<script>");
        // **** TEMPORARY ****
        c.emit_op_code(OpCode::OP_RETURN, (0, 0));

        // Return the compiled chunk.
        Ok(FunctionObject {
            chunk: c.chunk,
            min_arity: 0,
            max_arity: 0,
            name: "<Script>",
        })
    }

    /// Compiles an AST node.
    pub fn compile_node(&mut self, node: ASTNode<'a>) {
        return match node {
            ASTNode::Literal(x) => self.compile_literal(x),
            ASTNode::Binary(x) => self.compile_binary_expr(x),
            ASTNode::Unary(x) => self.compile_unary_expr(x),
            ASTNode::TernaryConditional(x) => self.compile_ternary_conditional(x),
            ASTNode::Identifier(_) => todo!("Add support for compiling identifiers."),
            ASTNode::PrintStmt(x) => self.compile_print_statement(x),
            ASTNode::ExpressionStmt(x) => {
                self.compile_node(*x.child);
                self.emit_op_code(OpCode::OP_POP_STACK, x.pos);
            },
        };
    }

    /// Compiles a binary expression
    pub fn compile_binary_expr(&mut self, expr: BinaryExprNode<'a>) {
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

    /// Compiles a unary expression
    pub fn compile_unary_expr(&mut self, expr: UnaryExprNode<'a>) {
        self.compile_node(*expr.operand);

        let expression_op_code = match expr.opr_type {
            ast::UnaryExprType::ArithmeticNeg => OpCode::OP_NEGATE,
            ast::UnaryExprType::LogicNeg => OpCode::OP_LOGIC_NOT,
            ast::UnaryExprType::BitwiseNeg => OpCode::OP_BITWISE_NOT,
        };

        self.emit_op_code(expression_op_code, expr.pos);
    }

    /// Compiles a ternary conditional expression
    pub fn compile_ternary_conditional(&mut self, expr: TernaryConditionalNode<'a>) {
        self.compile_node(*expr.condition);
        self.compile_node(*expr.branch_true);
        self.compile_node(*expr.branch_false);

        self.emit_op_code(OpCode::OP_TERNARY, expr.pos);
    }

    /// Compiles a literal expression
    pub fn compile_literal(&mut self, expr: LiteralExprNode<'a>) {
        let obj = Rc::clone(&expr.value);
        let opr_pos = (expr.token.line_num, expr.token.column_num);

        match *obj {
            Object::Bool(x) if x => self.emit_op_code(OpCode::OP_TRUE, opr_pos),
            Object::Bool(x) if !x => self.emit_op_code(OpCode::OP_FALSE, opr_pos),
            Object::Null() => self.emit_op_code(OpCode::OP_NULL, opr_pos),
            // _ => {}
            _ => self.emit_constant_instruction(obj, expr.token),
        }
    }

    pub fn compile_print_statement(&mut self, stmt: PrintStmtNode<'a>) {
        self.compile_node(*stmt.child);
        self.emit_op_code(OpCode::OP_PRINT, stmt.pos);
    }

    /// Emits a constant instruction and adds the related object to the constant pool
    pub fn emit_constant_instruction(&mut self, obj: Rc<Object<'a>>, token: Rc<Token<'a>>) {
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

    /// Emits a byte instruction from an OpCode into the chunk's instruction list.
    ///
    /// ## Arguments
    /// * `instr` – The OpCode instruction to added to the chunk.
    pub fn emit_op_code(&mut self, instr: OpCode, pos: (usize, usize)) {
        self.chunk.codes.push_byte(instr as u8);
        self.chunk.locations.push(pos.clone());
    }

    /// Emits a short instruction from a 16-bit integer into the chunk's instruction list.
    ///
    /// ## Arguments
    /// * `instr` – The 16-bit short instruction to added to the chunk.
    pub fn emit_short(&mut self, instr: u16, pos: (usize, usize)) {
        self.chunk.codes.push_short(instr);
        self.chunk.locations.push(pos.clone());
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
