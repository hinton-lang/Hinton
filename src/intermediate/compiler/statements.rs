use super::Compiler;
use std::rc::Rc;

use crate::{
    chunk::{op_codes::OpCode, ConstantPos},
    intermediate::ast::{PrintStmtNode, VariableDeclNode},
    lexer::tokens::Token,
    objects::Object,
};

impl Compiler {
    /// Compiles a print statement.
    ///
    /// ## Arguments
    /// * `expr` – A print statement node.
    pub(super) fn compile_print_stmt(&mut self, stmt: PrintStmtNode) {
        self.compile_node(*stmt.child);
        self.emit_op_code(OpCode::OP_PRINT, stmt.pos);
    }

    /// Compiles a variable declaration.
    ///
    /// ## Arguments
    /// * `expr` – A variable declaration node.
    pub fn compile_variable_decl(&mut self, decl: VariableDeclNode) {
        let mut ids: Vec<u16> = Vec::new();

        // Declares the variables
        for id in decl.identifiers.iter() {
            match self.add_identifier_to_pool(Rc::clone(&id)) {
                ConstantPos::Pos(x) => ids.push(x),
                // If there are too many constants, display an error.
                ConstantPos::Error => return self.error_at_token(Rc::clone(&id), "Could not complete variable declaration."),
            };
        }

        // Compiles the variable's value
        self.compile_node(*decl.value);

        // Defines the variables
        let mut index = 0;
        for idx in ids {
            let tok = decl.identifiers.get(index).unwrap();

            self.emit_op_code(OpCode::OP_DEFINE_GLOBAL_VAR, (tok.line_num, tok.column_num));
            self.emit_short(idx, (tok.line_num, tok.column_num));

            index += 1;
        }

        // Pop variable's value off the stack once we
        // are done declaring the variables
        let last = decl.identifiers.last().unwrap();
        self.emit_op_code(OpCode::OP_POP_STACK, (last.line_num, last.column_num));
    }

    /// Adds an identifier to the constant pool
    ///
    /// ## Arguments
    /// * `token` – The token associated with the identifier being added to the pool
    pub(super) fn add_identifier_to_pool(&mut self, token: Rc<Token>) -> ConstantPos {
        self.chunk.add_constant(Rc::new(Object::String(token.lexeme.clone())))
    }
}
