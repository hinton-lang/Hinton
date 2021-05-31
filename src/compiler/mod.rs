use self::symbols::{Symbol, SymbolTable, SymbolType};
use crate::{
    ast::*,
    bytecode::{self, Chunk},
    errors::CompilerErrorType,
    errors::ErrorReport,
    lexer::tokens::Token,
    objects::FuncObject,
};
use std::{convert::TryFrom, str, vec};

// Submodules
mod expressions;
mod functions;
mod loops;
mod resolver;
mod statements;
mod symbols;

/// Represents a break statement, which is associated with a loop.
struct BreakScope {
    parent_loop: LoopScope,
    loop_position: usize,
}

/// Represents a loop statement at compile time. Used primarily by
/// break statements to know which loops to break.
#[derive(Clone, Copy)]
struct LoopScope {
    /// The position of the loop's start in the chunk
    position: usize,
    /// The scope depth of the body of the loop
    scope_depth: usize,
    /// The type of loop this represents
    loop_type: LoopType,
}

/// The types of loops available in Hinton. This is useful when \
/// compiling break statements to emit extra `POP` operations \
/// based on the loop type.
#[derive(Clone, Copy)]
enum LoopType {
    ForIn,
    While,
}

/// Types of compilers we can create
#[derive(Clone)]
enum CompilerType {
    Function,
    Script,
}

pub struct FunctionScope {
    function: FuncObject,
    // Lexical scoping of declarations
    s_table: SymbolTable,
    scope_depth: usize,
    // Used to match the break statements with the correct
    // loop (specially useful with nested loops), and to
    // patch the break (OP_JUMP) offsets.
    loops: Vec<LoopScope>,
    breaks: Vec<BreakScope>,
}

/// Represents a compiler and its internal state.
pub struct Compiler {
    compiler_type: CompilerType,
    functions: Vec<FunctionScope>,
    filepath: String,
    globals: SymbolTable,
    errors: Vec<ErrorReport>,
}

impl Compiler {
    /// Compiles an Abstract Syntax Tree into ByteCode.
    /// While the AST is used to perform static analysis like type checking
    /// and error detection, the ByteCode is used to execute the program faster.
    ///
    /// ## Arguments
    /// * `filepath` – The program's filepath.
    /// * `program` – The root node of the AST for a particular program.
    ///
    /// ## Returns
    /// `Result<chunk, InterpretResult>` – If the program had no compile-time errors, returns
    /// the main chunk for this module. Otherwise returns an InterpretResult::INTERPRET_COMPILE_ERROR.
    pub fn compile_file(filepath: &str, program: &ASTNode) -> Result<FuncObject, Vec<ErrorReport>> {
        // The first element in a symbol table is always the symbol representing
        // the function to which the symbol table belongs.
        let symbols = SymbolTable {
            symbols: vec![Symbol {
                name: format!("<File '{}'>", filepath),
                symbol_type: SymbolType::Function,
                is_initialized: true,
                symbol_depth: 0,
                is_used: true,
                line_info: (0, 0),
                is_global: true,
            }],
        };

        let base_fn = FunctionScope {
            function: FuncObject {
                defaults: vec![],
                min_arity: 0,
                max_arity: 0,
                chunk: bytecode::Chunk::new(),
                name: format!("<File '{}'>", filepath),
            },
            s_table: symbols,
            scope_depth: 0,
            loops: vec![],
            breaks: vec![],
        };

        let mut _self = Compiler {
            compiler_type: CompilerType::Script,
            functions: vec![base_fn],
            filepath: String::from(filepath),
            errors: vec![],
            globals: SymbolTable { symbols: vec![] },
        };

        // Compile the function body
        _self.compile_node(&program);
        _self.end_function();

        if _self.errors.len() == 0 {
            Ok(std::mem::take(&mut _self.current_func_scope_mut().function))
        } else {
            return Err(_self.errors);
        }
    }

