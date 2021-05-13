use super::{BreakScope, Compiler, CompilerType, Symbol, SymbolType};
use std::borrow::Borrow;

use crate::{ast::*, chunk::OpCode, lexer::tokens::Token, objects::Object};

impl Compiler {
    pub(super) fn compile_expression_stmt(&mut self, stmt: &ExpressionStmtNode) {
        self.compile_node(&stmt.child);
        self.emit_op_code(OpCode::PopStack, stmt.pos);
    }

    /// Compiles a variable declaration.
    ///
    /// ## Arguments
    /// * `expr` – A variable declaration node.
    pub fn compile_variable_decl(&mut self, decl: &VariableDeclNode) {
        // Declares the variables
        for id in decl.identifiers.iter() {
            match self.declare_symbol(id, SymbolType::Variable) {
                Ok(symbol_pos) => {
                    // Compiles the variable's value
                    self.compile_node(&decl.value);

                    // Marks the variables as initialized
                    // a.k.a, defines the variables
                    self.symbol_table[symbol_pos].is_initialized = true;
                }

                // We do nothing if there was an error because the `declare_symbol()`
                // function takes care of reporting the appropriate error for us.
                // Explicit `return` to stop the loop.
                Err(_) => return,
            }
        }
    }

    /// Compiles a constant declaration.
    ///
    /// ## Arguments
    /// * `expr` – A constant declaration node.
    pub fn compile_constant_decl(&mut self, decl: &ConstantDeclNode) {
        // Declares the constant
        match self.declare_symbol(&decl.name, SymbolType::Constant) {
            Ok(symbol_pos) => {
                // Compiles the constant's value
                self.compile_node(&decl.value);

                // Marks the constant as initialized
                self.symbol_table[symbol_pos].is_initialized = true;
            }

            // We do nothing if there was an error because the `declare_symbol()`
            // function takes care of reporting the appropriate error for us.
            Err(_) => {}
        }
    }

    /// Declares the symbol by adding it to the symbol table
    ///
    /// ## Arguments
    /// * `token` – The token (symbol name) related to the symbol being declared.
    /// * `symbol_type` – The type of symbol being declared.
    ///
    /// ## Returns
    /// `Result<(), ()>` – Whether or not there was an error with the variable declaration.
    fn declare_symbol(&mut self, token: &Token, symbol_type: SymbolType) -> Result<usize, ()> {
        // Look for the symbols declared in this scope to see if
        // there is a symbol with the same name already declared.
        for symbol in self.symbol_table.iter().rev() {
            // Only look for the symbol in the current scope.
            if symbol.symbol_depth < self.scope_depth {
                break;
            }

            if symbol.name == token.lexeme {
                match symbol.symbol_type {
                    SymbolType::Variable => self.error_at_token(
                        token,
                        "A variable with this name already exists in this scope.",
                    ),
                    SymbolType::Constant => self.error_at_token(
                        token,
                        "A constant with this name already exists in this scope.",
                    ),
                    SymbolType::Function => self.error_at_token(
                        token,
                        "A function with this name already exists in this scope.",
                    ),
                    SymbolType::Class => self.error_at_token(
                        token,
                        "A class with this name already exists in this scope.",
                    ),
                    SymbolType::Enum => self.error_at_token(
                        token,
                        "An enum with this name already exists in this scope.",
                    ),
                    SymbolType::Parameter => self.error_at_token(
                        token,
                        "A parameter with this name already exists in this scope.",
                    ),
                }

                return Err(());
            }
        }

        // Emit the symbol if there is no symbol with the
        // same name in the current scope.
        self.emit_symbol(&token, symbol_type)
    }

    /// Tries to emit a symbol declaration into the symbol table.
    ///
    /// ## Arguments
    /// * `token` – The token (symbol name) related to the symbol being emitted.
    /// * `symbol_type` – The type of symbol being emitted.
    ///
    /// ## Returns
    /// `Result<(), ()>` – Whether or not there was an error with the symbol declaration.
    fn emit_symbol(&mut self, name: &Token, symbol_type: SymbolType) -> Result<usize, ()> {
        if self.symbol_table.len() >= (u16::MAX as usize) {
            self.error_at_token(name, "Too many variables in this scope.");
            return Err(());
        }

        self.symbol_table.push(Symbol {
            name: name.lexeme.clone(),
            symbol_depth: self.scope_depth,
            is_initialized: match symbol_type {
                SymbolType::Variable | SymbolType::Constant => false,
                _ => true,
            },
            symbol_type,
            is_used: false,
            pos: (name.line_num, name.column_num),
        });

        // Variable was successfully declared
        Ok(self.symbol_table.len() - 1)
    }

    /// Compiles a block statement.
    ///
    /// * `block` – The block node being compiled.
    pub(super) fn compile_block_stmt(&mut self, block: &BlockNode) {
        self.begin_scope();

        for node in block.body.iter() {
            self.compile_node(&node.clone());

            // If after compiling the node there was an error, stop
            // the loop and return out of the function.
            if self.had_error {
                return;
            }
        }

        self.end_scope();
    }

