use std::time::{SystemTime, UNIX_EPOCH};

use serde_json::{json, Value};

use core::ast::ASTNodeKind::*;
use core::ast::*;
use core::tokens::{Token, TokenIdx, TokenKind, TokenList};
use core::utils::*;

/// The start and end times for the Lexer, Parser, and compiler respectively.
pub type PLVTimers = (u64, u64, u64, u64, u64, u64);

/// Get the current unix epoch time in milliseconds.
pub fn get_time_millis() -> u64 {
  let start = SystemTime::now();
  let time_since_epoch = start.duration_since(UNIX_EPOCH).expect("Time went backwards");
  time_since_epoch.as_secs() * 1000 + time_since_epoch.subsec_nanos() as u64 / 1_000_000
}

/// Maps a single token to its JSON representation.
///
/// # Arguments
///
/// * `tok`: A tuple containing the token's index and value respectively.∏
/// * `tokens_list`: The TokensList where the tokens are stored.
///
/// # Returns:
/// ```Value```
fn map_tok_to_json(tok: (usize, &Token), tokens_list: &TokenList) -> Value {
  json!({
     "name": format!("{:0>20?}", tok.1.kind),
     "line_num": tok.1.line_num,
     "column": tokens_list.location(&tok.0.into()).col_start,
     "lexeme": tokens_list.lexeme(&tok.0.into())
  })
}

/// Exports the Lexer, Parser, and compiler as JSON file that can be
/// uploaded to the Hinton Program Lifecycle Visualizer for inspection.
///
/// # Arguments
///
/// * `tokens_list`: The TokenList were the lexed tokens are stored.
/// * `arena`: The ASTArena where the AST nodes are stored.
/// * `_bytecode`: The compiled bytecode.
/// * `timers`: The times it took the Lexer, Parser, and compiler to execute.
///
/// # Returns:
/// ```()```
pub fn export(tokens_list: &TokenList, arena: &ASTArena, _bytecode: &[u8], timers: PLVTimers) {
  let plv_start = get_time_millis();

  let json_tokens = tokens_list
    .tokens
    .iter()
    .enumerate()
    .map(|t| map_tok_to_json(t, tokens_list))
    .collect::<Vec<Value>>();

  let lex = json!({
     "start": timers.0,
     "end": timers.1,
     "tokens": json_tokens
  });

  let plv_generator = PLVJsonGenerator { tokens_list, arena };
  let pars = json!({
     "start": timers.2,
     "end": timers.3,
     "ast": plv_generator.ast_to_json(&0.into(), "root")
  });

  // Compose the JSON report
  let report = json!({
     "date": get_time_millis(),
     "run_type": if cfg!(debug_assertions) { "DEV" } else { "RELEASE" },
     "lexer": lex,
     "parser": pars
  });

  // Save the file
  let str_json = serde_json::to_string_pretty(&report).unwrap();
  std::fs::write("./local_dev/plv_data.json", str_json).unwrap();

  let plv_end = get_time_millis();
  println!(
    "PLV >> Finished execution in {:.3}s",
    ((plv_end - plv_start) as f32) / 1000.0
  );
}

/// The JSON representation of an AST node for the PLV.
type JSONNodeData = (String, Value, Vec<Value>);

