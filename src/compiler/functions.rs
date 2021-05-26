use super::{Compiler, CompilerErrorType, CompilerType, SymbolType};
use crate::{ast::*, chunk::OpCode, objects::Object};
use std::borrow::BorrowMut;

impl Compiler {
    pub(super) fn compile_function_decl(&mut self, decl: &FunctionDeclNode) {
        match self.declare_symbol(&decl.name, SymbolType::Function) {
            Ok(symbol_pos) => {
                let comp = match Compiler::compile_function(
                    self.filepath.clone(),
                    self.natives.clone(),
                    &decl,
                ) {
                    Ok(func) => func,
                    Err(mut e) => {
                        self.errors.append(e.borrow_mut());

                        // If there is an error in the body of the function, then
                        // the program will not run, but we still try to compile
                        // the function's named parameters to try and catch
                        // any errors there too and show them to the programmer.
                        self.compile_named_parameters(decl);

                        return;
                    }
                };

                // Defines the function so that it can be loaded onto the stack.
                // When the function is first loaded onto the stack, it has no
                // default parameters initialized.
                self.add_literal_to_pool(Object::Function(comp), &decl.name);

                // Compile the named parameters so that they can be
                // bound to the function at runtime.
                self.compile_named_parameters(decl);

                // Mark the function as initialized for the parent scope.
                self.symbol_table[symbol_pos].is_initialized = true;
            }

            // We do nothing if there was an error because the `declare_symbol()`
            // function takes care of reporting the appropriate error for us.
            // Explicit `return` to stop the loop.
            Err(_) => return,
        }
    }

    fn compile_named_parameters(&mut self, decl: &FunctionDeclNode) {
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

    pub(super) fn compile_parameters(&mut self, params: &Vec<Parameter>) {
        for param in params.iter() {
            match self.declare_symbol(&param.name, SymbolType::Parameter) {
                Ok(_) => {
                    // Do nothing after parameter has been declared. Default
                    // values will be compiled by the function's parent scope.
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
            self.error_at_token(
                &stmt.token,
                CompilerErrorType::Syntax,
                "Cannot return outside of function.",
            );
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