    /// Starts a new scope.
    pub(super) fn begin_scope(&mut self) {
        self.scope_depth += 1;
    }

    /// Ends a scope.
    pub(super) fn end_scope(&mut self) {
        self.scope_depth -= 1;

        // When a scope ends, we remove all local symbols in the scope.
        while self.symbol_table.len() > 0
            && self
                .symbol_table
                .get(self.symbol_table.len() - 1)
                .unwrap()
                .symbol_depth
                > self.scope_depth
        {
            // Because variables live in the stack, once we are done with
            // them for this scope, we take them out of the stack by emitting
            // the OP_POP_STACK instruction for each one of the variables.
            let symbol = self.symbol_table.pop().unwrap();
            self.emit_op_code(OpCode::PopStack, symbol.pos);
            if !symbol.is_used {
                println!(
                    "\x1b[33;1mCompilerWarning\x1b[0m at [{}:{}] – Variable '\x1b[37;1m{}\x1b[0m' is never used.",
                    symbol.pos.0, symbol.pos.1, symbol.name
                );
            }
        }
    }

    /// Compiles an if statement.
    ///
    /// * `block` – The if statement node being compiled.
    pub(super) fn compile_if_stmt(&mut self, stmt: &IfStmtNode) {
        let condition_is_lit_true = stmt.condition.is_truthy_literal();
        let condition_is_lit_false = stmt.condition.is_false_literal();

        let mut then_jump = 0;
        // Only execute the condition if it is not a boolean literal (or equivalent).
        if !condition_is_lit_true && !condition_is_lit_false {
            // Compiles the condition so that its value is at the top of the
            // stack during runtime. This value is then checked for truthiness
            // to execute the correct branch of the if statement.
            self.compile_node(&stmt.condition);
            then_jump = self.emit_jump(OpCode::JumpIfFalse, &stmt.then_token);
            self.emit_op_code(
                OpCode::PopStack,
                (stmt.then_token.line_num, stmt.then_token.column_num),
            );
        }

        // If the condition is always false, the `then`
        // branch does not need to be compiled at all.
        if !condition_is_lit_false {
            self.compile_node(&stmt.then_branch);
        }

        // If the condition is always true, then the `else`
        // branch does not need to be compiled at all.
        if condition_is_lit_true {
            return;
        }

        // If the condition is always false, then we only care about
        // compiling the else branch. However, if there is no else branch
        // we return out of the function because there is nothing else to do.
        if condition_is_lit_false {
            match stmt.else_branch.borrow() {
                Some(else_branch) => self.compile_node(&else_branch),
                None => return,
            }

            return;
        }

        let else_jump = match stmt.else_token.borrow() {
            Some(token) => self.emit_jump(OpCode::Jump, &token),
            // We are okay to return a dummy value because the only way `else_jump` can
            // be used is if there was an `else` branch in the first place. If there is
            // no `else` token, then there is no `else` branch, which means that the bellow
            // match statement will not execute, and so this value will not be used.
            None => 0,
        };

        if !condition_is_lit_false {
            self.patch_jump(then_jump, &stmt.then_token);
            self.emit_op_code(
                OpCode::PopStack,
                (stmt.then_token.line_num, stmt.then_token.column_num),
            );
        }

        if let Some(else_branch) = stmt.else_branch.borrow() {
            self.compile_node(&else_branch);
            // Because at this point we *do* have an 'else' branch, we know that for sure
            // these is an `else_token`, so it is safe to unwrap without check.
            self.patch_jump(else_jump, &stmt.else_token.clone().unwrap());
        }
    }

    /// Compiles a while statement.
    ///
    /// * `stmt` – The while statement node being compiled.
    pub(super) fn compile_while_stmt(&mut self, stmt: &WhileStmtNode) {
        // We don't need to compile the loop if the condition is a
        // `false` literal because it will never execute.
        if stmt.condition.is_false_literal() {
            return;
        }

        let condition_is_truthy_lit = stmt.condition.is_truthy_literal();

        let loop_start = self.function.chunk.len();
        self.loops.push(loop_start); // starts this loop's scope

        // Only compile the condition if it is not a truthy literal or equivalent.
        let mut exit_jump = 0;
        if !condition_is_truthy_lit {
            self.compile_node(&stmt.condition);
            exit_jump = self.emit_jump(OpCode::JumpIfFalse, &stmt.token);

            // If the condition is not false, remove the condition value from the stack
            // and continue to execute the loop's body.
            self.emit_op_code(
                OpCode::PopStack,
                (stmt.token.line_num, stmt.token.column_num),
            );
        }

        self.compile_node(&stmt.body);
        // Stop compiling if there was an error compiling the loop's body.
        if self.had_error {
            return;
        }

        // Looks for any break statements associated with this loop
        let mut breaks: Vec<usize> = vec![];
        for b in self.breaks.iter().filter(|br| br.loop_start == loop_start) {
            breaks.push(b.loop_position);
        }

        // Patches all the breaks associated with this loop
        for b in breaks {
            self.patch_break(b, !condition_is_truthy_lit, &stmt.token);
        }

        // Jump back to the start of the loop (including the re-execution of the condition)
        self.emit_loop(loop_start, &stmt.token);

        // If the condition is not a truthy literal, then we must patch the 'OP_JUMP_IF_FALSE' above
        if !condition_is_truthy_lit {
            self.patch_jump(exit_jump, &stmt.token);
            self.emit_op_code(
                OpCode::PopStack,
                (stmt.token.line_num, stmt.token.column_num),
            );
        }

        self.loops.pop(); // ends this loop's scope
    }

