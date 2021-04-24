use std::convert::TryFrom;
use std::rc::Rc;
mod expressions;
mod statements;

use crate::{
    chunk::{op_codes::OpCode, Chunk},
    lexer::tokens::{Token, TokenType},
    objects::FunctionObject,
    virtual_machine::InterpretResult,
};

use super::ast::*;

/// Represents a variable declaration. Used for lexical scoping.
pub struct Variable {
    name: Rc<Token>,
    depth: usize,
    is_initialized: bool,
    is_const: bool,
    is_used: bool,
}

/// Represents a compiler and its internal state.
pub struct Compiler {
    had_error: bool,
    is_in_panic: bool,
    chunk: Chunk,
    // Lexical scoping of declarations
    variables: Vec<Variable>,
    scope_depth: usize,
}

impl Compiler {
    /// Compiles an Abstract Syntax Tree into ByteCode.
    /// While the AST is used to perform static analysis like type checking
    /// and error detection, the ByteCode is used to execute the program faster.
    ///
    /// ## Arguments
    /// * `program` – The root node of the AST for a particular program. This contains all the statements
    /// and declarations in a program.
    ///
    /// ## Returns
    /// `Result<FunctionObject, InterpretResult>` – If the program had no compile-time errors, returns
    /// the main function object for this module. Otherwise returns an InterpretResult::INTERPRET_COMPILE_ERROR.
    pub fn compile(program: Rc<ModuleNode>) -> Result<FunctionObject, InterpretResult> {
        let mut c = Compiler {
            had_error: false,
            is_in_panic: false,
            chunk: Chunk::new(),
            variables: Vec::with_capacity(u16::MAX as usize),
            scope_depth: 0,
        };

        for node in program.body.iter() {
            if c.had_error {
                return Err(InterpretResult::INTERPRET_COMPILE_ERROR);
            }

            // TODO: What can we do so that cloning each node is no longer necessary?
            // Cloning each node is a very expensive operation because some of the nodes
            // could have an arbitrarily big amount of data. Fox example, large bodies
            // of literal text could drastically slow down the performance of the compiler
            // when those strings have to be cloned.
            c.compile_node(node.clone());
        }

        // **** TEMPORARY ****
        c.emit_op_code(OpCode::OP_RETURN, (0, 0));
        // Shows the chunk.
        // c.chunk.disassemble("<script>");
        // c.chunk.print_raw("<script>");

        if !c.had_error {
            // Return the compiled chunk.
            Ok(FunctionObject {
                chunk: c.chunk,
                min_arity: 0,
                max_arity: 0,
                name: String::from("<Script>"),
            })
        } else {
            return Err(InterpretResult::INTERPRET_COMPILE_ERROR);
        }
    }

    /// Compiles an AST node.
    ///
    /// ## Arguments
    /// * `node` – The node to be compiled.
    pub fn compile_node(&mut self, node: ASTNode) {
        return match node {
            ASTNode::Binary(x) => self.compile_binary_expr(x),
            ASTNode::BlockStmt(x) => self.compile_block_stmt(x),
            ASTNode::ConstantDecl(x) => self.compile_constant_decl(x),
            ASTNode::ExpressionStmt(x) => {
                self.compile_node(*x.child);
                self.emit_op_code(OpCode::OP_POP_STACK, x.pos);
            }
            ASTNode::Identifier(x) => self.compile_identifier_expr(x),
            ASTNode::Literal(x) => self.compile_literal(x),
            ASTNode::PrintStmt(x) => self.compile_print_stmt(x),
            ASTNode::TernaryConditional(x) => self.compile_ternary_conditional(x),
            ASTNode::Unary(x) => self.compile_unary_expr(x),
            ASTNode::VarReassignment(x) => self.compile_var_reassignment_expr(x),
            ASTNode::VariableDecl(x) => self.compile_variable_decl(x),
            ASTNode::IfStmt(x) => self.compile_if_statement(x),
        };
    }

    /// Emits a byte instruction from an OpCode into the chunk's instruction list.
    ///
    /// ## Arguments
    /// * `instr` – The OpCode instruction to added to the chunk.
    ///
    /// ## Returns
    /// * `usize` – The position of the currently emitted OpCode in the chunk.
    pub fn emit_op_code(&mut self, instr: OpCode, pos: (usize, usize)) -> usize {
        self.chunk.codes.push_byte(instr as u8);
        self.chunk.locations.push(pos.clone());

        return self.chunk.codes.len() - 1;
    }

    /// Emits a short instruction from a 16-bit integer into the chunk's instruction list.
    ///
    /// ## Arguments
    /// * `instr` – The 16-bit short instruction to added to the chunk.
    ///
    /// ## Returns
    /// * `usize` – The position of the first byte for the currently emitted 16-bit short
    /// in the chunk.
    pub fn emit_short(&mut self, instr: u16, pos: (usize, usize)) -> usize {
        self.chunk.codes.push_short(instr);
        self.chunk.locations.push(pos.clone());
        self.chunk.locations.push(pos.clone());

        return self.chunk.codes.len() - 1;
    }

    /// Emits a jump instructions with a dummy jump offset. This offset should be
    // later replaced by calling the `patch_jump(...)` function.
    ///
    /// ## Arguments
    /// * `instruction` – The jump instruction to emit to the chunk.
    /// * `token` – The token associated with this jump.
    ///
    /// ## Returns
    /// `usize` – The position of the currently emitted jump instruction. This value
    /// should be used by the call to the `patch_jump(...)` function to patch the
    /// correct jump instruction's offset.
    fn emit_jump(&mut self, instruction: OpCode, token: Rc<Token>) -> usize {
        // TODO: Emit the correct position
        self.emit_op_code(instruction, (token.line_num, token.column_num));
        // We emit a temporary short representing the jump that will be
        // made by the vm during runtime
        // TODO: Emit the correct position
        self.emit_short(0xffff, (token.line_num, token.column_num))
    }

    /// Patches the offset of a jump instruction.
    ///
    /// ## Arguments
    /// * `offset` – The position in the chunk of the jump instruction to be patched.
    /// * `token` – The token associated with this jump patch.
    fn patch_jump(&mut self, offset: usize, token: Rc<Token>) {
        // -1 to adjust for the bytecode for the jump offset itself.
        let jump = match u16::try_from((self.chunk.codes.len() - offset) - 1) {
            Ok(x) => x,
            Err(_) => {
                return self.error_at_token(token, "Too much code to jump over.");
            }
        };

        let j = jump.to_be_bytes();
        self.chunk.codes.modify_byte(offset - 1, j[0]);
        self.chunk.codes.modify_byte(offset, j[1]);
    }

    /// Emits a compiler error from the given token.
    ///
    /// ## Arguments
    /// *  `tok` – The token that caused the error.
    /// * `message` – The error message to display.
    pub(super) fn error_at_token(&mut self, tok: Rc<Token>, message: &str) {
        if self.is_in_panic {
            return ();
        }

        self.is_in_panic = true;

        print!("\x1b[31;1mSyntaxError\x1b[0m [{}:{}]", tok.line_num, tok.column_num);

        if let TokenType::EOF = tok.token_type {
            println!(" – At the end of the program.");
        } else if let TokenType::ERROR = tok.token_type {
            // Nothing...
        } else {
            print!(" at '\x1b[37;1m{}\x1b[0m' – ", tok.lexeme);
        }

        println!("{}", message);
        self.had_error = true;
    }
}
