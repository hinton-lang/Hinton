use core::errors::error_at_tok;
use core::tokens::TokenIdx;

use crate::symbols::*;

macro_rules! new_scope_id {
  ($s:ident) => {{
    let current_table = $s.get_current_table_mut();
    current_table.max_scope_id += 1;
    current_table.max_scope_id
  }};
}

// Helper function only used by the visitor pattern defined below.
impl<'a> SymbolTableArena<'a> {
  fn visit_all(&mut self, nodes: &[ASTNodeIdx], data: SymbolScopeData) {
    nodes.iter().for_each(|node| self.ast_visit_node(*node, data))
  }

  fn visit_new_block(&mut self, block: &BlockNode, id: usize, depth: u16) {
    let prev_stack_len = self.get_current_table().stack_len;
    self.visit_all(&block.0, SymbolScopeData { id, depth });
    self.get_current_table_mut().stack_len = prev_stack_len;
  }

  fn visit_compound_id_decl(&mut self, decl: &CompoundIdDecl, kind: SymbolKind, data: SymbolScopeData) {
    match decl {
      CompoundIdDecl::Single(tok) => self.declare_id(*tok, data.depth, data.id, kind),
      CompoundIdDecl::Destruct(x) => {
        for member in x {
          if let DestructPatternMember::Id(tok) | DestructPatternMember::NamedWildcard(tok) = member {
            self.declare_id(*tok, data.depth, data.id, kind)
          }
        }
      }
    }
  }

  fn visit_compact_loop_heads(&mut self, heads: &[CompactForLoop], data: SymbolScopeData) {
    for h in heads {
      self.ast_visit_node(h.head.target, data);
      self.visit_compound_id_decl(&h.head.id, SymbolKind::Var, data);

      if let Some(condition) = h.cond {
        self.ast_visit_node(condition, data);
      }
    }
  }

  fn visit_params_list(&mut self, params: &[SingleParam], data: SymbolScopeData) {
    // First visit the values of all names params. That way,
    // we get references to variables outside the params list.
    params.iter().for_each(|p| {
      if let SingleParamKind::Named(n) = p.kind {
        self.ast_visit_node(n, data)
      }
    });
    // Then declare each of the parameter names.
    params.iter().for_each(|p| self.declare_id(p.name, 0, 0, SymbolKind::Var));
  }
}

