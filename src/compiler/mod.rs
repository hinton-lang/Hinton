use crate::{
    ast::*, bytecode, errors::CompilerErrorType, errors::ErrorReport, lexer::tokens::Token,
    objects::FunctionObject,
};
use std::{convert::TryFrom, fmt, fmt::Display, str, vec};

// Submodules
mod expressions;
mod functions;
mod loops;
mod statements;

/// Types of symbols available in Hinton.
enum SymbolType {
    Variable,
    Constant,
    Function,
    Class,
    Enum,
    Parameter,
}

/// Represents a symbol. Used for lexical scoping.
struct Symbol {
    name: String,
    symbol_depth: usize,
    symbol_type: SymbolType,
    is_initialized: bool,
    is_used: bool,
    pos: (usize, usize),
}

impl Display for Symbol {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        let s = format!(
            "(name: {}, scope: {})",
            self.name.as_str(),
            self.symbol_depth
        );
        fmt::Debug::fmt(s.as_str(), f)
    }
}

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

/// Represents a compiler and its internal state.
pub struct Compiler {
    compiler_type: CompilerType,
    function: FunctionObject,
    filepath: String,
    // Lexical scoping of declarations
    symbol_table: Vec<Symbol>,
    scope_depth: usize,
    // Used to match the break statements with the correct
    // loop (specially useful with nested loops), and to
    // patch the break (OP_JUMP) offsets.
    loops: Vec<LoopScope>,
    breaks: Vec<BreakScope>,
    // A list of errors generated when compiling the AST
    errors: Vec<ErrorReport>,
}

/// Types of compilers we can create
enum CompilerType {
    Function,
    Script,
}

impl Compiler {
    /// Compiles an Abstract Syntax Tree into ByteCode.
    /// While the AST is used to perform static analysis like type checking
    /// and error detection, the ByteCode is used to execute the program faster.
    ///
    /// ## Arguments
    /// * `filepath` – The program's filepath.
    /// * `natives` – The native functions.
    /// * `program` – The root node of the AST for a particular program.
    ///
    /// ## Returns
    /// `Result<chunk, InterpretResult>` – If the program had no compile-time errors, returns
    /// the main chunk for this module. Otherwise returns an InterpretResult::INTERPRET_COMPILE_ERROR.
    pub fn compile_file(
        filepath: &str,
        program: &ASTNode,
    ) -> Result<FunctionObject, Vec<ErrorReport>> {
        let base_fn = FunctionObject {
            defaults: vec![],
            min_arity: 0,
            max_arity: 0,
            chunk: bytecode::Chunk::new(),
            name: format!("<File '{}'>", filepath),
        };

        let symbols = vec![Symbol {
            name: format!("<File '{}'>", filepath),
            symbol_depth: 0,
            symbol_type: SymbolType::Function,
            is_initialized: true,
            is_used: true,
            pos: (0, 0),
        }];

        let mut _self = Compiler {
            compiler_type: CompilerType::Script,
            function: base_fn,
            symbol_table: symbols,
            scope_depth: 0,
            loops: vec![],
            breaks: vec![],
            filepath: String::from(filepath),
            errors: vec![],
        };

        // Compile the function body
        _self.compile_node(&program);
        // ends the compiler.
        _self.end_compiler();

        if _self.errors.len() == 0 {
            Ok(_self.function)
        } else {
            return Err(_self.errors);
        }
    }

    /// Compiles an Abstract Syntax Tree into ByteCode.
    /// While the AST is used to perform static analysis like type checking
    /// and error detection, the ByteCode is used to execute the program faster.
    ///
    /// ## Arguments
    /// * `Func` – The function declaration node to be compiled into a chunk.
    ///
    /// ## Returns
    /// `Result<chunk, InterpretResult>` – If the program had no compile-time errors, returns
    /// the main chunk for this module. Otherwise returns an InterpretResult::INTERPRET_COMPILE_ERROR.
    pub fn compile_function(
        filepath: String,
        func: &FunctionDeclNode,
    ) -> Result<FunctionObject, Vec<ErrorReport>> {
        let base_fn = FunctionObject {
            defaults: vec![],
            min_arity: func.min_arity,
            max_arity: func.max_arity,
            chunk: bytecode::Chunk::new(),
            name: func.name.lexeme.clone(),
        };

        let symbols = vec![Symbol {
            name: func.name.lexeme.clone(),
            symbol_depth: 0,
            symbol_type: SymbolType::Function,
            is_initialized: true,
            is_used: true,
            pos: (func.name.line_num, func.name.column_num),
        }];

        let mut _self = Compiler {
            compiler_type: CompilerType::Function,
            function: base_fn,
            symbol_table: symbols,
            scope_depth: 0,
            loops: vec![],
            breaks: vec![],
            filepath,
            errors: vec![],
        };

        // compiles the parameter declarations so that the compiler
        // knows about their their lexical scoping (their stack position),
        // but does not compile the default value for named parameters.
        _self.compile_parameters(&func.params);
        // Compile the function body
        _self.compile_node(&func.body);
        // ends the compiler.
        _self.end_compiler();

        if _self.errors.len() == 0 {
            Ok(_self.function)
        } else {
            return Err(_self.errors);
        }
    }

    fn end_compiler(&mut self) {
        // TODO: Emit the correct position
        self.emit_op_code(bytecode::OpCode::LoadImmNull, (0, 0));
        self.emit_op_code(bytecode::OpCode::Return, (0, 0));

        // The number of local symbols that need to be popped off the stack
        let num_of_symbols = self.symbol_table.len() - 1;
        self.emit_raw_byte(num_of_symbols as u8, (0, 0));

        // Shows the chunk.
        #[cfg(feature = "show_bytecode")]
        bytecode::disassemble_chunk(&self.function.chunk, self.function.name.as_str());
        #[cfg(feature = "show_raw_bytecode")]
        bytecode::print_raw(&self.function.chunk, self.function.name.as_str());
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

    /// Emits a byte instruction from an OpCode into the chunk's instruction list.
    ///
    /// ## Arguments
    /// * `instr` – The OpCode instruction to added to the chunk.
    ///
    /// ## Returns
    /// * `usize` – The position of the currently emitted OpCode in the chunk.
    fn emit_op_code(&mut self, instr: bytecode::OpCode, pos: (usize, usize)) -> usize {
        self.function.chunk.push_byte(instr as u8);
        self.function.chunk.push_line_info(pos.clone());

        return self.function.chunk.len() - 1;
    }

    /// Emits a raw byte instruction into the chunk's instruction list.
    ///
    /// ## Arguments
    /// * `byte` – The byte instruction to added to the chunk.
    ///
    /// ## Returns
    /// * `usize` – The position of the currently emitted OpCode in the chunk.
    fn emit_raw_byte(&mut self, byte: u8, pos: (usize, usize)) -> usize {
        self.function.chunk.push_byte(byte);
        self.function.chunk.push_line_info(pos.clone());

        return self.function.chunk.len() - 1;
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
        self.function.chunk.push_short(instr);
        self.function.chunk.push_line_info(pos.clone());
        self.function.chunk.push_line_info(pos.clone());

        return self.function.chunk.len() - 2;
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
        match u16::try_from(self.function.chunk.len() - offset - 2) {
            Ok(forward) => {
                let jump = forward.to_be_bytes();

                self.function.chunk.modify_byte(offset, jump[0]);
                self.function.chunk.modify_byte(offset + 1, jump[1]);
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
        let offset = self.function.chunk.len() - loop_start;

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
