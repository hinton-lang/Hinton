use serde_json::{json, Value};

use analyzers::symbols::{SymbolTable, SymbolTableArena};
use compiler::Compiler;
use core::ast::*;
use core::errors::ErrorReport;
use core::tokens::{Token, TokenList};
use core::utils::get_time_millis;
use objects::gc::{GarbageCollector, GcId};
use objects::Object;
use parser::Parser;

mod disassembler;
mod visitor;

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
     "line_num": tok.1.loc.line_num,
     "column": tok.1.loc.col_start(),
     "lexeme": tokens_list.lexeme(tok.0)
  })
}

/// Exports the Lexer, Parser, and compiler as JSON file that can be
/// uploaded to the Hinton Program Lifecycle Visualizer for inspection.
///
/// # Arguments
///
/// * `tokens_list`: The TokenList were the lexed tokens are stored.
/// * `lexer_time`: The amount of time it took for the lexer to execute.
pub fn export(
  tokens_list: &TokenList,
  lexer_time: u64,
) -> Result<(GarbageCollector, Vec<Object>, GcId), Vec<ErrorReport>> {
  println!("=================================================");
  println!("Program Lifecycle Visualizer");
  println!("-----------");
  println!("Lexer Finished:          {}ms", lexer_time);

  let ps = get_time_millis();
  let ast = Parser::parse(tokens_list)?;
  let pe = get_time_millis();

  println!("Parser Finished:         {}ms", pe - ps);

  let ans = get_time_millis();
  let symbol_tables = SymbolTableArena::tables_from(tokens_list, &ast)?;
  let ane = get_time_millis();

  println!("Symbol Tables Finished:  {}ms", ane - ans);

  let cs = get_time_millis();
  let compiler = Compiler::new(tokens_list, &ast, &symbol_tables);
  let (gc, constants, main_fn) = Compiler::compile_from(compiler)?;
  let ce = get_time_millis();

  println!("Compiler Finished:       {}ms", ce - cs);

  let plv_start = get_time_millis();

  let json_tokens = tokens_list
    .tokens
    .iter()
    .enumerate()
    .map(|t| map_tok_to_json(t, tokens_list))
    .collect::<Vec<Value>>();

  let mut plv_generator = PLVJsonGenerator {
    tokens_list,
    ast: &ast,
    symbol_tables: &symbol_tables,
    constants: &constants,
  };

  let ast_json = plv_generator.ast_to_json(0, "root");
  let instructions = plv_generator.disassemble_all(&gc);

  // Compose the JSON report
  let report = json!({
    "date": get_time_millis(),
    "run_type": if cfg!(debug_assertions) { "DEV" } else { "RELEASE" },
    "lexer": json!({
      "time": lexer_time,
      "tokens": json_tokens
    }),
    "parser": json!({
      "time": pe - ps,
      "ast": ast_json
    }),
    "symbol_tables": json!({
      "time": ane - ans,
      "tables": ast_json
    }),
    "compiler": json!({
      "time": ce - cs,
      "bytecode": instructions,
      "raw_bytes": []
    })
  });

  // Save the file
  let str_json = serde_json::to_string_pretty(&report).unwrap();
  std::fs::write("./local_dev/plv_data.json", str_json).unwrap();
  let plv_end = get_time_millis();

  println!("-----------");
  println!("PLV Finished in {:.3}s", ((plv_end - plv_start) as f32) / 1000.0);
  println!("=================================================");

  Ok((gc, constants, main_fn))
}

/// The JSON generator for the Hinton Program Lifecycle Visualizer
#[derive(Clone, Copy)]
pub struct PLVJsonGenerator<'a> {
  pub tokens_list: &'a TokenList<'a>,
  pub ast: &'a ASTArena,
  pub symbol_tables: &'a [SymbolTable],
  pub constants: &'a [Object],
}
