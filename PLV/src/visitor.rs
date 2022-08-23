use crate::PLVJsonGenerator;
use core::ast::ASTNodeKind::*;
use core::ast::*;
use core::tokens::*;
use core::utils::{parse_float_lexeme, parse_int_from_lexeme_base, parse_int_lexeme, parse_scientific_literal_lexeme};
use serde_json::{json, Value};

impl<'a> PLVJsonGenerator<'a> {
  /// Converts a list of ASTNodes into their JSON representation.
  ///
  /// # Arguments
  ///
  /// * `nodes`: The nodes to convert to JSON.
  /// * `branch_name`: The name for each generated branch.
  ///
  /// # Returns:
  /// ```Vec<Value, Global>```
  fn ast_list_to_json(&mut self, nodes: &[ASTNodeIdx], branch_name: &'a str) -> Vec<Value> {
    nodes.iter().map(|x| self.ast_to_json(*x, branch_name)).collect()
  }

  /// Converts an AST node and its children to JSON.
  ///
  /// # Arguments
  ///
  /// * `idx`: The index of the node to be converted to JSON.
  /// * `branch_name`: The name of the branch, if any, for this node.
  ///
  /// # Returns:
  /// ```Value```
  pub fn ast_to_json(&mut self, node: ASTNodeIdx, data: &'a str) -> Value {
    let (name, mut attributes, children) = match self.get_ast().get(node) {
      Module(node) => self.ast_visit_module(node, data),
      ArrayLiteral(node) => self.ast_visit_array_literal(node, data),
      ArraySlice(node) => self.ast_visit_array_slice(node, data),
      BinaryExpr(node) => self.ast_visit_binary_expr(node, data),
      BlockStmt(node) => self.ast_visit_block_stmt(node, data),
      BreakStmt(node) => self.ast_visit_break_stmt(node, data),
      CallExpr(node) => self.ast_visit_call_expr(node, data),
      ClassDecl(node) => self.ast_visit_class_decl(node, data),
      CompactArrOrTpl(node) => self.ast_visit_compact_arr_or_tpl(node, data),
      CompactDict(node) => self.ast_visit_compact_dict(node, data),
      ContinueStmt(node) => self.ast_visit_continue_stmt(node, data),
      DelStmt(node) => self.ast_visit_del_stmt(node, data),
      DictKeyValPair(node) => self.ast_visit_dict_key_val_pair(node, data),
      DictLiteral(node) => self.ast_visit_dict_literal(node, data),
      EvaluatedDictKey(node) => self.ast_visit_evaluated_dict_key(node, data),
      ExportDecl(node) => self.ast_visit_export_decl(node, data),
      ExprStmt(node) => self.ast_visit_expr_stmt(node, data),
      FalseLiteral(node) => self.ast_visit_false_literal(node, data),
      ForLoop(node) => self.ast_visit_for_loop(node, data),
      FuncDecl(node) => self.ast_visit_func_decl(node, data),
      IdLiteral(node) => self.ast_visit_id_literal(node, data),
      IfStmt(node) => self.ast_visit_if_stmt(node, data),
      ImportDecl(node) => self.ast_visit_import_decl(node, data),
      Indexing(node) => self.ast_visit_indexing(node, data),
      Lambda(node) => self.ast_visit_lambda(node, data),
      LoopExpr(node) => self.ast_visit_loop_expr(node, data),
      MemberAccess(node) => self.ast_visit_member_access(node, data),
      NoneLiteral(node) => self.ast_visit_none_literal(node, data),
      NumLiteral(node) => self.ast_visit_num_literal(node, data),
      Reassignment(node) => self.ast_visit_reassignment(node, data),
      RepeatLiteral(node) => self.ast_visit_repeat_literal(node, data),
      ReturnStmt(node) => self.ast_visit_return_stmt(node, data),
      SelfLiteral(node) => self.ast_visit_self_literal(node, data),
      SpreadExpr(node) => self.ast_visit_spread_expr(node, data),
      StringInterpol(node) => self.ast_visit_string_interpol(node, data),
      StringLiteral(node) => self.ast_visit_string_literal(node, data),
      SuperLiteral(node) => self.ast_visit_super_literal(node, data),
      TernaryConditional(node) => self.ast_visit_ternary_conditional(node, data),
      ThrowStmt(node) => self.ast_visit_throw_stmt(node, data),
      TrueLiteral(node) => self.ast_visit_true_literal(node, data),
      TryCatchFinally(node) => self.ast_visit_try_catch_finally(node, data),
      TupleLiteral(node) => self.ast_visit_tuple_literal(node, data),
      UnaryExpr(node) => self.ast_visit_unary_expr(node, data),
      VarConstDecl(node) => self.ast_visit_var_const_decl(node, data),
      WhileLoop(node) => self.ast_visit_while_loop(node, data),
      WithStmt(node) => self.ast_visit_with_stmt(node, data),
      YieldStmt(node) => self.ast_visit_yield_stmt(node, data),
    };

    if !data.is_empty() {
      attributes["branch"] = Value::from(data)
    }

    json!({
       "name": name,
       "attributes": attributes,
       "children": children,
    })
  }
}

