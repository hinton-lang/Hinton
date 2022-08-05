use std::time::{SystemTime, UNIX_EPOCH};

use serde_json::{json, Value};

use crate::lexer::tokens::{Token, TokenList};
use crate::objects::FuncObject;
use crate::parser::ast::ASTNodeKind::*;
use crate::parser::ast::*;

pub fn get_time_millis() -> u64 {
  let start = SystemTime::now();
  let time_since_epoch = start.duration_since(UNIX_EPOCH).expect("Time went backwards");
  time_since_epoch.as_secs() * 1000 + time_since_epoch.subsec_nanos() as u64 / 1_000_000
}

fn ast_list_to_json(tokens: &[Token], arena: &ASTArena, nodes: &[ASTNodeIdx], bname: &str) -> Vec<Value> {
  nodes.iter().map(|x| ast_to_json(tokens, arena, x, bname)).collect()
}

pub type PLVTimers = (u64, u64, u64, u64, u64, u64);

pub fn export(toks: &TokenList, ast: &ASTArena, _module: Option<&FuncObject>, timers: PLVTimers) {
  fn map_tok_to_json(tok: (usize, &Token), toks: &TokenList) -> Value {
    json!({
       "name": format!("{:0>20?}", tok.1.kind),
       "line_num": tok.1.line_num,
       "column": tok.1.span.0,
       "lexeme": toks.lexeme(tok.0)
    })
  }
  // Lex the source file
  let lex = json!({
     "start": timers.0,
     "end": timers.1,
     "tokens": toks.tokens.iter().enumerate().map(|t| map_tok_to_json(t, toks)).collect::<Vec<Value>>()
  });

  let pars = json!({
     "start": timers.2,
     "end": timers.3,
     "ast": ast_to_json(toks.tokens, ast, &0.into(), "")
  });

  // let comp = if let Some(m) = module {
  //   json!({
  //      "start": timers.4,
  //      "end": timers.5,
  //      "raw_bytes": m.chunk.get_instructions_list().clone()
  //   })
  // } else {
  //   json!({})
  // };

  // Compose the JSON report
  let report = json!({
     "date": get_time_millis(),
     "run_type": if cfg!(debug_assertions) { "DEV" } else { "RELEASE" },
     "lexer": lex,
     "parser": pars,
     // "compiler": comp
  });

  // Save the file
  let str_json = serde_json::to_string_pretty(&report).unwrap();
  std::fs::write("./local_dev/plv_data.json", str_json).unwrap();
}

