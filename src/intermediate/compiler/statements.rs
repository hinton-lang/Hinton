use super::{BreakScope, Compiler, Variable};
use std::borrow::Borrow;
use std::rc::Rc;

use crate::{
    chunk::op_codes::OpCode,
    intermediate::ast::{BlockNode, BreakStmtNode, ConstantDeclNode, IfStmtNode, PrintStmtNode, VariableDeclNode, WhileStmtNode},
    lexer::tokens::Token,
};

impl Compiler {
    /// Compiles a print statement.
    ///
    /// ## Arguments
    /// * `expr` – A print statement node.
    pub(super) fn compile_print_stmt(&mut self, stmt: &PrintStmtNode) {
        self.compile_node(&stmt.child);
        self.emit_op_code(OpCode::OP_PRINT, stmt.pos);
    }

    /// Compiles a variable declaration.
    ///
    /// ## Arguments
    /// * `expr` – A variable declaration node.
    pub fn compile_variable_decl(&mut self, decl: &VariableDeclNode) {
        // Declares the variables
        for id in decl.identifiers.iter() {
            match self.declare_variable(Rc::clone(&id), false) {
                Ok(_) => {
                    // Compiles the variable's value
                    self.compile_node(&decl.clone().value);
                }
                Err(_) => return,
            }
        }

        // Marks the variables as initialized the variables
        // a.k.a, defines the variables
        for var in self.variables.iter_mut().rev() {
            if var.depth < self.scope_depth {
                break;
            } else {
                var.is_initialized = true;
            }
        }
    }

    /// Compiles a variable declaration.
    ///
    /// ## Arguments
    /// * `expr` – A variable declaration node.
    pub fn compile_constant_decl(&mut self, decl: &ConstantDeclNode) {
        // Declares the variable
        match self.declare_variable(Rc::clone(&decl.name), true) {
            Ok(_) => {
                // Compiles the variable's value
                self.compile_node(&decl.value);

                // Marks the variables as initialized the variables
                for var in self.variables.iter_mut().rev() {
                    if var.depth < self.scope_depth {
                        break;
                    } else {
                        var.is_initialized = true;
                    }
                }
            }

            // We do nothing if there was an error because the `declare_variable()`
            // function takes care of reporting the appropriate error for us.
            Err(_) => {}
        }
    }

    /// Declares the variable by adding it to the variables array
    ///
    /// ## Arguments
    /// * `token` – The token (variable name) related to the variable being declared.
    /// * `is_const` – Whether or not the variable being declared is a constant.
    ///
    /// ## Returns
    /// `Result<(), ()>` – Whether or not there was an error with the variable declaration.
    fn declare_variable(&mut self, token: Rc<Token>, is_const: bool) -> Result<(), ()> {
        // Look for the variables declared in this scope to see if
        // there is a variable with the same name already declared.
        for var in self.variables.iter() {
            // Only look for the variable in the current scope.
            if var.depth < self.scope_depth {
                break;
            }

            if var.name.lexeme == token.lexeme {
                if !is_const {
                    self.error_at_token(token, "Cannot redeclare variable in the same scope.");
                } else {
                    self.error_at_token(token, "Cannot redeclare constant.");
                }
                return Err(());
            }
        }

        // Emit the variable if there is no variable with the
        // same name in the current scope.
        self.emit_variable(token, is_const)
    }

    /// Tries to emit a variable declaration into the variables array.
    ///
    /// ## Arguments
    /// * `token` – The token (variable name) related to the variable being emitted.
    /// * `is_const` – Whether or not the variable being emitted is a constant.
    ///
    /// ## Returns
    /// `Result<(), ()>` – Whether or not there was an error with the variable declaration.
    fn emit_variable(&mut self, name: Rc<Token>, is_const: bool) -> Result<(), ()> {
        if self.variables.len() >= (u16::MAX as usize) {
            self.error_at_token(name, "Too many variables in this program. Only 2^16 variables allowed.");
            return Err(());
        }

        self.variables.push(Variable {
            name,
            depth: self.scope_depth,
            is_initialized: false,
            is_const,
            is_used: false,
        });

        // Variable was successfully declared
        Ok(())
    }

