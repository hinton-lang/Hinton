use super::{symbols::SymbolLoc, Compiler};
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
        if let Ok(s) = self.resolve_local_symbol(token, reassign) {
            return s;
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
    fn resolve_local_symbol(&mut self, token: &Token, reassign: bool) -> Result<SymbolLoc, ()> {
        if let Some(symbol_info) = self.current_func_scope_mut().s_table.resolve(&token.lexeme, true) {
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
}
