use super::{Compiler, CompilerErrorType, CompilerType, SymbolType};
use crate::{
    ast::*,
    bytecode::{self, OpCode},
    compiler::{
        symbols::{Symbol, SymbolTable},
        FunctionScope,
    },
    objects::{FuncObject, Object},
};

impl Compiler {
    pub(super) fn compile_function_decl(&mut self, decl: &FunctionDeclNode) {
        match self.declare_symbol(&decl.name, SymbolType::Function) {
            Ok(parent_symbol_pos) => {
                let func_pos = (decl.name.line_num, decl.name.column_num);

                let prev_compiler_type =
                    std::mem::replace(&mut self.compiler_type, CompilerType::Function);

                // The first element in a symbol table is always the symbol representing
                // the function to which the symbol table belongs.
                let symbols = SymbolTable {
                    symbols: vec![Symbol {
                        name: decl.name.lexeme.clone(),
                        symbol_type: SymbolType::Function,
                        is_initialized: true,
                        symbol_depth: 0,
                        is_used: true,
                        line_info: func_pos,
                        is_global: false,
                    }],
                };

                let new_function_scope = FunctionScope {
                    function: FuncObject {
                        defaults: vec![],
                        min_arity: decl.min_arity,
                        max_arity: decl.max_arity,
                        chunk: bytecode::Chunk::new(),
                        name: decl.name.lexeme.clone(),
                    },
                    s_table: symbols,
                    scope_depth: 0,
                    loops: vec![],
                    breaks: vec![],
                };

                // Make the this function declaration the
                // current function scope.
                self.functions.push(new_function_scope);

                // Add the function's name to the pool of the function
                self.add_literal_to_pool(
                    Object::String(decl.name.lexeme.clone()),
                    &decl.name,
                    false,
                );

                // compiles the parameter declarations so that the compiler
                // knows about their their lexical scoping (their stack position),
                // but does not compile the default value for named parameters.
                self.compile_parameters(&decl.params);

                // Compile the function body
                self.compile_node(&decl.body);
                self.end_function();

                // Defines the function so that it can be loaded onto the stack.
                // When the function is first loaded onto the stack, it has no
                // default parameters initialized.
                let compiled_function = std::mem::take(&mut self.current_func_scope_mut().function);

                // Go back to the previous function
                self.functions.pop();
                self.compiler_type = prev_compiler_type;

                // Add the function object to the literal pool of the parent function
                self.add_literal_to_pool(Object::Function(compiled_function), &decl.name, true);

                // Compile the named parameters so that they can be
                // bound to the function at runtime.
                if decl.min_arity != decl.max_arity {
                    self.compile_named_parameters(decl);
                }

                // If we are in the global scope, declarations are
                // stored in the VM.globals hashmap
                if self.is_global_scope() {
                    self.define_as_global(&decl.name);
                    self.globals.symbols[parent_symbol_pos].is_initialized = true;
                } else {
                    // Marks the variables as initialized
                    // a.k.a, defines the variables
                    self.current_func_scope_mut().s_table.symbols[parent_symbol_pos]
                        .is_initialized = true;
                }
            }

            // We do nothing if there was an error because the `declare_symbol()`
            // function takes care of reporting the appropriate error for us.
            // Explicit `return` to stop the loop.
            Err(_) => return,
        }
    }

    fn compile_named_parameters(&mut self, decl: &FunctionDeclNode) {
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
        let num_of_symbols = self.current_function_scope().s_table.len() - 1;
        self.emit_raw_byte(
            num_of_symbols as u8,
            (stmt.token.line_num, stmt.token.column_num),
        );
    }
}