// AST Visitor Pattern for the Symbol Table Arena
impl<'a> ASTVisitor<'a> for SymbolTableArena<'a> {
  type Res = ();
  type Data = SymbolScopeData;

  fn get_ast(&self) -> &'a ASTArena {
    self.ast
  }

  fn ast_visit_module(&mut self, node: &ASTModuleNode, data: Self::Data) {
    self.visit_all(&node.children, data);
  }

  fn ast_visit_block_stmt(&mut self, node: &BlockNode, data: Self::Data) -> Self::Res {
    let new_scope = new_scope_id![self];
    self.visit_new_block(node, new_scope, data.depth + 1);
  }

  fn ast_visit_del_stmt(&mut self, node: &ASTNodeIdx, data: Self::Data) -> Self::Res {
    self.ast_visit_node(*node, data)
  }

  fn ast_visit_export_decl(&mut self, _: &ASTImportExportNode, _: Self::Data) -> Self::Res {
    todo!("Implement export declaration.")
  }

  fn ast_visit_expr_stmt(&mut self, node: &ASTNodeIdx, data: Self::Data) -> Self::Res {
    self.ast_visit_node(*node, data)
  }

  fn ast_visit_if_stmt(&mut self, node: &ASTIfStmtNode, data: Self::Data) -> Self::Res {
    self.ast_visit_node(node.cond, data);

    let new_scope_1 = new_scope_id![self];
    self.visit_new_block(&node.true_branch, new_scope_1, data.depth + 1);

    let new_scope_2 = SymbolScopeData {
      id: new_scope_id![self],
      depth: data.depth + 1,
    };

    match &node.else_branch {
      ElseBranch::Block(x) => self.visit_new_block(x, new_scope_2.id, new_scope_2.depth),
      ElseBranch::IfStmt(x) => self.ast_visit_node(*x, new_scope_2),
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
    self.visit_new_block(&node.body, new_scope, data.depth + 1);

    for catch in &node.catchers {
      let new_scope = new_scope_id![self];

      if let Some(target) = &catch.target {
        self.resolve_id(target.error_class);

        if let Some(id) = &target.error_result {
          self.declare_id(*id, data.depth + 1, new_scope, SymbolKind::Var)
        }
      }

      self.visit_new_block(&catch.body, new_scope, data.depth + 1);
    }

    if let Some(finally) = &node.finally {
      let new_scope = new_scope_id![self];
      self.visit_new_block(finally, new_scope, data.depth + 1);
    }
  }

  fn ast_visit_var_const_decl(&mut self, node: &ASTVarConsDeclNode, data: Self::Data) -> Self::Res {
    self.ast_visit_node(node.val, data);
    let kind = if node.is_const { SymbolKind::Const } else { SymbolKind::Var };
    self.visit_compound_id_decl(&node.id, kind, data);
  }

  fn ast_visit_with_stmt(&mut self, node: &ASTWithStmtNode, data: Self::Data) -> Self::Res {
    let new_scope = new_scope_id![self];

    for h in &node.heads {
      self.ast_visit_node(h.expr, data);
      self.declare_id(h.id, data.depth + 1, new_scope, SymbolKind::Var)
    }

    self.visit_new_block(&node.body, new_scope, data.depth + 1);
  }

  fn ast_visit_break_stmt(&mut self, node: &ASTBreakStmtNode, data: Self::Data) -> Self::Res {
    if let TableLoopState::None = self.get_current_table().loop_ctx {
      self.errors.push(error_at_tok(
        self.tokens,
        node.token,
        "SyntaxError",
        "Can only break from within a loop.",
      ));
    }

    if let Some(val) = node.val {
      if let TableLoopState::While | TableLoopState::For = self.get_current_table().loop_ctx {
        self.errors.push(error_at_tok(
          self.tokens,
          node.token,
          "SyntaxError",
          "Can only break with expression from within a 'loop' expression.",
        ));
      }

      self.ast_visit_node(val, data)
    }
  }

  fn ast_visit_continue_stmt(&mut self, node: &TokenIdx, _: Self::Data) -> Self::Res {
    if let TableLoopState::None = self.get_current_table().loop_ctx {
      self.errors.push(error_at_tok(
        self.tokens,
        *node,
        "SyntaxError",
        "Can only continue from within a loop.",
      ));
    }
  }

  fn ast_visit_for_loop(&mut self, node: &ASTForLoopNode, data: Self::Data) -> Self::Res {
    let prev_loop_ctx = self.get_current_table().loop_ctx;
    self.get_current_table_mut().loop_ctx = TableLoopState::For;

    let new_scope = new_scope_id![self];
    self.ast_visit_node(node.head.target, data);
    self.visit_compound_id_decl(&node.head.id, SymbolKind::Var, data);
    self.visit_new_block(&node.body, new_scope, data.depth + 1);

    self.get_current_table_mut().loop_ctx = prev_loop_ctx;
  }

  fn ast_visit_loop_expr(&mut self, node: &ASTLoopExprNode, data: Self::Data) -> Self::Res {
    let prev_loop_ctx = self.get_current_table().loop_ctx;
    self.get_current_table_mut().loop_ctx = TableLoopState::Loop;

    let scope_id = new_scope_id![self];

    if let Some(count) = node.count {
      self.declare_id(count, data.depth + 1, scope_id, SymbolKind::Const)
    }

    self.visit_new_block(&node.body, scope_id, data.depth + 1);
    self.get_current_table_mut().loop_ctx = prev_loop_ctx;
  }

  fn ast_visit_while_loop(&mut self, node: &ASTWhileLoopNode, data: Self::Data) -> Self::Res {
    let prev_loop_ctx = self.get_current_table().loop_ctx;
    self.get_current_table_mut().loop_ctx = TableLoopState::While;

    self.ast_visit_node(node.cond, data);
    let new_scope = new_scope_id![self];

    if let Some(token) = node.let_id {
      self.declare_id(token, data.depth + 1, new_scope, SymbolKind::Var)
    }

    self.visit_new_block(&node.body, new_scope, data.depth + 1);
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
    self.declare_id(class.name, data.depth, data.id, SymbolKind::Class);

    // Create a new scope for the class's body
    let new_data = SymbolScopeData {
      depth: data.depth + 1,
      id: new_scope_id![self],
    };

    let prev_class_ctx = self.get_current_table().is_class_ctx;
    self.get_current_table_mut().is_class_ctx = true;

    // Visit the class's extends and implements
    class.extends.iter().for_each(|e| self.resolve_id(*e));
    class.impls.iter().for_each(|i| self.resolve_id(*i));

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
      self.declare_id(param.param.name, new_data.depth, new_data.id, decl_kind);
    }

    // Visit the init block
    if let Some(init) = &class.init {
      let new_scope = new_scope_id![self];
      self.visit_new_block(init, new_scope, new_data.depth + 1);
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

    // Declare the function in the current scope
    self.declare_id(node.name, data.depth, data.id, SymbolKind::Func);

    // Push the new table to the arena
    let prev_table = self.current_table;
    self.arena.push(SymbolTable::new(
      Some(self.current_table),
      data.id,
      true,
      // If previous context was a class, then the new context should also be a class.
      self.get_current_table().is_class_ctx,
    ));

    // Make the new table the current table
    self.current_table = self.arena.len() - 1;

    // Declare the function in the new table
    self.declare_id(node.name, 0, 0, SymbolKind::Func);

    // Declare the parameters on the new table
    self.visit_params_list(&node.params, SymbolScopeData { depth: 0, id: 0 });

    // Compile the function's body in the new table
    self.visit_new_block(&node.body, 0, 0);

    // Return to the previous table
    self.current_table = prev_table;
  }

  fn ast_visit_lambda(&mut self, node: &ASTLambdaNode, data: Self::Data) -> Self::Res {
    let prev_table = self.current_table;

    // Push the new table to the arena
    self.arena.push(SymbolTable::new(
      Some(self.current_table),
      data.id,
      true,
      // If previous context was a class, then the new context should also be a class.
      self.get_current_table().is_class_ctx,
    ));

    // Make the new table the current table
    self.current_table = self.arena.len() - 1;

    let new_data = SymbolScopeData { depth: 0, id: 0 };
    self.visit_params_list(&node.params, new_data);

    // Compile the lambda's body in the new table
    match &node.body {
      LambdaBody::Expr(e) => self.ast_visit_node(*e, new_data),
      LambdaBody::Block(b) => self.visit_new_block(b, 0, 0),
    }

    // Return to the previous table
    self.current_table = prev_table;
  }

  fn ast_visit_return_stmt(&mut self, node: &ASTReturnStmtNode, data: Self::Data) -> Self::Res {
    if !self.get_current_table().is_func_ctx {
      self.errors.push(error_at_tok(
        self.tokens,
        node.token,
        "SyntaxError",
        "Can only return from within a function body.",
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
    self.resolve_id(node.member);
  }

  fn ast_visit_reassignment(&mut self, node: &ASTReassignmentNode, data: Self::Data) -> Self::Res {
    self.ast_visit_node(node.target, data);
    self.ast_visit_node(node.value, data);
  }

  fn ast_visit_spread_expr(&mut self, node: &ASTNodeIdx, data: Self::Data) -> Self::Res {
    self.ast_visit_node(*node, data)
  }

  fn ast_visit_string_interpol(&mut self, nodes: &[ASTNodeIdx], data: Self::Data) -> Self::Res {
    self.visit_all(nodes, data);
  }

  fn ast_visit_ternary_conditional(&mut self, node: &ASTTernaryConditionalNode, data: Self::Data) -> Self::Res {
    self.ast_visit_node(node.condition, data);
    self.ast_visit_node(node.branch_true, data);
    self.ast_visit_node(node.branch_false, data);
  }

  fn ast_visit_unary_expr(&mut self, node: &ASTUnaryExprNode, data: Self::Data) -> Self::Res {
    self.ast_visit_node(node.operand, data)
  }

  fn ast_visit_array_literal(&mut self, nodes: &[ASTNodeIdx], data: Self::Data) -> Self::Res {
    self.visit_all(nodes, data);
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
    self.visit_all(nodes, data);
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

  fn ast_visit_tuple_literal(&mut self, nodes: &[ASTNodeIdx], data: Self::Data) -> Self::Res {
    self.visit_all(nodes, data);
  }

  fn ast_visit_id_literal(&mut self, node: &TokenIdx, _: Self::Data) -> Self::Res {
    self.resolve_id(*node)
  }

  fn ast_visit_self_literal(&mut self, node: &TokenIdx, _: Self::Data) -> Self::Res {
    if !self.get_current_table().is_class_ctx {
      self.errors.push(error_at_tok(
        self.tokens,
        *node,
        "SyntaxError",
        "Can only reference 'self' from within a class body.",
      ));
    }
  }

  fn ast_visit_super_literal(&mut self, node: &TokenIdx, _: Self::Data) -> Self::Res {
    if !self.get_current_table().is_class_ctx {
      self.errors.push(error_at_tok(
        self.tokens,
        *node,
        "SyntaxError",
        "Can only reference 'super' from within a class body.",
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