    fn end_function(&mut self) {
        // TODO: Emit the correct position
        self.emit_op_code(bytecode::OpCode::LoadImmNull, (0, 0));
        self.emit_op_code(bytecode::OpCode::Return, (0, 0));

        // The number of local symbols that need to be popped off the stack
        let num_of_symbols = if self.functions.len() == 1 {
            0
        } else {
            self.current_function_scope().s_table.len() - 1
        };

        self.emit_raw_byte(num_of_symbols as u8, (0, 0));

        // Shows the chunk.
        #[cfg(feature = "show_bytecode")]
        bytecode::disassemble_chunk(
            self.current_chunk(),
            self.current_function_scope().function.name.clone().as_str(),
        );
        #[cfg(feature = "show_raw_bytecode")]
        bytecode::print_raw(
            self.current_chunk(),
            self.current_function_scope().function.name.clone().as_str(),
        );
    }

    /// Compiles an AST node.
    ///
    /// ## Arguments
    /// * `node` – The node to be compiled.
    fn compile_node(&mut self, node: &ASTNode) {
        return match node {
            ASTNode::Array(x) => self.compile_array_expr(x),
            ASTNode::ArrayIndexing(x) => self.compile_array_indexing_expr(x),
            ASTNode::Binary(x) => self.compile_binary_expr(x),
            ASTNode::BlockStmt(x) => self.compile_block_stmt(x),
            ASTNode::BreakStmt(x) => self.compile_break_stmt(x),
            ASTNode::ConstantDecl(x) => self.compile_constant_decl(x),
            ASTNode::ExpressionStmt(x) => self.compile_expression_stmt(x),
            ASTNode::ForStmt(x) => self.compile_for_stmt(x),
            ASTNode::FunctionCallExpr(x) => self.compile_function_call_expr(x),
            ASTNode::FunctionDecl(x) => self.compile_function_decl(x),
            ASTNode::Identifier(x) => self.compile_identifier_expr(x),
            ASTNode::IfStmt(x) => self.compile_if_stmt(x),
            ASTNode::Literal(x) => self.compile_literal_expr(x),
            ASTNode::Module(x) => self.compile_module_node(x),
            ASTNode::ReturnStmt(x) => self.compile_return_stmt(x),
            ASTNode::TernaryConditional(x) => self.compile_ternary_conditional_expr(x),
            ASTNode::Tuple(x) => self.compile_tuple_expr(x),
            ASTNode::Unary(x) => self.compile_unary_expr(x),
            ASTNode::VarReassignment(x) => self.compile_var_reassignment_expr(x),
            ASTNode::VariableDecl(x) => self.compile_variable_decl(x),
            ASTNode::WhileStmt(x) => self.compile_while_stmt(x),
        };
    }

    fn compile_module_node(&mut self, module: &ModuleNode) {
        for node in module.body.iter() {
            self.compile_node(node);
        }
    }

    fn is_global_scope(&self) -> bool {
        if self.functions.len() == 1 && self.current_function_scope().scope_depth == 0 {
            if let CompilerType::Script = self.compiler_type {
                return true;
            }
        }

        false
    }

    fn current_function_scope(&self) -> &FunctionScope {
        self.functions.last().unwrap()
    }

    fn current_func_scope_mut(&mut self) -> &mut FunctionScope {
        self.functions.last_mut().unwrap()
    }

    fn current_chunk(&self) -> &Chunk {
        &self.current_function_scope().function.chunk
    }

    fn current_chunk_mut(&mut self) -> &mut Chunk {
        &mut self.current_func_scope_mut().function.chunk
    }

    fn relative_scope_depth(&self) -> usize {
        self.current_function_scope().scope_depth
    }

    /// Emits a byte instruction from an OpCode into the chunk's instruction list.
    ///
    /// ## Arguments
    /// * `instr` – The OpCode instruction to added to the chunk.
    ///
    /// ## Returns
    /// * `usize` – The position of the currently emitted OpCode in the chunk.
    fn emit_op_code(&mut self, instr: bytecode::OpCode, pos: (usize, usize)) -> usize {
        self.current_chunk_mut().push_byte(instr as u8);
        self.current_chunk_mut().push_line_info(pos.clone());

        return self.current_chunk_mut().len() - 1;
    }

    /// Emits a raw byte instruction into the chunk's instruction list.
    ///
    /// ## Arguments
    /// * `byte` – The byte instruction to added to the chunk.
    ///
    /// ## Returns
    /// * `usize` – The position of the currently emitted OpCode in the chunk.
    fn emit_raw_byte(&mut self, byte: u8, pos: (usize, usize)) -> usize {
        self.current_chunk_mut().push_byte(byte);
        self.current_chunk_mut().push_line_info(pos.clone());

        return self.current_chunk_mut().len() - 1;
    }

