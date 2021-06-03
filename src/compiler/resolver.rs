use super::{
    symbols::{Symbol, SymbolLoc},
    Compiler, UpValue,
};
use crate::{
    bytecode::OpCode, compiler::symbols::SymbolType, errors::CompilerErrorType, lexer::tokens::Token,
    natives, objects::Object,
};

impl Compiler {
    /// Looks for a symbol with the given token name in current script.
    ///
    /// ## Arguments
    /// * `token` – A reference to the token (symbol name) related to the symbol.
    /// * `reassign` – Wether of not we are resolving the symbol for the purpose of reassignment.
    ///
    /// ## Returns
    /// * `SymbolLoc` – The location (if found) and resolution type of the symbol.
    pub(super) fn resolve_symbol(&mut self, token: &Token, reassign: bool) -> SymbolLoc {
        // Look for the symbol in the local scope of the current function
        if let Ok(s) = self.resolve_local_symbol(token, reassign, self.functions.len() - 1) {
            return s;
        }

        // If we are in a function within a function, then we also look for symbols
        // in the scope of the parent function to create upValues & closures.
        if self.functions.len() > 2 {
            if let Ok(s) = self.resolve_up_value(token, reassign, self.functions.len() - 2) {
                return s;
            }
        }

        // Looks for the symbol in the global scope of the current script
        if let Ok(s) = self.resolve_global_symbol(token, reassign) {
            return s;
        }

        // Look for the identifier in the natives
        if natives::check_is_native(&token.lexeme) {
            if reassign {
                self.error_at_token(
                    token,
                    CompilerErrorType::Reassignment,
                    &format!("Cannot modify native function '{}'.", token.lexeme),
                );
            } else {
                self.add_literal_to_pool(Object::String(token.lexeme.clone()), token, true);
                self.emit_op_code(OpCode::LoadNative, (token.line_num, token.column_num));
            }

            return SymbolLoc::Native;
        }

        // The symbol doesn't exist
        self.error_at_token(
            token,
            CompilerErrorType::Reference,
            &format!("Use of undeclared identifier '{}'.", token.lexeme),
        );

        SymbolLoc::None
    }

    /// Looks for a symbol with the given token name in current function scope.
    ///
    /// ## Arguments
    /// * `token` – A reference to the token (symbol name) related to the symbol.
    /// * `reassign` – Wether of not we are resolving the symbol for the purpose of reassignment.
    ///
    /// ## Returns
    /// * `Result<SymbolLoc, ()>` – The location (if found) and resolution type of the symbol.
    fn resolve_local_symbol(&mut self, token: &Token, reassign: bool, func: usize) -> Result<SymbolLoc, ()> {
        let func = &mut self.functions[func];

        if let Some(symbol_info) = func.s_table.resolve(&token.lexeme, true) {
            if !symbol_info.0.is_initialized {
                let sym_type = match symbol_info.0.symbol_type {
                    SymbolType::Variable => "variable",
                    SymbolType::Constant => "constant",
                    SymbolType::Function => "function",
                    _ => unreachable!("Symbol should have been initialized by now."),
                };

                self.error_at_token(
                    &token,
                    CompilerErrorType::Reference,
                    &format!(
                        "Cannot reference {} '{}' before it has been initialized.",
                        sym_type, token.lexeme
                    ),
                );

                // Return None here because a symbol should not be referenced
                // until it has been initialized.
                return Ok(SymbolLoc::None);
            }

            if reassign {
                let sym_type = match &symbol_info.0.symbol_type {
                    SymbolType::Constant => "Constants",
                    SymbolType::Function => "Functions",
                    SymbolType::Class => "Classes",
                    SymbolType::Enum => "Enums",
                    // Only variables & parameters are re-assignable
                    SymbolType::Variable | SymbolType::Parameter => {
                        return Ok(SymbolLoc::Local(symbol_info.0, symbol_info.1))
                    }
                };

                self.error_at_token(
                    token,
                    CompilerErrorType::Reassignment,
                    &format!("{} are immutable.", sym_type),
                );

                return Ok(SymbolLoc::None);
            }

            return Ok(SymbolLoc::Local(symbol_info.0, symbol_info.1));
        }

        Err(())
    }