impl<'a> ASTVisitor<'a> for PLVJsonGenerator<'a> {
  type Res = (String, Value, Vec<Value>);
  type Data = &'a str;

  fn get_ast(&self) -> &'a ASTArena {
    self.ast
  }

  fn ast_visit_module(&mut self, node: &ASTModuleNode, _: Self::Data) -> Self::Res {
    let children = self.ast_list_to_json(&node.children, "top-level");
    ("Module".into(), json!({}), children)
  }

  fn ast_visit_block_stmt(&mut self, node: &BlockNode, _: Self::Data) -> Self::Res {
    ("Block Stmt".into(), json!({}), self.ast_list_to_json(&node.0, ""))
  }

  fn ast_visit_del_stmt(&mut self, node: &ASTNodeIdx, _: Self::Data) -> Self::Res {
    ("Del".into(), json!({}), vec![self.ast_to_json(*node, "value")])
  }

  fn ast_visit_export_decl(&mut self, _: &ASTImportExportNode, _: Self::Data) -> Self::Res {
    // TODO
    ("".into(), json!({}), vec![])
  }

  fn ast_visit_expr_stmt(&mut self, node: &ASTExprStmt, _: Self::Data) -> Self::Res {
    (
      "Expr Stmt".into(),
      json!({}),
      vec![self.ast_to_json(node.expr, "operand")],
    )
  }

  fn ast_visit_if_stmt(&mut self, _: &ASTIfStmtNode, _: Self::Data) -> Self::Res {
    // TODO
    ("".into(), json!({}), vec![])
  }

  fn ast_visit_import_decl(&mut self, _: &ASTImportExportNode, _: Self::Data) -> Self::Res {
    // TODO
    ("".into(), json!({}), vec![])
  }

  fn ast_visit_throw_stmt(&mut self, node: &ASTNodeIdx, _: Self::Data) -> Self::Res {
    ("Throw".into(), json!({}), vec![self.ast_to_json(*node, "value")])
  }

  fn ast_visit_try_catch_finally(&mut self, _: &ASTTryCatchFinallyNode, _: Self::Data) -> Self::Res {
    // TODO
    ("".into(), json!({}), vec![])
  }

  fn ast_visit_var_const_decl(&mut self, _: &ASTVarConsDeclNode, _: Self::Data) -> Self::Res {
    // TODO
    ("".into(), json!({}), vec![])
  }

  fn ast_visit_with_stmt(&mut self, _: &ASTWithStmtNode, _: Self::Data) -> Self::Res {
    // TODO
    ("".into(), json!({}), vec![])
  }

  fn ast_visit_break_stmt(&mut self, node: &ASTBreakStmtNode, _: Self::Data) -> Self::Res {
    let val = if let Some(v) = node.val {
      vec![self.ast_to_json(v, "value")]
    } else {
      vec![]
    };
    ("<BREAK>".into(), json!({}), val)
  }

  fn ast_visit_continue_stmt(&mut self, _: &TokenIdx, _: Self::Data) -> Self::Res {
    ("<CONTINUE>".into(), json!({}), vec![])
  }

  fn ast_visit_for_loop(&mut self, _: &ASTForLoopNode, _: Self::Data) -> Self::Res {
    // TODO
    ("".into(), json!({}), vec![])
  }

  fn ast_visit_loop_expr(&mut self, _: &ASTLoopExprNode, _: Self::Data) -> Self::Res {
    // TODO
    ("".into(), json!({}), vec![])
  }

  fn ast_visit_while_loop(&mut self, _: &ASTWhileLoopNode, _: Self::Data) -> Self::Res {
    // TODO
    ("".into(), json!({}), vec![])
  }

  fn ast_visit_class_decl(&mut self, _: &ASTClassIdx, _: Self::Data) -> Self::Res {
    // TODO
    ("".into(), json!({}), vec![])
  }

  fn ast_visit_func_decl(&mut self, _: &ASTFuncDeclNode, _: Self::Data) -> Self::Res {
    // TODO
    ("".into(), json!({}), vec![])
  }

  fn ast_visit_lambda(&mut self, _: &ASTLambdaNode, _: Self::Data) -> Self::Res {
    // TODO
    ("".into(), json!({}), vec![])
  }

  fn ast_visit_return_stmt(&mut self, node: &ASTReturnStmtNode, _: Self::Data) -> Self::Res {
    ("Return".into(), json!({}), vec![self.ast_to_json(node.val, "value")])
  }

  fn ast_visit_yield_stmt(&mut self, node: &ASTNodeIdx, _: Self::Data) -> Self::Res {
    ("Yield".into(), json!({}), vec![self.ast_to_json(*node, "value")])
  }

  fn ast_visit_binary_expr(&mut self, node: &ASTBinaryExprNode, _: Self::Data) -> Self::Res {
    let branches = vec![
      self.ast_to_json(node.left, "left"),
      self.ast_to_json(node.right, "right"),
    ];
    let opr = format!("{:?}", node.kind);
    ("Binary".into(), json!({ "operator": opr }), branches)
  }

  fn ast_visit_call_expr(&mut self, _: &ASTCallExprNode, _: Self::Data) -> Self::Res {
    // TODO
    ("".into(), json!({}), vec![])
  }

  fn ast_visit_member_access(&mut self, node: &ASTMemberAccessNode, _: Self::Data) -> Self::Res {
    let is_safe = if node.is_safe { "true" } else { "false" };
    let target = self.ast_to_json(node.target, "target");
    // let member = self.ast_to_json(&x.member, "member");
    ("Member".into(), json!({ "is safe": is_safe }), vec![target])
  }

  fn ast_visit_reassignment(&mut self, node: &ASTReassignmentNode, _: Self::Data) -> Self::Res {
    let children = vec![
      self.ast_to_json(node.target, "target"),
      self.ast_to_json(node.value, "value"),
    ];
    let kind = json!({ "kind": format!("{:?}", node.kind) });
    ("Reassignment".into(), kind, children)
  }

  fn ast_visit_spread_expr(&mut self, node: &ASTNodeIdx, _: Self::Data) -> Self::Res {
    ("Spread".into(), json!({}), vec![self.ast_to_json(*node, "target")])
  }

  fn ast_visit_string_interpol(&mut self, nodes: &[ASTNodeIdx], _: Self::Data) -> Self::Res {
    ("Str Interpolation".into(), json!({}), self.ast_list_to_json(nodes, ""))
  }

  fn ast_visit_ternary_conditional(&mut self, node: &ASTTernaryConditionalNode, _: Self::Data) -> Self::Res {
    let cond = self.ast_to_json(node.condition, "condition");
    let b_true = self.ast_to_json(node.branch_true, "true");
    let b_false = self.ast_to_json(node.branch_false, "false");
    ("Ternary".into(), json!({}), vec![cond, b_true, b_false])
  }

  fn ast_visit_unary_expr(&mut self, node: &ASTUnaryExprNode, _: Self::Data) -> Self::Res {
    (
      "Unary".into(),
      json!({}),
      vec![self.ast_to_json(node.operand, "operand")],
    )
  }

  fn ast_visit_array_literal(&mut self, node: &ASTArrayLiteralNode, _: Self::Data) -> Self::Res {
    ("Array".into(), json!({}), self.ast_list_to_json(&node.values, ""))
  }

  fn ast_visit_array_slice(&mut self, node: &ASTArraySliceNode, _: Self::Data) -> Self::Res {
    let mut children: Vec<Value> = vec![];

    if let Some(upper) = node.upper {
      children.push(self.ast_to_json(upper, "upper"))
    }

    if let Some(lower) = node.lower {
      children.push(self.ast_to_json(lower, "lower"))
    }

    ("Slice".into(), json!({}), children)
  }

  fn ast_visit_compact_arr_or_tpl(&mut self, _: &ASTCompactArrOrTplNode, _: Self::Data) -> Self::Res {
    // TODO
    ("".into(), json!({}), vec![])
  }

  fn ast_visit_compact_dict(&mut self, _: &ASTCompactDictNode, _: Self::Data) -> Self::Res {
    // TODO
    ("".into(), json!({}), vec![])
  }

  fn ast_visit_dict_key_val_pair(&mut self, _: &(ASTNodeIdx, ASTNodeIdx), _: Self::Data) -> Self::Res {
    // TODO
    ("".into(), json!({}), vec![])
  }

  fn ast_visit_dict_literal(&mut self, _s: &[ASTNodeIdx], _: Self::Data) -> Self::Res {
    // TODO
    ("".into(), json!({}), vec![])
  }

  fn ast_visit_evaluated_dict_key(&mut self, _: &ASTNodeIdx, _: Self::Data) -> Self::Res {
    // TODO
    ("".into(), json!({}), vec![])
  }

  fn ast_visit_indexing(&mut self, node: &ASTIndexingNode, _: Self::Data) -> Self::Res {
    let mut children = vec![self.ast_to_json(node.target, "target")];
    children.append(&mut self.ast_list_to_json(&node.indexers, "indexer"));
    ("Indexing".into(), json!({}), children)
  }

  fn ast_visit_repeat_literal(&mut self, node: &ASTRepeatLiteralNode, _: Self::Data) -> Self::Res {
    let value = self.ast_to_json(node.value, "value");
    let count = self.ast_to_json(node.count, "count");
    let kind = format!("{:?}", node.kind);
    ("Repeat".into(), json!({ "kind": kind }), vec![value, count])
  }

  fn ast_visit_tuple_literal(&mut self, node: &ASTTupleLiteralNode, _: Self::Data) -> Self::Res {
    ("Tuple".into(), json!({}), self.ast_list_to_json(&node.values, ""))
  }

  fn ast_visit_id_literal(&mut self, node: &TokenIdx, _: Self::Data) -> Self::Res {
    let lexeme = self.tokens_list.lexeme(*node);
    (lexeme, json!({ "kind": "identifier" }), vec![])
  }

  fn ast_visit_self_literal(&mut self, _: &TokenIdx, _: Self::Data) -> Self::Res {
    // TODO
    ("".into(), json!({}), vec![])
  }

  fn ast_visit_super_literal(&mut self, _: &TokenIdx, _: Self::Data) -> Self::Res {
    // TODO
    ("".into(), json!({}), vec![])
  }

  fn ast_visit_false_literal(&mut self, _: &TokenIdx, _: Self::Data) -> Self::Res {
    // TODO
    ("".into(), json!({}), vec![])
  }

  fn ast_visit_none_literal(&mut self, _: &TokenIdx, _: Self::Data) -> Self::Res {
    // TODO
    ("".into(), json!({}), vec![])
  }

  fn ast_visit_num_literal(&mut self, node: &TokenIdx, _: Self::Data) -> Self::Res {
    let lexeme = self.tokens_list.lexeme(*node);

    let val = match self.tokens_list[*node].kind {
      TokenKind::INT_LIT => parse_int_lexeme(lexeme).expect("Count not convert to int").to_string(),
      TokenKind::FLOAT_LIT => parse_float_lexeme(lexeme).expect("Count not convert to float").to_string(),
      TokenKind::HEX_LIT => parse_int_from_lexeme_base(lexeme, 16).expect("Count not convert hex to int").to_string(),
      TokenKind::OCTAL_LIT => parse_int_from_lexeme_base(lexeme, 8).expect("Count not convert oct to int").to_string(),
      TokenKind::BINARY_LIT => parse_int_from_lexeme_base(lexeme, 2).expect("Count not convert bin to int").to_string(),
      TokenKind::SCIENTIFIC_LIT => parse_scientific_literal_lexeme(lexeme)
        .expect("Could not parse scientific literal to int")
        .to_string(),
      _ => unreachable!("Should have parsed a numeric literal."),
    };

    let lexeme = self.tokens_list.lexeme(*node);
    (val, json!({ "kind": "Numeric Literal", "lexeme": lexeme }), vec![])
  }

  fn ast_visit_string_literal(&mut self, node: &TokenIdx, _: Self::Data) -> Self::Res {
    let lexeme = self.tokens_list.lexeme(*node);
    ("Str".into(), json!({ "value": lexeme }), vec![])
  }

  fn ast_visit_true_literal(&mut self, _: &TokenIdx, _: Self::Data) -> Self::Res {
    // TODO
    ("".into(), json!({}), vec![])
  }
}
