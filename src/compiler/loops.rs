use crate::{
    ast::{BreakStmtNode, ForStmtNode, WhileStmtNode},
    chunk::OpCode,
    compiler::{Symbol, SymbolType},
    lexer::tokens::Token,
};

use super::{BreakScope, Compiler, LoopScope};

impl Compiler {
    /// Compiles a `while` statement.
    ///
    /// * `stmt` – The `while` statement node being compiled.
    pub(super) fn compile_while_stmt(&mut self, stmt: &WhileStmtNode) {
        // We don't need to compile the loop if the condition is a
        // `false` literal because it will never execute.
        if stmt.condition.is_false_literal() {
            return;
        }

        let condition_is_truthy_lit = stmt.condition.is_truthy_literal();

        let loop_start = self.function.chunk.len();
        // starts this loop's break scope
        self.loops.push(LoopScope {
            position: loop_start,
            loop_type: super::LoopType::While,
        });

        // Only compile the condition if it is not a truthy literal or equivalent.
        let mut exit_jump = 0;
        if !condition_is_truthy_lit {
            self.compile_node(&stmt.condition);
            exit_jump = self.emit_jump(OpCode::PopJumpIfFalse, &stmt.token);
        }

        self.compile_node(&stmt.body);
        // Stop compiling if there was an error compiling the loop's body.
        if self.had_error {
            return;
        }

        // Jump back to the start of the loop (including the re-execution of the condition)
        self.emit_loop(loop_start, &stmt.token);

        // If the condition is not a truthy literal, then we must patch the 'OP_JUMP_IF_FALSE' above
        if !condition_is_truthy_lit {
            self.patch_jump(exit_jump, &stmt.token);
        }

        self.close_breaks(loop_start, &stmt.token);
        self.loops.pop(); // ends this loop's break scope
    }

    /// Compiles a `for` statement.
    ///
    /// * `stmt` – The `for` statement node being compiled.
    pub(super) fn compile_for_stmt(&mut self, stmt: &ForStmtNode) {
        let loop_line_info = (stmt.token.line_num, stmt.token.column_num);

        // Create the iterator at runtime
        self.compile_node(&stmt.iterator);
        self.emit_op_code(OpCode::MakeIter, loop_line_info);

        // Begin the loop
        self.emit_op_code(OpCode::ForLoopIterNext, loop_line_info);
        let loop_start = self.function.chunk.len() - 1;
        // Starts this loop's break scope
        self.loops.push(LoopScope {
            position: loop_start,
            loop_type: super::LoopType::ForIn,
        });

        // Increment the scope so that the loop's identifier and iterator
        // placeholder have their own scope.
        self.scope_depth += 1;

        // Emits a placeholder symbol for the loop's iterator, which lives on
        // the stack until the end of the loop. The programmer will never be
        // able to access this "variable" because of the format of its name.
        match self.push_symbol(
            &stmt.token,
            Symbol {
                name: format!("<for-loop placeholder at byte #{}>", loop_start),
                symbol_depth: self.scope_depth,
                is_initialized: true,
                symbol_type: SymbolType::Constant,
                is_used: true,
                pos: loop_line_info,
            },
        ) {
            Ok(symbol_pos) => self.symbol_table[symbol_pos].is_initialized = true,
            Err(_) => return,
        }

        // Declares the loop's identifier.
        match self.declare_symbol(&stmt.id.token, SymbolType::Constant) {
            Ok(symbol_pos) => self.symbol_table[symbol_pos].is_initialized = true,
            Err(_) => return,
        }

        // Compiles the loop's body
        self.compile_node(&stmt.body);

        // Stop compiling if there was an error compiling the loop's body.
        if self.had_error {
            return;
        }

        // +2 to count the jump instruction and its one operand
        let offset = (self.function.chunk.len() + 2) - loop_start;

        // Jump back to the start of the loop if we haven't reached the end of the
        // iterator. This instruction takes care of popping the loop's  variable.
        if offset < 256 {
            self.emit_op_code(OpCode::JumpHasNextOrPop, loop_line_info);
            self.emit_raw_byte(offset as u8, loop_line_info);
        } else {
            // +3 to count the Jump instruction and its two operands
            let offset = (self.function.chunk.len() + 3) - loop_start;

            if offset < u16::MAX as usize {
                self.emit_op_code(OpCode::JumpHasNextOrPopLong, loop_line_info);
                self.emit_short(offset as u16, loop_line_info);
            } else {
                self.error_at_token(&stmt.token, "Loop body too large.");
                return;
            }
        }

        // Closes & patches all breaks associated with this loop
        self.close_breaks(loop_start, &stmt.token);

        // Ends this loop's break scope
        self.loops.pop();

        // Remove the loop's variables and end the scope
        self.symbol_table.pop();
        self.symbol_table.pop();
        self.scope_depth -= 1;
    }

    /// Patches all breaks associated with the current loop.
    fn close_breaks(&mut self, loop_start: usize, token: &Token) {
        // Looks for any break statements associated with this loop
        let mut breaks: Vec<usize> = vec![];
        for b in self
            .breaks
            .iter()
            .filter(|br| br.parent_loop.position == loop_start)
        {
            breaks.push(b.loop_position);
        }

        // Patches all the breaks associated with this loop
        for b in breaks {
            self.patch_jump(b, token);
        }
    }

    /// Compiles a `break` statement.
    ///
    /// * `stmt` – The `break` statement node being compiled.
    pub(super) fn compile_break_stmt(&mut self, stmt: &BreakStmtNode) {
        if self.loops.len() == 0 {
            self.error_at_token(&stmt.token, "Cannot break outside of loop.");
            return;
        }

        let current_loop = *self.loops.last().unwrap();
        let mut pop_count = 0usize;
        let mut last_symbol_pos = (0, 0);

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
            pop_count += 1;
            last_symbol_pos = symbol.pos;

            i += 1;
        }

        // If we are breaking inside a for-in loop, also pop the loop's variable
        // and the iterator off the stack before exiting the loop.
        if let super::LoopType::ForIn = current_loop.loop_type {
            pop_count += 2;
        }

        if pop_count > 0 {
            if pop_count < 256 {
                self.emit_op_code(OpCode::PopStackN, last_symbol_pos);
                self.emit_raw_byte(pop_count as u8, last_symbol_pos);
            } else {
                self.emit_op_code(OpCode::PopStackNLong, last_symbol_pos);
                self.emit_short(pop_count as u16, last_symbol_pos);
            }
        }

        // Jump out of the loop
        let break_pos = self.emit_jump(OpCode::JumpForward, &stmt.token);

        // Add to the breaks list to breaks associated with the current loop
        self.breaks.push(BreakScope {
            parent_loop: current_loop,
            loop_position: break_pos,
        })
    }
}