    /// Compiles a break statement.
    ///
    /// * `stmt` – The break statement node being compiled.
    pub(super) fn compile_break_stmt(&mut self, stmt: &BreakStmtNode) {
        if self.loops.len() == 0 {
            self.error_at_token(&stmt.token, "Cannot break outside of loop.");
            return;
        }

        // Pop all local symbols off the stack when the loop ends, but do not
        // remove the symbols from the symbol table since they must also be
        // removed when the loop's scope.
        let mut i = 1;
        while self.symbol_table.len() > 0
            && self
                .symbol_table
                .get(self.symbol_table.len() - i)
                .unwrap()
                .symbol_depth
                >= self.scope_depth
        {
            let idx = self.symbol_table.len() - i;
            let symbol = &self.symbol_table[idx];
            let pos = symbol.pos;

            self.emit_op_code(OpCode::PopStack, pos);
            i += 1;
        }

        // Jump out of the loop
        let break_pos = self.emit_jump(OpCode::Jump, &stmt.token);

        // Add to the breaks list to breaks associated with the current loop
        self.breaks.push(BreakScope {
            loop_start: *self.loops.last().unwrap(),
            loop_position: break_pos,
        })
    }

    pub(super) fn compile_function_decl(&mut self, decl: &FunctionDeclNode) {
        match self.declare_symbol(&decl.name, SymbolType::Function) {
            Ok(_) => {
                let comp = match Compiler::compile_function(&decl, self.natives.clone()) {
                    Ok(func) => func,
                    Err(_) => {
                        // We specify that there was an error inside the body
                        // of the function so that we can stop compiling the
                        // program all together.
                        self.had_error = true;
                        return;
                    }
                };

                // Defines the function so that it can be loaded onto the stack.
                // When the function is first loaded onto the stack, it has no
                // default parameters initialized.
                self.add_literal_to_pool(Object::Function(comp), &decl.name);

                if decl.min_arity != decl.max_arity {
                    // Compiles the named parameters so that they can be on top
                    // of the stack when the function gets composed at runtime.
                    for param in &decl.params {
                        match &param.default {
                            Some(expr) => {
                                self.compile_node(&expr);
                            }
                            None => {
                                if param.is_optional {
                                    self.emit_op_code(
                                        OpCode::LoadImmNull,
                                        (param.name.line_num, param.name.column_num),
                                    );
                                }
                            }
                        }
                    }

                    // Once all the named parameter expressions are compiled, we bind
                    // each of the named parameters to the function
                    self.emit_op_code(
                        OpCode::BindDefaults,
                        (decl.name.line_num, decl.name.column_num),
                    );
                    self.emit_raw_byte(
                        (decl.max_arity - decl.min_arity) as u8,
                        (decl.name.line_num, decl.name.column_num),
                    );
                }
            }

            // We do nothing if there was an error because the `declare_symbol()`
            // function takes care of reporting the appropriate error for us.
            // Explicit `return` to stop the loop.
            Err(_) => return,
        }
    }

    pub(super) fn compile_parameters(&mut self, params: &Vec<Parameter>) {
        for param in params.iter() {
            match self.declare_symbol(&param.name, SymbolType::Parameter) {
                Ok(symbol_pos) => {
                    self.symbol_table[symbol_pos].is_initialized = true;
                }

                // We do nothing if there was an error because the `declare_symbol()`
                // function takes care of reporting the appropriate error for us.
                // Explicit `return` to stop the loop.
                Err(_) => return,
            }
        }
    }

    pub(super) fn compile_return_stmt(&mut self, stmt: &ReturnStmtNode) {
        if let CompilerType::Script = self.compiler_type {
            self.error_at_token(&stmt.token, "Cannot return outside of function.");
            return;
        }

        match &stmt.value {
            Some(v) => {
                self.compile_node(v);
            }
            None => {
                self.emit_op_code(
                    OpCode::LoadImmNull,
                    (stmt.token.line_num, stmt.token.column_num),
                );
            }
        }

        self.emit_op_code(OpCode::Return, (stmt.token.line_num, stmt.token.column_num));
        // The number of local symbols that need to be popped off the stack
        let num_of_symbols = self.symbol_table.len() - 1;
        self.emit_raw_byte(
            num_of_symbols as u8,
            (stmt.token.line_num, stmt.token.column_num),
        );
    }
}
