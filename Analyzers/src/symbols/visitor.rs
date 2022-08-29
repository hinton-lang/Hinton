use core::errors::{error_at_tok, ErrMsg};
use core::tokens::TokenIdx;

use crate::scope;
use crate::symbols::*;

/// Generates a unique a id for a new block scope.
macro_rules! new_scope_id {
  ($s:ident) => {{
    let current_table = $s.get_current_table_mut();
    current_table.max_scope_id += 1;
    current_table.max_scope_id
  }};
}

// Helper function only used by the visitor pattern defined below.
impl<'a> SymbolTableArena<'a> {
  fn visit_new_block(&mut self, block: &BlockNode, data: SymbolScope) {
    let prev_stack_len = self.get_current_table().stack_len;
    self.ast_visit_all(&block.children, data);
    self.get_current_table_mut().stack_len = prev_stack_len;
  }

  fn visit_compound_id_decl(&mut self, decl: &CompoundIdDecl, kind: SymbolKind, data: SymbolScope) {
    match decl {
      CompoundIdDecl::Single(tok) => self.declare_id(*tok, kind, data),
      CompoundIdDecl::Unpack(x) => {
        for member in &x.decls {
          if let UnpackPatternMember::Id(tok) | UnpackPatternMember::NamedWildcard(tok) = member {
            self.declare_id(*tok, kind, data)
          }
        }
      }
    }
  }

  fn visit_compact_loop_heads(&mut self, heads: &[CompactForLoop], data: SymbolScope) {
    for h in heads {
      self.ast_visit_node(h.head.target, data);
      self.visit_compound_id_decl(&h.head.id, SymbolKind::Var, data);

      if let Some(condition) = h.cond {
        self.ast_visit_node(condition, data);
      }
    }
  }
}