fn ast_to_json(tokens: &[Token], arena: &ASTArena, idx: &ASTNodeIdx, bname: &str) -> Value {
  let (name, mut attributes, children) = match &arena.get(idx).kind {
    Module(x) => ("Module", json!({}), ast_list_to_json(tokens, arena, &x.children, "")),
    Reassignment(_) => ("Reassignment", json!({}), vec![]),
    NumLiteral(_) => ("Num Literal", json!({ "kind": "literal" }), vec![]),
    TrueLiteral(_) => ("true", json!({ "kind": "literal" }), vec![]),
    FalseLiteral(_) => ("false", json!({ "kind": "literal" }), vec![]),
    NoneLiteral(_) => ("none", json!({ "kind": "literal" }), vec![]),
    StringLiteral(_) => ("String", json!({ "value": "::todo::" }), vec![]),
    SelfLiteral(_) => ("Self", json!({}), vec![]),
    SuperLiteral(_) => ("Super", json!({}), vec![]),
    Identifier(_) => ("id", json!({ "kind": "identifier" }), vec![]),
    TernaryConditional(_) => ("Ternary", json!({}), vec![]),
    BinaryExpr(x) => (
      "Binary",
      json!({ "operator": format!("{:?}", x.kind) }),
      vec![
        ast_to_json(tokens, arena, &x.left, "left"),
        ast_to_json(tokens, arena, &x.right, "right"),
      ],
    ),
    UnaryExpr(x) => ("Unary", json!({}), vec![ast_to_json(tokens, arena, &x.operand, "")]),
    Indexing(x) => {
      let mut children = vec![ast_to_json(tokens, arena, &x.target, "target")];
      children.append(&mut ast_list_to_json(tokens, arena, &x.indexers, "indexer"));
      ("Indexing", json!({}), children)
    }
    ArraySlice(x) => {
      let mut children: Vec<Value> = vec![];

      if let Some(upper) = &x.upper {
        children.push(ast_to_json(tokens, arena, upper, "upper"))
      }

      if let Some(lower) = &x.lower {
        children.push(ast_to_json(tokens, arena, lower, "lower"))
      }

      ("Slice", json!({}), children)
    }
    MemberAccess(_) => ("Member", json!({}), vec![]),
    ArrayLiteral(_) => ("Array", json!({}), vec![]),
    TupleLiteral(_) => ("Tuple", json!({}), vec![]),
    RepeatLiteral(_) => ("Repeat", json!({}), vec![]),
    SpreadExpr(x) => ("Spread", json!({}), vec![ast_to_json(tokens, arena, x, "")]),
    DictLiteral(_) => ("Dict", json!({}), vec![]),
    DictKeyValPair(_) => ("KeyVal", json!({}), vec![]),
    EvaluatedDictKey(_) => ("EvaluatedDictKey", json!({}), vec![]),
    CallExpr(x) => {
      let mut children = vec![ast_to_json(tokens, arena, &x.target, "target")];
      children.append(&mut ast_list_to_json(tokens, arena, &x.val_args, "value arg"));
      children.append(&mut ast_list_to_json(tokens, arena, &x.rest_args, "rest arg"));
      // children.append(&mut ast_list_to_json(tokens, arena, &x.named_args, "named arg"));
      //
      ("Call", json!({}), children)
    }
    ExprStmt(x) => ("Expr Stmt", json!({}), vec![ast_to_json(tokens, arena, x, "")]),
    BlockStmt(_) => ("Block Stmt", json!({}), vec![]),
    LoopExpr(_) => ("Loop Stmt", json!({}), vec![]),
    BreakStmt(_) => ("Break Stmt", json!({}), vec![]),
    ContinueStmt => ("Continue Stmt", json!({}), vec![]),
    ReturnStmt(_) => ("Return Stmt", json!({}), vec![]),
    YieldStmt(_) => ("Yield Stmt", json!({}), vec![]),
    ThrowStmt(_) => ("Throe Stmt", json!({}), vec![]),
    DelStmt(_) => ("Del Stmt", json!({}), vec![]),
    WhileLoop(_) => ("While Loop Stmt", json!({}), vec![]),
    ForLoop(_) => ("For Loop Stmt", json!({}), vec![]),
    CompactArrOrTpl(_) => ("Compact Arr or Tpl", json!({}), vec![]),
    CompactDict(_) => ("Compact Dict", json!({}), vec![]),
    IfStmt(_) => ("IfStmt", json!({}), vec![]),
    VarConstDecl(_) => ("VarConstDecl", json!({}), vec![]),
    DestructingPattern(_) => ("DestructingPattern", json!({}), vec![]),
    DestructingWildCard(_) => ("DestructingWildCard", json!({}), vec![]),
    WithStmt(_) => ("WithStmt", json!({}), vec![]),
    FuncDecl(_) => ("FuncDecl", json!({}), vec![]),
    Lambda(_) => ("Lambda", json!({}), vec![]),
    TryCatchFinally(_) => ("TryCatchFinally", json!({}), vec![]),
    ImportDecl(_) => ("ImportDecl", json!({}), vec![]),
    ExportDecl(_) => ("ExportDecl", json!({}), vec![]),
  };

  if !bname.is_empty() {
    attributes["branch"] = Value::from(bname)
  }

  json!({
     "name": name,
     "attributes": attributes,
     "children": children,
  })
}