    /// Emits a short instruction from a 16-bit integer into the chunk's instruction list.
    ///
    /// ## Arguments
    /// * `instr` – The 16-bit short instruction to added to the chunk.
    ///
    /// ## Returns
    /// * `usize` – The position of the first byte for the currently emitted 16-bit short
    /// in the chunk.
    fn emit_short(&mut self, instr: u16, pos: (usize, usize)) -> usize {
        self.current_chunk_mut().push_short(instr);
        self.current_chunk_mut().push_line_info(pos.clone());
        self.current_chunk_mut().push_line_info(pos.clone());

        return self.current_chunk_mut().len() - 2;
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
    fn emit_jump(&mut self, instruction: bytecode::OpCode, token: &Token) -> usize {
        self.emit_op_code(instruction, (token.line_num, token.column_num));
        // We emit a temporary short representing the jump that will be
        // made by the vm during runtime
        self.emit_short(0xffff, (token.line_num, token.column_num))
    }

    /// Patches the offset of a jump instruction.
    ///
    /// ## Arguments
    /// * `offset` – The position in the chunk of the jump instruction to be patched.
    /// * `token` – The token associated with this jump patch.
    fn patch_jump(&mut self, offset: usize, token: &Token) {
        // -2 to adjust for the bytecode for the jump offset itself.
        match u16::try_from(self.current_chunk_mut().len() - offset - 2) {
            Ok(forward) => {
                let jump = forward.to_be_bytes();

                self.current_chunk_mut().modify_byte(offset, jump[0]);
                self.current_chunk_mut().modify_byte(offset + 1, jump[1]);
            }
            Err(_) => {
                return self.error_at_token(
                    token,
                    CompilerErrorType::MaxCapacity,
                    "Too much code to jump over.",
                );
            }
        }
    }

    /// Patches the offset of a jump instruction.
    ///
    /// ## Arguments
    /// * `loop_start` – The position in the chunk of the jump instruction to be patched.
    /// * `token` – The token associated with this jump patch.
    fn emit_loop(&mut self, loop_start: usize, token: &Token) {
        let offset = self.current_chunk_mut().len() - loop_start;

        if offset < (u8::MAX - 2) as usize {
            self.emit_op_code(
                bytecode::OpCode::LoopJump,
                (token.line_num, token.column_num),
            );

            // +2 to account for the 'OP_LOOP_JUMP' and its operand.
            let jump = (offset + 2) as u8;
            self.emit_raw_byte(jump, (token.line_num, token.column_num));
        } else if offset < (u16::MAX - 3) as usize {
            self.emit_op_code(
                bytecode::OpCode::LoopJumpLong,
                (token.line_num, token.column_num),
            );

            // +3 to account for the 'OP_LOOP_JUMP_LONG' and its operands.
            let jump = (offset + 3) as u16;
            self.emit_short(jump, (token.line_num, token.column_num));
        } else {
            return self.error_at_token(
                token,
                CompilerErrorType::MaxCapacity,
                "Loop body too large.",
            );
        }
    }

    /// Emits a compiler error from the given token.
    ///
    /// ## Arguments
    /// *  `tok` – The token that caused the error.
    /// * `message` – The error message to display.
    fn error_at_token(&mut self, token: &Token, err_type: CompilerErrorType, message: &str) {
        let err_name = match err_type {
            CompilerErrorType::MaxCapacity => "MaxCapacityError",
            CompilerErrorType::Reassignment => "ReassignmentError",
            CompilerErrorType::Reference => "ReferenceError",
            CompilerErrorType::Syntax => "SyntaxError",
            CompilerErrorType::Duplication => "DuplicationError",
        };

        let msg = format!(
            "\x1b[31;1m{}\x1b[0m\x1b[1m at [{}:{}]: {}\x1b[0m",
            err_name, token.line_num, token.column_num, message
        );

        self.errors.push(ErrorReport {
            line: token.line_num,
            column: token.column_num,
            lexeme_len: token.lexeme.len(),
            message: msg,
        });
    }
}
