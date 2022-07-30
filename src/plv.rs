use std::time::{SystemTime, UNIX_EPOCH};

use serde_json::{json, Value};

use crate::lexer::tokens::Token;
use crate::lexer::Lexer;
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

pub fn export(lexer: Option<&Lexer>, ast: Option<&ASTArena>, _module: Option<&FuncObject>, timers: PLVTimers) {
  // Lex the source file
  let lex = match lexer {
    Some(l) => {
      fn map_tok_to_json(t: &Token) -> Value {
        json!({
           "name": format!("{:0>20?}", t.kind),
           "line_num": t.line_num,
           "column": t.column_start,
           "lexeme": t.lexeme
        })
      }

      json!({
         "start": timers.0,
         "end": timers.1,
         "tokens": l.tokens.iter().map(map_tok_to_json).collect::<Vec<Value>>()
      })
    }
    None => json!({}),
  };

  let pars = match ast {
    Some(p) => json!({
       "start": timers.2,
       "end": timers.3,
       "ast": ast_to_json(&lexer.unwrap().tokens, p, &0, "")
    }),
    None => json!({}),
  };

  // let comp = if let Some(m) = module {
  //    json!({
  //       "start": timers.4,
  //       "end": timers.5,
  //       "raw_bytes": m.chunk.get_instructions_list().clone()
  //    })
  // } else {
  //    json!({})
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
  let (name, mut attributes, children) = match &arena.get(*idx).kind {
    Module(x) => ("Module", json!({}), ast_list_to_json(tokens, arena, x, "")),
    Reassignment(_) => ("Reassignment", json!({}), vec![]),
    Literal(x) => (
      tokens[x.token_idx].lexeme.as_str(),
      json!({ "kind": "literal" }),
      vec![],
    ),
    StringLiteral(_) => ("String", json!({}), vec![]),
    SelfLiteral(_) => ("Self", json!({}), vec![]),
    SuperLiteral(_) => ("Super", json!({}), vec![]),
    Identifier(x) => (tokens[*x].lexeme.as_str(), json!({ "kind": "identifier" }), vec![]),
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

      if let Some(upper) = x.upper {
        children.push(ast_to_json(tokens, arena, &upper, "upper"))
      }

      if let Some(lower) = x.lower {
        children.push(ast_to_json(tokens, arena, &lower, "lower"))
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
    ExprStmt(_) => ("Expr Stmt", json!({}), vec![]),
    BlockStmt(_) => ("Block Stmt", json!({}), vec![]),
    LoopExprStmt(_) => ("Loop Stmt", json!({}), vec![]),
    BreakStmt(_) => ("Break Stmt", json!({}), vec![]),
    ContinueStmt => ("Continue Stmt", json!({}), vec![]),
    ReturnStmt(_) => ("Return Stmt", json!({}), vec![]),
    YieldStmt(_) => ("Yield Stmt", json!({}), vec![]),
    ThrowStmt(_) => ("Throe Stmt", json!({}), vec![]),
    DelStmt(_) => ("Del Stmt", json!({}), vec![]),
    WhileLoop(_) => ("While Loop Stmt", json!({}), vec![]),
    ForLoop(_) => ("For Loop Stmt", json!({}), vec![]),
    ForLoopHead(_) => ("For Loop Head", json!({}), vec![]),
    CompactArrOrTpl(_) => ("Compact Arr or Tpl", json!({}), vec![]),
    CompactDict(_) => ("Compact Dict", json!({}), vec![]),
    CompactForLoop(_) => ("Compact For Loop", json!({}), vec![]),
    IfStmt(_) => ("IfStmt", json!({}), vec![]),
    VarConstDecl(_) => ("VarConstDecl", json!({}), vec![]),
    DestructingPattern(_) => ("DestructingPattern", json!({}), vec![]),
    DestructingWildCard(_) => ("DestructingWildCard", json!({}), vec![]),
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