    /// Looks for a symbol with the given token name in the global scope.
    ///
    /// ## Arguments
    /// * `token` – A reference to the token (symbol name) related to the symbol.
    /// * `reassign` – Wether of not we are resolving the symbol for the purpose of reassignment.
    ///
    /// ## Returns
    /// * `Result<SymbolLoc, ()>` – The location (if found) and resolution type of the symbol.
    fn resolve_global_symbol(&mut self, token: &Token, reassign: bool) -> Result<SymbolLoc, ()> {
        if let Some(symbol_info) = self.globals.resolve(&token.lexeme, true) {
            if !symbol_info.0.is_initialized {
                let sym_type = match symbol_info.0.symbol_type {
                    SymbolType::Variable => "variable",
                    SymbolType::Constant => "constant",
                    SymbolType::Function => "function",
                    _ => unreachable!("Symbol should have been initialized by now."),
                };

                self.error_at_token(
                    &token,
                    CompilerErrorType::Reference,
                    &format!(
                        "Cannot reference {} '{}' before it has been initialized.",
                        sym_type, token.lexeme
                    ),
                );

                // Return None here because a symbol should not be referenced
                // until it has been initialized.
                return Ok(SymbolLoc::None);
            }

            if reassign {
                let sym_type = match &symbol_info.0.symbol_type {
                    SymbolType::Constant => "Constants",
                    SymbolType::Function => "Functions",
                    SymbolType::Class => "Classes",
                    SymbolType::Enum => "Enums",
                    // Only variables & parameters are re-assignable
                    SymbolType::Variable | SymbolType::Parameter => {
                        match self.add_literal_to_pool(Object::String(token.lexeme.clone()), &token, false) {
                            Some(idx) => return Ok(SymbolLoc::Global(symbol_info.0, idx as usize)),
                            None => return Ok(SymbolLoc::None),
                        }
                    }
                };

                self.error_at_token(
                    token,
                    CompilerErrorType::Reassignment,
                    &format!("{} are immutable.", sym_type),
                );

                return Ok(SymbolLoc::None);
            }

            return match self.add_literal_to_pool(Object::String(token.lexeme.clone()), &token, false) {
                Some(idx) => Ok(SymbolLoc::Global(symbol_info.0, idx as usize)),
                None => Ok(SymbolLoc::None),
            };
        }

        Err(())
    }

    /// Looks for a symbol with the given token name in the provided function scope index.
    /// This function executes with the assumption that it is being called by a child
    /// function scope to look for UpValues in the scope of its parent, and will recursively
    /// look for the symbol in scopes of parent functions for provided function scope index.
    ///
    /// ## Arguments
    /// * `token` – A reference to the token (symbol name) related to the symbol.
    /// * `reassign` – Wether of not we are resolving the symbol for the purpose of reassignment.
    /// * `func_idx` – The index of the function scope to start looking for the symbol.
    ///
    /// ## Returns
    /// * `Result<SymbolLoc, ()>` – The location (if found) and resolution type of the symbol.
    fn resolve_up_value(&mut self, token: &Token, reassign: bool, func_idx: usize) -> Result<SymbolLoc, ()> {
        if func_idx == 0 {
            return Err(());
        }

        // Look for the symbol in the local scope of the current function.
        // NOTE: By the time `self.resolve_up_value(...)` is called, the `self.resolve_symbol(...)`
        // function has already looked for local variables in the `current_func_scope`. So this
        // next call to `self.resolve_local_symbol(...)` is looking at symbols in the local scope
        // for the `self.functions.len() - 2` function scope. That is, the local scope of the parent
        // function of the parent function. Look at the call to `self.resolve_up_value(...)` in
        // `self.resolve_symbol(...)` to understand this better.
        if let Ok(s) = self.resolve_local_symbol(token, reassign, func_idx) {
            return match s {
                SymbolLoc::Local(s, p) => self.add_up_value(token, func_idx + 1, s, p, true),
                _ => unreachable!("SymbolLoc should have been a local symbol."),
            };
        }

        // Recursively look for the symbol in higher function scopes.
        if let Ok(s) = self.resolve_up_value(token, reassign, func_idx - 1) {
            return match s {
                SymbolLoc::UpValue(u, p) => self.add_up_value(token, func_idx + 1, u.symbol, p, false),
                _ => unreachable!("SymbolLoc should have been an up_value symbol."),
            };
        }

        return Err(());
    }

    fn add_up_value(
        &mut self,
        token: &Token,
        func_idx: usize,
        symbol: Symbol,
        index: usize,
        is_local: bool,
    ) -> Result<SymbolLoc, ()> {
        // Prevent creating repeated up_values
        for (index, up_val) in self.functions[func_idx].up_values.iter().enumerate() {
            if up_val.index == index && up_val.is_local == is_local && up_val.symbol.name == symbol.name {
                return Ok(SymbolLoc::UpValue(up_val.clone(), index));
            }
        }

        if self.functions[func_idx].up_values.len() >= u16::MAX as usize {
            self.error_at_token(
                token,
                CompilerErrorType::MaxCapacity,
                "Too many closure variables in function.",
            );
            return Err(());
        }

        let up_value = UpValue {
            symbol,
            index,
            is_local,
        };

        self.functions[func_idx].up_values.push(up_value.clone());
        self.functions[func_idx].function.up_val_count += 1;

        return Ok(SymbolLoc::UpValue(
            up_value,
            self.functions[func_idx].up_values.len() - 1,
        ));
    }
}
