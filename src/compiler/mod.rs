use crate::{
    ast::*,
    chunk::{self, Chunk, OpCode},
    lexer::tokens::{Token, TokenType},
    objects::{FunctionChunk, FunctionObject},
    virtual_machine::InterpretResult,
};
use std::{convert::TryFrom, fmt, fmt::Display, rc::Rc, str};

// Submodules
mod expressions;
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
        let s = format!("(name: {}, scope: {})", self.name.as_str(), self.symbol_depth);
        fmt::Debug::fmt(s.as_str(), f)
    }
}

/// Represents a break statement, which is associated with a loop.
struct BreakScope {
    loop_start: usize,
    loop_position: usize,
}

/// Represents a compiler and its internal state.
pub struct Compiler {
    compiler_type: CompilerType,
    had_error: bool,
    is_in_panic: bool,
    function: FunctionObject,
    // Lexical scoping of declarations
    symbol_table: Vec<Symbol>,
    scope_depth: usize,
    // Used to match the break statements with the correct
    // loop (specially useful with nested loops), and to
    // patch the break (OP_JUMP) offsets.
    loops: Vec<usize>,
    breaks: Vec<BreakScope>,
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
    /// * `program` – The root node of the AST for a particular program.
    ///
    /// ## Returns
    /// `Result<chunk, InterpretResult>` – If the program had no compile-time errors, returns
    /// the main chunk for this module. Otherwise returns an InterpretResult::INTERPRET_COMPILE_ERROR.
    pub fn compile_file(filepath: &str, program: &ASTNode) -> Result<FunctionObject, InterpretResult> {
        let mut c = Compiler {
            compiler_type: CompilerType::Script,
            had_error: false,
            is_in_panic: false,
            function: FunctionObject {
                defaults: vec![],
                body: FunctionChunk {
                    min_arity: 0,
                    max_arity: 0,
                    chunk: Chunk::new(),
                    name: format!("Script: '{}'", filepath),
                },
            },
            symbol_table: vec![Symbol {
                name: format!("Script: '{}'", filepath),
                symbol_depth: 0,
                symbol_type: SymbolType::Function,
                is_initialized: true,
                is_used: true,
                pos: (0, 0),
            }],
            scope_depth: 0,
            loops: vec![],
            breaks: vec![],
        };

        // Compile the function body
        c.compile_node(&program);
        // ends the compiler.
        c.end_compiler();

        if !c.had_error && !c.is_in_panic {
            Ok(c.function)
        } else {
            return Err(InterpretResult::CompileError);
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
    pub fn compile_function(func: &FunctionDeclNode) -> Result<FunctionObject, InterpretResult> {
        let mut c = Compiler {
            compiler_type: CompilerType::Function,
            had_error: false,
            is_in_panic: false,
            function: FunctionObject {
                defaults: vec![],
                body: FunctionChunk {
                    min_arity: func.min_arity,
                    max_arity: func.max_arity,
                    chunk: Chunk::new(),
                    name: func.name.lexeme.clone(),
                },
            },
            symbol_table: vec![Symbol {
                name: func.name.lexeme.clone(),
                symbol_depth: 0,
                symbol_type: SymbolType::Function,
                is_initialized: true,
                is_used: true,
                pos: (0, 0),
            }],
            scope_depth: 0,
            loops: vec![],
            breaks: vec![],
        };

        // compiles the parameter declarations so that the compiler
        // knows about their their lexical scoping (their stack position),
        // but does not compile the default value for named parameters.
        c.compile_parameters(&func.params);
        // Compile the function body
        c.compile_node(&func.body);
        // ends the compiler.
        c.end_compiler();

        if !c.had_error && !c.is_in_panic {
            Ok(c.function)
        } else {
            return Err(InterpretResult::CompileError);
        }
    }

    fn end_compiler(&mut self) {
        // TODO: Emit the correct position
        self.emit_op_code(OpCode::LoadImmNull, (0, 0));
        self.emit_op_code(OpCode::Return, (0, 0));

        // The number of local symbols that need to be popped off the stack
        let num_of_symbols = self.symbol_table.len() - 1;
        self.emit_raw_byte(num_of_symbols as u8, (0, 0));

        // Shows the chunk.
        // chunk::disassemble_chunk(&self.function.body.chunk, self.function.body.name.as_str());
        // chunk::print_raw(&self.function.body.chunk, self.function.body.name.as_str());
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
            ASTNode::FunctionCallExpr(x) => self.compile_function_call_expr(x),
            ASTNode::FunctionDecl(x) => self.compile_function_decl(x),
            ASTNode::Identifier(x) => self.compile_identifier_expr(x),
            ASTNode::IfStmt(x) => self.compile_if_stmt(x),
            ASTNode::Literal(x) => self.compile_literal_expr(x),
            ASTNode::Module(x) => self.compile_module_node(x),
            ASTNode::PostDecrement(x) => self.compile_post_decrement_expr(x),
            ASTNode::PostIncrement(x) => self.compile_post_increment_expr(x),
            ASTNode::PrintStmt(x) => self.compile_print_stmt(x),
            ASTNode::ReturnStmt(x) => self.compile_return_stmt(x),
            ASTNode::TernaryConditional(x) => self.compile_ternary_conditional_expr(x),
            ASTNode::Unary(x) => self.compile_unary_expr(x),
            ASTNode::VarReassignment(x) => self.compile_var_reassignment_expr(x),
            ASTNode::VariableDecl(x) => self.compile_variable_decl(x),
            ASTNode::WhileStmt(x) => self.compile_while_stmt(x),
        };
    }

    fn compile_module_node(&mut self, module: &ModuleNode) {
        for node in module.body.iter() {
            self.compile_node(node);

            if self.had_error {
                break;
            }
        }
    }

    /// Emits a byte instruction from an OpCode into the chunk's instruction list.
    ///
    /// ## Arguments
    /// * `instr` – The OpCode instruction to added to the chunk.
    ///
    /// ## Returns
    /// * `usize` – The position of the currently emitted OpCode in the chunk.
    fn emit_op_code(&mut self, instr: OpCode, pos: (usize, usize)) -> usize {
        self.function.body.chunk.push_byte(instr as u8);
        self.function.body.chunk.push_line_info(pos.clone());

        return self.function.body.chunk.len() - 1;
    }

    /// Emits a raw byte instruction into the chunk's instruction list.
    ///
    /// ## Arguments
    /// * `byte` – The byte instruction to added to the chunk.
    ///
    /// ## Returns
    /// * `usize` – The position of the currently emitted OpCode in the chunk.
    fn emit_raw_byte(&mut self, byte: u8, pos: (usize, usize)) -> usize {
        self.function.body.chunk.push_byte(byte);
        self.function.body.chunk.push_line_info(pos.clone());

        return self.function.body.chunk.len() - 1;
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
        self.function.body.chunk.push_short(instr);
        self.function.body.chunk.push_line_info(pos.clone());
        self.function.body.chunk.push_line_info(pos.clone());

        return self.function.body.chunk.len() - 2;
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
    fn patch_jump(&mut self, offset: usize, token: Rc<Token>) {
        // -2 to adjust for the bytecode for the jump offset itself.
        let jump = match u16::try_from((self.function.body.chunk.len() - offset) - 2) {
            Ok(x) => x,
            Err(_) => {
                return self.error_at_token(token, "Too much code to jump over.");
            }
        };

        let j = jump.to_be_bytes();
        self.function.body.chunk.modify_byte(offset, j[0]);
        self.function.body.chunk.modify_byte(offset + 1, j[1]);
    }

    /// Patches the offset of a break (OP_JUMP) instruction.
    ///
    /// ## Arguments
    /// * `offset` – The position in the chunk of the break (OP_JUMP) instruction to be patched.
    /// * `token` – The token associated with this jump patch.
    fn patch_break(&mut self, offset: usize, has_condition: bool, token: Rc<Token>) {
        // If the corresponding loop does not have a truthy literal condition, then that
        // condition MUST be popped off the stack after the loop ends. Because breaking the
        // loop with the `break` keyword ends the loop early, the break statement must take care
        // of popping the condition off the stack. That is why we add one to the jump offset,
        // so that the condition is popped off correctly. However, if there is no condition
        // to pop off, then we leave the stack untouched when we break the loop.
        let with_condition = if has_condition { 1 } else { 0 };

        let jump = match u16::try_from(self.function.body.chunk.len() - offset + with_condition) {
            Ok(x) => x,
            Err(_) => {
                return self.error_at_token(token, "Too much code to jump over.");
            }
        };

        let j = jump.to_be_bytes();
        self.function.body.chunk.modify_byte(offset, j[0]);
        self.function.body.chunk.modify_byte(offset + 1, j[1]);
    }

    /// Patches the offset of a jump instruction.
    ///
    /// ## Arguments
    /// * `loop_start` – The position in the chunk of the jump instruction to be patched.
    /// * `token` – The token associated with this jump patch.
    fn emit_loop(&mut self, loop_start: usize, token: Rc<Token>) {
        let offset = self.function.body.chunk.len() - loop_start;

        if offset < (u8::MAX - 2) as usize {
            self.emit_op_code(OpCode::LoopJump, (token.line_num, token.column_num));

            // +2 to account for the 'OP_LOOP_JUMP' and its operand.
            let jump = (offset + 2) as u8;
            self.emit_raw_byte(jump, (token.line_num, token.column_num));
        } else if offset < (u16::MAX - 3) as usize {
            self.emit_op_code(OpCode::LoopJumpLong, (token.line_num, token.column_num));

            // +3 to account for the 'OP_LOOP_JUMP_LONG' and its operands.
            let jump = (offset + 3) as u16;
            self.emit_short(jump, (token.line_num, token.column_num));
        } else {
            return self.error_at_token(token, "Loop body too large.");
        }
    }

    /// Emits a compiler error from the given token.
    ///
    /// ## Arguments
    /// *  `tok` – The token that caused the error.
    /// * `message` – The error message to display.
    fn error_at_token(&mut self, tok: Rc<Token>, message: &str) {
        if self.is_in_panic {
            return;
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