/// The JSON generator for the Hinton Program Lifecycle Visualizer
#[derive(Clone, Copy)]
pub struct PLVJsonGenerator<'a> {
  pub tokens_list: &'a TokenList<'a>,
  pub arena: &'a ASTArena,
}

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
  fn ast_list_to_json(&self, nodes: &[ASTNodeIdx], branch_name: &str) -> Vec<Value> {
    nodes.iter().map(|x| self.ast_to_json(x, branch_name)).collect()
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
  fn ast_to_json(&self, idx: &ASTNodeIdx, branch_name: &str) -> Value {
    let (name, mut attributes, children) = match &self.arena.get(idx) {
      ArrayLiteral(x) => self.array_literal_to_json(x),
      ArraySlice(x) => self.array_slice_to_json(x),
      BinaryExpr(x) => self.binary_to_json(x),
      BlockStmt(x) => self.block_stmt_to_json(x),
      BreakStmt(x) => self.break_stmt_to_json(x),
      CallExpr(x) => self.call_expr_to_json(&x),
      ClassDecl(x) => self.class_decl_to_json(x),
      CompactArrOrTpl(x) => self.compact_arr_or_tpl_to_json(x),
      CompactDict(x) => self.compact_dict_to_json(x),
      ContinueStmt => self.continue_stmt_to_json(),
      DelStmt(x) => self.del_stmt_to_json(x),
      DestructingPattern(x) => self.destructing_pattern_to_json(x),
      DestructingWildCard(x) => self.destructing_wild_card_to_json(x),
      DictKeyValPair(x) => self.dict_key_val_par_to_json(x),
      DictLiteral(_) => self.dict_literal_to_json(),
      EvaluatedDictKey(x) => self.evaluated_dict_key_to_json(x),
      ExportDecl(x) => self.export_decl_to_json(x),
      ExprStmt(x) => self.expr_stmt_to_json(x),
      FalseLiteral(_) => ("false".into(), json!({ "kind": "literal" }), vec![]),
      ForLoop(x) => self.for_loop_to_json(x),
      FuncDecl(x) => self.func_decl_to_json(x),
      Identifier(x) => self.identifier_to_json(x),
      IfStmt(x) => self.if_stmt_to_json(x),
      ImportDecl(x) => self.import_decl_to_json(x),
      Indexing(x) => self.indexing_to_json(x),
      Lambda(x) => self.lambda_to_json(x),
      LoopExpr(x) => self.loop_expr_to_json(x),
      MemberAccess(x) => self.member_access_to_json(x),
      Module(x) => self.module_node_to_json(x),
      NoneLiteral(_) => ("none".into(), json!({ "kind": "literal" }), vec![]),
      NumLiteral(x) => self.numeric_literal_to_json(x),
      Reassignment(x) => self.reassignment_node_to_json(x),
      RepeatLiteral(x) => self.repeat_literal_to_json(x),
      ReturnStmt(x) => self.return_stmt_to_json(x),
      SelfLiteral(_) => ("self".into(), json!({ "kind": "literal" }), vec![]),
      SpreadExpr(x) => self.spread_expr_to_json(x),
      StringInterpol(x) => self.string_interpolation_to_json(x),
      StringLiteral(x) => self.string_literal_to_json(x),
      SuperLiteral(_) => ("super".into(), json!({ "kind": "literal" }), vec![]),
      TernaryConditional(x) => self.ternary_to_json(x),
      ThrowStmt(x) => self.throw_stmt_to_json(x),
      TrueLiteral(_) => ("true".into(), json!({ "kind": "literal" }), vec![]),
      TryCatchFinally(x) => self.try_catch_finally_stmt_to_json(x),
      TupleLiteral(x) => self.tuple_literal_to_json(x),
      UnaryExpr(x) => self.unary_to_json(x),
      VarConstDecl(x) => self.var_const_decl_to_json(x),
      WhileLoop(x) => self.while_loop_to_json(x),
      WithStmt(x) => self.with_stmt_to_json(x),
      YieldStmt(x) => self.yield_stmt_to_json(x),
    };

    if !branch_name.is_empty() {
      attributes["branch"] = Value::from(branch_name)
    }

    json!({
       "name": name,
       "attributes": attributes,
       "children": children,
    })
  }

  /// Converts an AST module node into its JSON representation.
  fn module_node_to_json(self, x: &ASTModuleNode) -> JSONNodeData {
    let children = self.ast_list_to_json(&x.children, "top-level");
    ("Module".into(), json!({}), children)
  }

  /// Converts an AST reassignment node into its JSON representation.
  fn reassignment_node_to_json(self, x: &ASTReassignmentNode) -> JSONNodeData {
    let children = vec![
      self.ast_to_json(&x.target, "target"),
      self.ast_to_json(&x.value, "value"),
    ];

    (
      "Reassignment".into(),
      json!({ "kind": format!("{:?}", x.kind) }),
      children,
    )
  }

  /// Converts an AST numeric literal node into its JSON representation.
  fn numeric_literal_to_json(self, x: &TokenIdx) -> JSONNodeData {
    let lexeme = self.tokens_list.lexeme(x);

    let val = match self.tokens_list[x.0].kind {
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

    let lexeme = self.tokens_list.lexeme(x);
    (val, json!({ "kind": "Numeric Literal", "lexeme": lexeme }), vec![])
  }

  /// Converts an AST string literal node into its JSON representation.
  fn string_literal_to_json(self, x: &TokenIdx) -> JSONNodeData {
    let lexeme = self.tokens_list.lexeme(x);
    ("Str".into(), json!({ "value": lexeme }), vec![])
  }

  /// Converts an AST identifier node into its JSON representation.
  fn identifier_to_json(self, x: &TokenIdx) -> JSONNodeData {
    let lexeme = self.tokens_list.lexeme(x);
    (lexeme, json!({ "kind": "identifier" }), vec![])
  }

  /// Converts an AST ternary conditional node into its JSON representation.
  fn ternary_to_json(self, x: &ASTTernaryConditionalNode) -> JSONNodeData {
    let cond = self.ast_to_json(&x.condition, "condition");
    let b_true = self.ast_to_json(&x.branch_true, "true");
    let b_false = self.ast_to_json(&x.branch_false, "false");

    ("Ternary".into(), json!({}), vec![cond, b_true, b_false])
  }

  /// Converts an AST binary expression node into its JSON representation.
  fn binary_to_json(self, x: &ASTBinaryExprNode) -> JSONNodeData {
    let branches = vec![self.ast_to_json(&x.left, "left"), self.ast_to_json(&x.right, "right")];
    let opr = format!("{:?}", x.kind);
    ("Binary".into(), json!({ "operator": opr }), branches)
  }

  /// Converts an AST unary expression node into its JSON representation.
  fn unary_to_json(self, x: &ASTUnaryExprNode) -> JSONNodeData {
    ("Unary".into(), json!({}), vec![self.ast_to_json(&x.operand, "operand")])
  }

  /// Converts an AST indexing expression node into its JSON representation.
  fn indexing_to_json(self, x: &ASTIndexingNode) -> JSONNodeData {
    let mut children = vec![self.ast_to_json(&x.target, "target")];
    children.append(&mut self.ast_list_to_json(&x.indexers, "indexer"));

    ("Indexing".into(), json!({}), children)
  }

  /// Converts an AST array slice expression node into its JSON representation.
  fn array_slice_to_json(self, x: &ASTArraySliceNode) -> JSONNodeData {
    let mut children: Vec<Value> = vec![];

    if let Some(upper) = &x.upper {
      children.push(self.ast_to_json(upper, "upper"))
    }

    if let Some(lower) = &x.lower {
      children.push(self.ast_to_json(lower, "lower"))
    }

    ("Slice".into(), json!({}), children)
  }

  /// Converts an AST lambda expression node into its JSON representation.
  fn lambda_to_json(&self, _x: &ASTLambdaNode) -> JSONNodeData {
    // TODO: PLV Lambda Lit
    ("Lambda".into(), json!({}), vec![])
  }

  /// Converts an AST destructing wild card node into its JSON representation.
  fn destructing_wild_card_to_json(&self, _x: &Option<ASTNodeIdx>) -> JSONNodeData {
    // TODO: PLV Destruct Wild Card
    ("DestructingWildCard".into(), json!({}), vec![])
  }

  /// Converts an AST destructing pattern node into its JSON representation.
  fn destructing_pattern_to_json(&self, _x: &ASTDestructingPatternNode) -> JSONNodeData {
    // TODO: PLV Destruct Pattern
    ("DestructingPattern".into(), json!({}), vec![])
  }

  /// Converts an AST compact dict literal node into its JSON representation.
  fn compact_dict_to_json(&self, _x: &ASTCompactDictNode) -> JSONNodeData {
    // TODO: PLV Compact Dict Lit
    ("Compact Dict".into(), json!({}), vec![])
  }

  /// Converts an AST compact array or tuple literal node into its JSON representation.
  fn compact_arr_or_tpl_to_json(&self, _x: &ASTCompactArrOrTplNode) -> JSONNodeData {
    // TODO: PLV Compact Array/Tuple Lit
    ("Compact Arr or Tpl".into(), json!({}), vec![])
  }

  /// Converts an AST for loop node into its JSON representation.
  fn for_loop_to_json(&self, _x: &ASTForLoopNode) -> JSONNodeData {
    // TODO: PLV For Loop
    ("For Loop Stmt".into(), json!({}), vec![])
  }

  /// Converts an AST while loop node into its JSON representation.
  fn while_loop_to_json(&self, _x: &ASTWhileLoopNode) -> JSONNodeData {
    // TODO: PLV While Loop
    ("While Loop Stmt".into(), json!({}), vec![])
  }

  /// Converts an AST loop expression node into its JSON representation.
  fn loop_expr_to_json(&self, _x: &ASTNodeIdx) -> JSONNodeData {
    // TODO: PLV Loop Expr
    ("Loop".into(), json!({}), vec![])
  }

  /// Converts an AST var or const declaration node into its JSON representation.
  fn var_const_decl_to_json(&self, _x: &ASTVarConsDeclNode) -> JSONNodeData {
    // TODO: PLV Var/Const Decl
    ("VarConstDecl".into(), json!({}), vec![])
  }

  /// Converts an AST func declaration node into its JSON representation.
  fn func_decl_to_json(&self, _x: &ASTFuncDeclNode) -> JSONNodeData {
    // TODO: PLV Func Decl
    ("FuncDecl".into(), json!({}), vec![])
  }

  /// Converts an AST export declaration node into its JSON representation.
  fn export_decl_to_json(&self, _x: &ASTImportExportNode) -> JSONNodeData {
    // TODO: PLV Export Decl
    ("ExportDecl".into(), json!({}), vec![])
  }

  /// Converts an AST import declaration node into its JSON representation.
  fn import_decl_to_json(&self, _x: &ASTImportExportNode) -> JSONNodeData {
    // TODO: PLV Import Decl
    ("ImportDecl".into(), json!({}), vec![])
  }

  /// Converts an AST try-catch-finally node into its JSON representation.
  fn try_catch_finally_stmt_to_json(&self, _x: &ASTTryCatchFinallyNode) -> JSONNodeData {
    // TODO: PLV Try-Catch-Finally Stmt
    ("TryCatchFinally".into(), json!({}), vec![])
  }

  /// Converts an AST with statement node into its JSON representation.
  fn with_stmt_to_json(&self, _x: &ASTWithStmtNode) -> JSONNodeData {
    // TODO: PLV With Stmt
    ("WithStmt".into(), json!({}), vec![])
  }

  /// Converts an AST if statement node into its JSON representation.
  fn if_stmt_to_json(&self, _x: &ASTIfStmtNode) -> JSONNodeData {
    // TODO: PLV If Stmt
    ("IfStmt".into(), json!({}), vec![])
  }

  /// Converts an AST del statement node into its JSON representation.
  fn del_stmt_to_json(&self, x: &ASTNodeIdx) -> JSONNodeData {
    ("Del".into(), json!({}), vec![self.ast_to_json(x, "value")])
  }

  /// Converts an AST throw statement node into its JSON representation.
  fn throw_stmt_to_json(&self, x: &ASTNodeIdx) -> JSONNodeData {
    ("Throw".into(), json!({}), vec![self.ast_to_json(x, "value")])
  }

  /// Converts an AST yield statement node into its JSON representation.
  fn yield_stmt_to_json(&self, x: &ASTNodeIdx) -> JSONNodeData {
    ("Yield".into(), json!({}), vec![self.ast_to_json(x, "value")])
  }

  /// Converts an AST return statement node into its JSON representation.
  fn return_stmt_to_json(&self, x: &ASTNodeIdx) -> JSONNodeData {
    ("Return".into(), json!({}), vec![self.ast_to_json(x, "value")])
  }

  /// Converts an AST continue statement node into its JSON representation.
  fn continue_stmt_to_json(&self) -> JSONNodeData {
    ("<CONTINUE>".into(), json!({}), vec![])
  }

  /// Converts an AST with statement node into its JSON representation.
  fn break_stmt_to_json(&self, x: &Option<ASTNodeIdx>) -> JSONNodeData {
    let val = if let Some(v) = x { vec![self.ast_to_json(v, "value")] } else { vec![] };
    ("<BREAK>".into(), json!({}), val)
  }

  /// Converts an AST block statement node into its JSON representation.
  fn block_stmt_to_json(&self, x: &[ASTNodeIdx]) -> JSONNodeData {
    ("Block Stmt".into(), json!({}), self.ast_list_to_json(x, ""))
  }

  /// Converts an AST call expression node into its JSON representation.
  fn call_expr_to_json(&self, x: &&ASTCallExprNode) -> JSONNodeData {
    let mut children = vec![self.ast_to_json(&x.target, "target")];
    children.append(&mut self.ast_list_to_json(&x.val_args, "value arg"));
    children.append(&mut self.ast_list_to_json(&x.rest_args, "rest arg"));
    // children.append(&mut ast_list_to_json(tokens, arena, &x.named_args, "named arg"));

    ("Call".into(), json!({}), children)
  }

  /// Converts an AST evaluated dict key node into its JSON representation.
  fn evaluated_dict_key_to_json(&self, _x: &ASTNodeIdx) -> JSONNodeData {
    // TODO: PLV Dict Literal
    ("EvaluatedDictKey".into(), json!({}), vec![])
  }

  /// Converts an AST dict key-value pair node into its JSON representation.
  fn dict_key_val_par_to_json(&self, _x: &(ASTNodeIdx, ASTNodeIdx)) -> JSONNodeData {
    // TODO: PLV Dict Key-Value Pair
    ("KeyVal".into(), json!({}), vec![])
  }

  /// Converts an AST dict literal node into its JSON representation.
  fn dict_literal_to_json(&self) -> JSONNodeData {
    // TODO: PLV Dict Literal
    ("Dict".into(), json!({}), vec![])
  }

  /// Converts an AST spread expression node into its JSON representation.
  fn spread_expr_to_json(&self, x: &ASTNodeIdx) -> JSONNodeData {
    ("Spread".into(), json!({}), vec![self.ast_to_json(x, "target")])
  }

  /// Converts an AST repeat literal node into its JSON representation.
  fn repeat_literal_to_json(&self, x: &ASTRepeatLiteralNode) -> JSONNodeData {
    let value = self.ast_to_json(&x.value, "value");
    let count = self.ast_to_json(&x.count, "count");
    let kind = format!("{:?}", x.kind);
    ("Repeat".into(), json!({ "kind": kind }), vec![value, count])
  }

  /// Converts an AST tuple literal node into its JSON representation.
  fn tuple_literal_to_json(&self, x: &[ASTNodeIdx]) -> JSONNodeData {
    ("Tuple".into(), json!({}), self.ast_list_to_json(x, ""))
  }

  /// Converts an AST array literal node into its JSON representation.
  fn array_literal_to_json(&self, x: &[ASTNodeIdx]) -> JSONNodeData {
    ("Array".into(), json!({}), self.ast_list_to_json(x, ""))
  }

  /// Converts an AST member access node into its JSON representation.
  fn member_access_to_json(&self, x: &ASTMemberAccessNode) -> JSONNodeData {
    let is_safe = if x.is_safe { "true" } else { "false" };
    let target = self.ast_to_json(&x.target, "target");
    let member = self.ast_to_json(&x.member, "member");
    ("Member".into(), json!({ "is safe": is_safe }), vec![target, member])
  }

  /// Converts an AST expression statement node into its JSON representation.
  fn expr_stmt_to_json(&self, x: &ASTNodeIdx) -> JSONNodeData {
    ("Expr Stmt".into(), json!({}), vec![self.ast_to_json(x, "operand")])
  }

  fn class_decl_to_json(&self, _x: &ASTClassIdx) -> JSONNodeData {
    // TODO: PLV Class Decl
    ("Class Decl".into(), json!({}), vec![])
  }

  fn string_interpolation_to_json(&self, x: &[ASTNodeIdx]) -> JSONNodeData {
    ("Str Interpolation".into(), json!({}), self.ast_list_to_json(x, ""))
  }
}
