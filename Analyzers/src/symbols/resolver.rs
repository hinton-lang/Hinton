use core::errors::error_at_tok;
use core::tokens::TokenIdx;

use crate::symbols::*;

impl<'a> SymbolTableArena<'a> {
  /// Declares the given identifier in the current symbol table.
  ///
  /// # Arguments
  ///
  /// * `token_idx`: The index of the token associated with this declaration.
  /// * `depth`: The scope depth of the declaration relative to the nearest function.
  /// * `scope_id`: The scope id of the declaration relative to the nearest function.
  /// * `kind`: The kind of declaration.
  pub(super) fn declare_id(&mut self, token_idx: TokenIdx, depth: u16, scope_id: usize, kind: SymbolKind) {
    // Check that the symbol isn't already declared in the current scope id.
    for symbol in self.get_current_table().symbols.iter().filter(|s| s.scope.id == scope_id) {
      if self.tokens.lexeme(symbol.token_idx) == self.tokens.lexeme(token_idx) {
        let err_msg = &format!(
          "Can only declare identifier '{}' once per scope.",
          self.tokens.lexeme(token_idx)
        );
        self.errors.push(error_at_tok(self.tokens, token_idx, "ReferenceError", err_msg));
        return;
      }
    }

    // Determine the location of the variable.
    let loc = if self.current_table == 0 && depth == 0 {
      // Check the bounds of the globals vector
      if self.globals_len == u16::MAX as usize {
        let err_msg = "Too many global declarations.";
        self.errors.push(error_at_tok(self.tokens, token_idx, "MaxCapacity", err_msg));
      }

      self.globals_len += 1;
      SymbolLoc::Global(self.globals_len - 1)
    } else {
      // Check the bounds of the locals stack
      if self.get_current_table().stack_len == u16::MAX as usize {
        let err_msg = "Too many local declarations.";
        self.errors.push(error_at_tok(self.tokens, token_idx, "MaxCapacity", err_msg));
      }

      self.get_current_table_mut().stack_len += 1;
      SymbolLoc::Stack(self.get_current_table().stack_len - 1)
    };

    // Add the symbol to the current table
    self.get_current_table_mut().push(Symbol {
      token_idx,
      kind,
      scope: SymbolScopeData { id: scope_id, depth },
      has_reference: false,
      loc,
    });
  }

  pub fn resolve_id(&mut self, id: TokenIdx) {
    todo!();
  }
}