    /// Compiles a block statement.
    ///
    /// * `block` – The block node being compiled.
    pub(super) fn compile_block_stmt(&mut self, block: &BlockNode) {
        self.begin_scope();

        for node in block.body.iter() {
            if self.had_error {
                return;
            }

            self.compile_node(&node.clone());
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

        while self.variables.len() > 0 && self.variables.get(self.variables.len() - 1).unwrap().depth > self.scope_depth {
            // Because variables live in the stack, once we are done with
            // them for this scope, we take them out of the stack by emitting
            // the OP_POP_STACK instruction for each one of the variables.
            // TODO: Change position to be the correct tuple
            self.emit_op_code(OpCode::OP_POP_STACK, (0, 0));
            let var = self.variables.pop().unwrap();
            if !var.is_used {
                println!(
                    "\x1b[33;1mCompilerWarning\x1b[0m at [{}:{}] – Variable '\x1b[37;1m{}\x1b[0m' is never used.",
                    var.name.line_num, var.name.column_num, var.name.lexeme
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
            then_jump = self.emit_jump(OpCode::OP_JUMP_IF_FALSE, Rc::clone(&stmt.then_token));
            self.emit_op_code(OpCode::OP_POP_STACK, (stmt.then_token.line_num, stmt.then_token.column_num));
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
                None => return
            }

            return;
        }

        let else_jump = match stmt.else_token.borrow() {
            Some(token) => self.emit_jump(OpCode::OP_JUMP, Rc::clone(&token)),
            // We are okay to return a dummy value because the only way `else_jump` can
            // be used is if there was an `else` branch in the first place. If there is
            // no `else` token, then there is no `else` branch, which means that the bellow
            // match statement will not execute, and so this value will not be used.
            None => 0,
        };

        if !condition_is_lit_false {
            self.patch_jump(then_jump, Rc::clone(&stmt.then_token));
            self.emit_op_code(OpCode::OP_POP_STACK, (stmt.then_token.line_num, stmt.then_token.column_num));
        }

        match stmt.else_branch.borrow() {
            Some(else_branch) => {
                self.compile_node(&else_branch);
                // Because at this point we *do* have an 'else' branch, we know that for sure
                // these is an `else_token`, so it is safe to unwrap without check.
                self.patch_jump(else_jump, stmt.else_token.clone().unwrap());
            }
            None => {}
        }
    }

    /// Compiles a while statement.
    ///
    /// * `stmt` – The while statement node being compiled.
    pub(super) fn compile_while_stmt(&mut self, stmt: &WhileStmtNode) {
        let do_compile_condition = !stmt.condition.is_truthy_literal();

        // We don't need to compile the loop if the condition is a
        // `false` literal because it will never execute.
        if stmt.condition.is_false_literal() {
            return;
        }

        let loop_start = self.chunk.codes.len();
        self.loops.push(loop_start); // starts this loop's scope

        let mut exit_jump = 0;
        if do_compile_condition {
            // Compiles the condition so that its value is at the top of the
            // stack during runtime. This value is then continuously checked
            // for truthiness to keep executing the while loop.
            self.compile_node(&stmt.condition);
            // Stop the loop if the condition (the top of the stack) is false.
            exit_jump = self.emit_jump(OpCode::OP_JUMP_IF_FALSE, Rc::clone(&stmt.token));

            // However, if the condition is not false, remove the condition value from the stack
            // and execute the loop's body.
            self.emit_op_code(OpCode::OP_POP_STACK, (stmt.token.line_num, stmt.token.column_num));
        }

        self.compile_node(&stmt.body);

        // Looks for any break statements associated with this loop
        let mut breaks: Vec<usize> = vec![];
        for b in self.breaks.iter().filter(|br| br.loop_start == loop_start) {
            breaks.push(b.loop_position);
        }

        // Patches all the breaks associated with this loop
        for b in breaks {
            self.patch_break(b, do_compile_condition, Rc::clone(&stmt.token));
        }

        // Jump back to the start of the loop (including the re-execution of the condition)
        self.emit_loop(loop_start, Rc::clone(&stmt.token));

        if do_compile_condition {
            // Patches the 'exit_jump' so that is the condition is false, the 'OP_JUMP_IF_FALSE'
            // instruction above knows where the end of the loop is.
            self.patch_jump(exit_jump, Rc::clone(&stmt.token));
            self.emit_op_code(OpCode::OP_POP_STACK, (stmt.token.line_num, stmt.token.column_num));
        }

        self.loops.pop(); // ends this loop's scope
    }

    /// Compiles a break statement.
    ///
    /// * `stmt` – The break statement node being compiled.
    pub(super) fn compile_break_stmt(&mut self, stmt: &BreakStmtNode) {
        // TODO: Check that this works well with function declarations.
        // Specially, check for the cases when a function is declared inside
        // of a loop, but the break statement is inside the function declaration.
        // For example, consider the bellow Hinton code:
        // ```
        // while (true) {
        //     func my_function() {
        //         break; // This is wrong...
        //     }
        // }
        // ```
        // Having the break inside the function body should result in a compiler error,
        // even when the function is inside of a loop.
        if self.loops.len() == 0 {
            self.error_at_token(Rc::clone(&stmt.token), "Cannot break outside of loop.");
            return;
        }

        let break_pos = self.emit_jump(OpCode::OP_JUMP, Rc::clone(&stmt.token));

        self.breaks.push(BreakScope {
            loop_start: *self.loops.last().unwrap(),
            loop_position: break_pos,
        })
    }
}