// AST Visitor Pattern for the Symbol Table Arena
impl<'a> ASTVisitor<'a> for SymbolTableArena<'a> {
  type Res = ();
  type Data = SymbolScope;

  fn get_ast(&self) -> &'a ASTArena {
    self.ast
  }

  fn ast_visit_module(&mut self, node: &ASTModuleNode, data: Self::Data) {
    self.ast_visit_all(&node.children, data);
  }

  fn ast_visit_block_stmt(&mut self, node: &BlockNode, data: Self::Data) -> Self::Res {
    let new_scope = new_scope_id![self];
    self.visit_new_block(node, scope![new_scope, data.depth + 1]);

    // Mark all variables in the scope as "out of scope" so
    // they are unreachable by sibling scopes
    let symbols = &mut self.get_current_table_mut().table.symbols;
    symbols.iter_mut().filter(|s| s.scope.id == new_scope).for_each(|s| s.is_out_of_scope = true)
  }

  fn ast_visit_del_stmt(&mut self, node: &ASTNodeIdx, data: Self::Data) -> Self::Res {
    self.ast_visit_node(*node, data)
  }

  fn ast_visit_export_decl(&mut self, _: &ASTImportExportNode, _: Self::Data) -> Self::Res {
    todo!("Implement export declaration.")
  }

  fn ast_visit_expr_stmt(&mut self, node: &ASTExprStmt, data: Self::Data) -> Self::Res {
    self.ast_visit_node(node.expr, data)
  }

  fn ast_visit_if_stmt(&mut self, node: &ASTIfStmtNode, data: Self::Data) -> Self::Res {
    self.ast_visit_node(node.cond, data);

    let new_scope = new_scope_id![self];
    self.visit_new_block(&node.true_branch, scope![new_scope, data.depth + 1]);

    let new_data = scope![new_scope_id![self], data.depth + 1];
    match &node.else_branch {
      ElseBranch::Block(x) => self.visit_new_block(x, new_data),
      ElseBranch::IfStmt(x) => self.ast_visit_node(*x, new_data),
      ElseBranch::None => {}
    }
  }

  fn ast_visit_import_decl(&mut self, _: &ASTImportExportNode, _: Self::Data) -> Self::Res {
    todo!("Implement import declaration.")
  }

  fn ast_visit_throw_stmt(&mut self, node: &ASTNodeIdx, data: Self::Data) -> Self::Res {
    self.ast_visit_node(*node, data)
  }

  fn ast_visit_try_catch_finally(&mut self, node: &ASTTryCatchFinallyNode, data: Self::Data) -> Self::Res {
    let new_scope = new_scope_id![self];
    self.visit_new_block(&node.body, scope![new_scope, data.depth + 1]);

    for catch in &node.catchers {
      let new_scope = new_scope_id![self];

      if let Some(target) = &catch.target {
        // Look for the symbol outside of the catch's scope.
        self.resolve_id(target.error_class, self.current_table, false, false);

        if let Some(id) = &target.error_result {
          self.declare_id(*id, SymbolKind::Var, scope![new_scope, data.depth + 1])
        }
      }

      self.visit_new_block(&catch.body, scope![new_scope, data.depth + 1]);
    }

    if let Some(finally) = &node.finally {
      let new_scope = new_scope_id![self];
      self.visit_new_block(finally, scope![new_scope, data.depth + 1]);
    }
  }

  fn ast_visit_var_const_decl(&mut self, node: &ASTVarConsDeclNode, data: Self::Data) -> Self::Res {
    self.ast_visit_node(node.val, data);
    let kind = if node.is_const { SymbolKind::Const } else { SymbolKind::Var };
    self.visit_compound_id_decl(&node.id, kind, data);
  }

  fn ast_visit_with_stmt(&mut self, node: &ASTWithStmtNode, data: Self::Data) -> Self::Res {
    let new_scope = scope![new_scope_id![self], data.depth + 1];

    for h in &node.heads {
      self.ast_visit_node(h.expr, data);
      self.declare_id(h.id, SymbolKind::Var, new_scope);
    }

    self.visit_new_block(&node.body, new_scope);
  }

  fn ast_visit_break_stmt(&mut self, node: &ASTBreakStmtNode, data: Self::Data) -> Self::Res {
    if let TableLoopState::None = self.get_current_table().loop_ctx {
      self.errors.push(error_at_tok(
        node.token,
        ErrMsg::Syntax("Can only break from within a loop.".to_string()),
        None,
      ));
    }

    if let Some(val) = node.cond {
      self.ast_visit_node(val, data)
    }
  }

  fn ast_visit_continue_stmt(&mut self, node: &ASTContinueStmtNode, data: Self::Data) -> Self::Res {
    if let TableLoopState::None = self.get_current_table().loop_ctx {
      let err_msg = "Can only continue from within a loop.";
      self.errors.push(error_at_tok(node.token, ErrMsg::Syntax(err_msg.to_string()), None));
    }

    if let Some(val) = node.cond {
      self.ast_visit_node(val, data)
    }
  }

  fn ast_visit_for_loop(&mut self, node: &ASTForLoopNode, data: Self::Data) -> Self::Res {
    let prev_loop_ctx = self.get_current_table().loop_ctx;
    self.get_current_table_mut().loop_ctx = TableLoopState::For;

    self.ast_visit_node(node.head.target, data);

    let new_data = scope![new_scope_id![self], data.depth + 1];
    self.visit_compound_id_decl(&node.head.id, SymbolKind::Var, new_data);
    self.visit_new_block(&node.body, new_data);

    self.get_current_table_mut().loop_ctx = prev_loop_ctx;
  }

  fn ast_visit_loop_stmt(&mut self, node: &ASTLoopExprNode, data: Self::Data) -> Self::Res {
    let prev_loop_ctx = self.get_current_table().loop_ctx;
    self.get_current_table_mut().loop_ctx = TableLoopState::Loop;

    let new_data = scope![new_scope_id![self], data.depth + 1];

    self.visit_new_block(&node.body, new_data);
    self.get_current_table_mut().loop_ctx = prev_loop_ctx;
  }

  fn ast_visit_while_loop(&mut self, node: &ASTWhileLoopNode, data: Self::Data) -> Self::Res {
    let prev_loop_ctx = self.get_current_table().loop_ctx;
    self.get_current_table_mut().loop_ctx = TableLoopState::While;

    self.ast_visit_node(node.cond, data);

    let new_data = scope![new_scope_id![self], data.depth + 1];
    if let Some(token) = node.let_id {
      self.declare_id(token, SymbolKind::Var, new_data)
    }

    self.visit_new_block(&node.body, new_data);
    self.get_current_table_mut().loop_ctx = prev_loop_ctx;
  }

  fn ast_visit_class_decl(&mut self, node: &ASTClassIdx, data: Self::Data) -> Self::Res {
    // TODO: Revise this implementation.
    // This is not the final implementation of how classes will be compiled
    // or statically analyzed. Revise this again once there is a final model
    // of how classes will be compiled and represented at runtime.

    let class = self.ast.get_class(*node);

    // Visit the claas's decorators
    class.decor.iter().for_each(|d| self.ast_visit_node(d.0, data));

    // Declare the class in the current scope
    self.declare_id(class.name, SymbolKind::Class, data);

    // Create a new scope for the class's body
    let new_data = scope![new_scope_id![self], data.depth + 1];

    let prev_class_ctx = self.get_current_table().is_class_ctx;
    self.get_current_table_mut().is_class_ctx = true;

    // Look for the class's extends and implements outside of the class's body.
    class.extends.iter().for_each(|e| self.resolve_id(*e, self.current_table, false, false));
    class.impls.iter().for_each(|i| self.resolve_id(*i, self.current_table, false, false));

    // First visit the values of all names params. That way,
    // we get references to variables outside the params list.
    class.params.iter().for_each(|p| {
      if let SingleParamKind::Named(n) = p.param.kind {
        self.ast_visit_node(n, new_data)
      }
    });

    // Then declare the parameter names for the current class
    for param in &class.params {
      param.decor.iter().for_each(|d| self.ast_visit_node(d.0, data));
      let decl_kind = if param.is_const { SymbolKind::Const } else { SymbolKind::Var };
      self.declare_id(param.param.name, decl_kind, new_data);
    }

    // Visit the init block
    if let Some(init) = &class.init {
      let new_scope = new_scope_id![self];
      self.visit_new_block(init, scope![new_scope, data.depth + 1]);
    }

    // Visit the class members
    for member in &class.members {
      match &member.member {
        ClassMemberKind::Var(d, n) | ClassMemberKind::Const(d, n) => {
          d.iter().for_each(|d| self.ast_visit_node(d.0, new_data));
          self.ast_visit_node(*n, new_data)
        }
        ClassMemberKind::Func(f) => self.ast_visit_node(*f, new_data),
      }
    }

    self.get_current_table_mut().is_class_ctx = prev_class_ctx;
  }

  fn ast_visit_func_decl(&mut self, node: &ASTFuncDeclNode, data: Self::Data) -> Self::Res {
    // Visit the function's decorators
    node.decor.iter().for_each(|d| self.ast_visit_node(d.0, data));

    // First visit the values of all named params. That way,
    // we get references to variables outside the params list.
    node.params.iter().for_each(|p| {
      if let SingleParamKind::Named(n) = p.kind {
        self.ast_visit_node(n, data)
      }
    });

    // Declare the function in the current scope
    self.declare_id(node.name, SymbolKind::Func, data);

    // Push the new table to the arena
    let prev_table = self.current_table;
    self.arena.push(SymbolTableBuilder::new(
      Some(self.current_table),
      true,
      // If previous context was a class, then the new context should also be a class.
      self.get_current_table().is_class_ctx,
    ));

    // Make the new table the current table
    self.current_table = self.arena.len() - 1;

    // Declare the function in the new table
    self.declare_id(node.name, SymbolKind::Func, scope![0, 0]);

    // Declare the parameters on the new table
    node.params.iter().for_each(|p| self.declare_id(p.name, SymbolKind::Param, scope![0, 0]));

    // Compile the function's body in the new table
    self.visit_new_block(&node.body, scope![0, 0]);

    // Return to the previous table
    self.current_table = prev_table;
  }

  fn ast_visit_lambda(&mut self, node: &ASTLambdaNode, data: Self::Data) -> Self::Res {
    let prev_table = self.current_table;

    // First visit the values of all named params. That way,
    // we get references to variables outside the params list.
    node.params.iter().for_each(|p| {
      if let SingleParamKind::Named(n) = p.kind {
        self.ast_visit_node(n, data)
      }
    });

    // Push the new table to the arena
    self.arena.push(SymbolTableBuilder::new(
      Some(self.current_table),
      true,
      // If previous context was a class, then the new context should also be a class.
      self.get_current_table().is_class_ctx,
    ));

    // Make the new table the current table
    self.current_table = self.arena.len() - 1;

    let new_data = scope![0, 0];
    // Declare the function's parameters
    node.params.iter().for_each(|p| self.declare_id(p.name, SymbolKind::Param, new_data));

    // Compile the lambda's body in the new table
    match &node.body {
      LambdaBody::Expr(e) => self.ast_visit_node(*e, new_data),
      LambdaBody::Block(b) => self.visit_new_block(b, scope![0, 0]),
    }

    // Return to the previous table
    self.current_table = prev_table;
  }

  fn ast_visit_return_stmt(&mut self, node: &ASTReturnStmtNode, data: Self::Data) -> Self::Res {
    if !self.get_current_table().is_func_ctx {
      self.errors.push(error_at_tok(
        node.token,
        ErrMsg::Syntax("Can only return from within a function body.".to_string()),
        None,
      ));
    }

    self.ast_visit_node(node.val, data)
  }

  fn ast_visit_yield_stmt(&mut self, node: &ASTNodeIdx, data: Self::Data) -> Self::Res {
    self.ast_visit_node(*node, data)
  }

  fn ast_visit_binary_expr(&mut self, node: &ASTBinaryExprNode, data: Self::Data) -> Self::Res {
    self.ast_visit_node(node.left, data);
    self.ast_visit_node(node.right, data);
  }

  fn ast_visit_call_expr(&mut self, node: &ASTCallExprNode, data: Self::Data) -> Self::Res {
    self.ast_visit_node(node.target, data);

    for arg in &node.args {
      match arg {
        CallArg::Val(a) | CallArg::Rest(a) => self.ast_visit_node(*a, data),
        // Named params are resolved at runtime.
        // This could be done at compile time if Hinton supported static typing.
        CallArg::Named { value, .. } => self.ast_visit_node(*value, data),
      }
    }
  }

  fn ast_visit_member_access(&mut self, node: &ASTMemberAccessNode, data: Self::Data) -> Self::Res {
    self.ast_visit_node(node.target, data);
    // NOTE: The member name is resolved at runtime.
    // This could be done at compile time if Hinton supported static typing.
  }

  fn ast_visit_reassignment(&mut self, node: &ASTReassignmentNode, data: Self::Data) -> Self::Res {
    match self.ast.get(node.target) {
      // If target is a single identifier, resolve it now for reassignment.
      ASTNodeKind::IdLiteral(i) => self.resolve_id(*i, self.current_table, true, false),
      // Everything else gets checked at runtime.
      _ => self.ast_visit_node(node.target, data),
    }

    self.ast_visit_node(node.value, data);
  }

  fn ast_visit_spread_expr(&mut self, node: &ASTNodeIdx, data: Self::Data) -> Self::Res {
    self.ast_visit_node(*node, data)
  }

  fn ast_visit_string_interpol(&mut self, nodes: &ASTStringInterpol, data: Self::Data) -> Self::Res {
    self.ast_visit_all(&nodes.parts, data);
  }

  fn ast_visit_ternary_conditional(&mut self, node: &ASTTernaryConditionalNode, data: Self::Data) -> Self::Res {
    self.ast_visit_node(node.condition, data);
    self.ast_visit_node(node.branch_true, data);
    self.ast_visit_node(node.branch_false, data);
  }

  fn ast_visit_unary_expr(&mut self, node: &ASTUnaryExprNode, data: Self::Data) -> Self::Res {
    self.ast_visit_node(node.operand, data)
  }

  fn ast_visit_array_literal(&mut self, node: &ASTArrayLiteralNode, data: Self::Data) -> Self::Res {
    self.ast_visit_all(&node.values, data);
  }

  fn ast_visit_array_slice(&mut self, node: &ASTArraySliceNode, data: Self::Data) -> Self::Res {
    if let Some(lower) = node.lower {
      self.ast_visit_node(lower, data)
    }

    if let Some(upper) = node.upper {
      self.ast_visit_node(upper, data)
    }

    if let Some(step) = node.step {
      self.ast_visit_node(step, data)
    }
  }

  fn ast_visit_compact_arr_or_tpl(&mut self, node: &ASTCompactArrOrTplNode, data: Self::Data) -> Self::Res {
    self.visit_compact_loop_heads(&node.heads, data);
    self.ast_visit_node(node.body, data);
  }

  fn ast_visit_compact_dict(&mut self, node: &ASTCompactDictNode, data: Self::Data) -> Self::Res {
    self.visit_compact_loop_heads(&node.heads, data);
    self.ast_visit_node(node.body, data);
  }

  fn ast_visit_dict_key_val_pair(&mut self, node: &(ASTNodeIdx, ASTNodeIdx), data: Self::Data) -> Self::Res {
    self.ast_visit_node(node.0, data);
    self.ast_visit_node(node.1, data);
  }

  fn ast_visit_dict_literal(&mut self, nodes: &[ASTNodeIdx], data: Self::Data) -> Self::Res {
    self.ast_visit_all(nodes, data);
  }

  fn ast_visit_evaluated_dict_key(&mut self, node: &ASTNodeIdx, data: Self::Data) -> Self::Res {
    self.ast_visit_node(*node, data)
  }

  fn ast_visit_indexing(&mut self, node: &ASTIndexingNode, data: Self::Data) -> Self::Res {
    self.ast_visit_node(node.target, data);
    node.indexers.iter().for_each(|n| self.ast_visit_node(*n, data));
  }

  fn ast_visit_repeat_literal(&mut self, node: &ASTRepeatLiteralNode, data: Self::Data) -> Self::Res {
    self.ast_visit_node(node.count, data);
    self.ast_visit_node(node.value, data);
  }

  fn ast_visit_tuple_literal(&mut self, node: &ASTTupleLiteralNode, data: Self::Data) -> Self::Res {
    self.ast_visit_all(&node.values, data);
  }

  fn ast_visit_id_literal(&mut self, node: &TokenIdx, _: Self::Data) -> Self::Res {
    self.resolve_id(*node, self.current_table, false, false)
  }

  fn ast_visit_self_literal(&mut self, node: &TokenIdx, _: Self::Data) -> Self::Res {
    if !self.get_current_table().is_class_ctx {
      self.errors.push(error_at_tok(
        *node,
        ErrMsg::Reference("Can only reference 'self' from within a class body.".to_string()),
        None,
      ));
    }
  }

  fn ast_visit_super_literal(&mut self, node: &TokenIdx, _: Self::Data) -> Self::Res {
    if !self.get_current_table().is_class_ctx {
      self.errors.push(error_at_tok(
        *node,
        ErrMsg::Reference("Can only reference 'super' from within a class body.".to_string()),
        None,
      ));
    }
  }

  // We do not do anything with value literals
  fn ast_visit_false_literal(&mut self, _: &TokenIdx, _: Self::Data) -> Self::Res {}
  fn ast_visit_none_literal(&mut self, _: &TokenIdx, _: Self::Data) -> Self::Res {}
  fn ast_visit_num_literal(&mut self, _: &TokenIdx, _: Self::Data) -> Self::Res {}
  fn ast_visit_string_literal(&mut self, _: &TokenIdx, _: Self::Data) -> Self::Res {}
  fn ast_visit_true_literal(&mut self, _: &TokenIdx, _: Self::Data) -> Self::Res {}
}
