use core::errors::{error_at_tok, ErrMsg};
use core::tokens::TokenIdx;
use objects::native_func_obj::NATIVES;

use crate::symbols::*;

impl<'a> SymbolTableArena<'a> {
  /// Declares the given identifier in the current symbol table.
  ///
  /// # Arguments
  ///
  /// * `token_idx`: The index of the token associated with this declaration.
  /// * `kind`: The kind of declaration.
  /// * `data`: The scope data for the declaration.
  pub(super) fn declare_id(&mut self, token_idx: TokenIdx, kind: SymbolKind, data: SymbolScope) {
    // Check that the symbol isn't already declared in the current scope id.
    for symbol in self.get_current_table().table.symbols.iter().filter(|s| s.scope.id == data.id) {
      if self.tokens.lexeme(symbol.token_idx) == self.tokens.lexeme(token_idx) {
        let err_msg = format!(
          "Duplicate declaration of identifier '{}'.",
          self.tokens.lexeme(token_idx)
        );

        let kind = match symbol.kind {
          SymbolKind::Var => "variable",
          SymbolKind::Const => "constant",
          SymbolKind::Func => "function",
          SymbolKind::Class => "class",
          SymbolKind::Method => "method",
          SymbolKind::Param => "parameter",
        };

        let tok_loc = self.tokens.loc(symbol.token_idx);
        let hint = format!(
          "Identifier previously declared as a {} on line {}, column {}.",
          kind,
          tok_loc.line_num,
          tok_loc.col_start()
        );

        self.errors.push(error_at_tok(token_idx, ErrMsg::Duplication(err_msg), Some(hint)));
        return;
      }
    }

    // Determine the location of the declaration.
    let loc = if self.current_table == 0 && data.depth == 0 {
      // Check the bounds of the globals vector
      if self.globals_len == u16::MAX as usize {
        let err_msg = "Too many global declarations.".to_string();
        self.errors.push(error_at_tok(token_idx, ErrMsg::MaxCapacity(err_msg), None));
      }

      self.globals_len += 1;
      SymLoc::Global(self.globals_len - 1)
    } else {
      // Check the bounds of the locals stack
      if self.get_current_table().stack_len == u16::MAX as usize {
        let err_msg = "Too many local declarations.".to_string();
        self.errors.push(error_at_tok(token_idx, ErrMsg::MaxCapacity(err_msg), None));
      }

      self.get_current_table_mut().stack_len += 1;
      SymLoc::Stack(self.get_current_table().stack_len - 1)
    };

    // Add the symbol to the current table
    self.get_current_table_mut().push(Symbol {
      token_idx,
      kind,
      scope: data,
      has_reference: false,
      loc,
      is_out_of_scope: false,
    });
  }

  /// Tries to resolve the location of an identifier declaration.
  ///
  /// # Arguments
  ///
  /// * `id`: The identifier's token index.
  /// * `func`: The current function to look for the identifier.
  /// * `is_reassign`: Whether or not the resolution if for a reassignment.
  /// * `is_captured`: Whether or not the symbol will be closed-over by the current function.
  ///
  /// # Returns: `()`
  pub fn resolve_id(&mut self, id: TokenIdx, func: SymbolTableIdx, is_reassign: bool, is_captured: bool) {
    let resolved = self.resolve(id, func, is_reassign, is_captured);

    match resolved {
      // Could not resolve the identifier
      SymRes::None => {
        let name = self.tokens.lexeme(id);

        let err = if is_reassign {
          let msg = format!("Cannot assign to undeclared identifier '{}'.", name);
          let hint = Some("Did you mean to bind the name to a 'let' or 'const' declaration here?".to_string());
          error_at_tok(id, ErrMsg::Reassignment(msg), hint)
        } else {
          let msg = format!("Use of undeclared identifier '{}'.", name);
          error_at_tok(id, ErrMsg::Reference(msg), None)
        };

        self.errors.push(err);
      }
      // Identifier successfully resolved (local, up-val, global, native, or primitive).
      _ => self.get_current_table_mut().table.resolved.push((id, resolved)),
    }
  }

  /// Recursively tries to resolve the location of an identifier declaration.
  ///
  /// # Arguments
  ///
  /// * `id`: The identifier's token index.
  /// * `func`: The current function to look for the identifier.
  /// * `is_reassign`: Whether or not the resolution if for a reassignment.
  /// * `is_captured`: Whether or not the symbol will be closed-over by the current function.
  ///
  /// # Returns:
  /// ```SymRes```
  fn resolve(&mut self, id: TokenIdx, func: SymbolTableIdx, is_reassign: bool, is_captured: bool) -> SymRes {
    let current_func = &mut self.arena[func];
    let tok_name = self.tokens.lexeme(id);

    // Find all in-scope symbols (including globals) with the name
    // we're interested in, and take the one in the most recent scope.
    let is_candidate = |s: &Symbol| !s.is_out_of_scope && self.tokens.lexeme(s.token_idx) == tok_name;
    if let Some(symbol) = current_func.table.symbols.iter_mut().filter(|s| is_candidate(s)).last() {
      symbol.has_reference = true;

      // Check that the symbol can be reassigned. Notice that after emitting the error
      // we still return the found symbol. If we don't, the `resolve_id` function
      // would emit another error saying the variable was not found, which is not true.
      if is_reassign && !matches![symbol.kind, SymbolKind::Var | SymbolKind::Param] {
        let err_msg = "Cannot reassign to immutable declaration.".to_string();

        let name = match symbol.kind {
          SymbolKind::Const => "constant",
          SymbolKind::Func => "function",
          SymbolKind::Class => "class",
          SymbolKind::Method => "class method",
          SymbolKind::Var | SymbolKind::Param => unreachable!(),
        };

        let sl = self.tokens.loc(symbol.token_idx);
        let hint = format!(
          "This identifier refers to a {} declared on line {}, column {}.",
          name,
          sl.line_num,
          sl.col_start()
        );

        self.errors.push(error_at_tok(id, ErrMsg::Reassignment(err_msg), Some(hint)));
      }

      // Note: Casting to u16 here is ok because by the time we resolve this id,
      // the `declare_id` will have already checked the declaration limits.
      return match symbol.loc {
        SymLoc::Global(x) => SymRes::Global(x as u16),
        SymLoc::Stack(x) if !is_captured => SymRes::Stack(x as u16),
        // TODO: Implement closure up-values.
        SymLoc::Stack(x) => SymRes::Stack(x as u16),
      };
    }

    match current_func.table.parent_table {
      // Look for the symbol in parent functions
      Some(table) => self.resolve(id, table, is_reassign, true),
      // Look for the symbol in the native functions scope
      None => {
        if let Some(pos) = NATIVES.iter().position(|x| x.name == tok_name) {
          if is_reassign {
            let err_msg = format!("Native function '{}' cannot be reassigned.", tok_name);
            let hint = "Try binding the name to a 'let' or 'const' declaration";
            self.errors.push(error_at_tok(id, ErrMsg::Reassignment(err_msg), Some(hint.into())));
          }

          SymRes::Native(pos as u16)
        } else {
          SymRes::None
        }
      }
    }
  }
}
