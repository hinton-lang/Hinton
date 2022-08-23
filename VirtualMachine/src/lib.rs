#[cfg(not(feature = "PLV"))]
use compiler::Compiler;
use core::chunk::Chunk;
use core::errors::report_errors_list;
use core::tokens::TokenList;
use core::values::Value;
use core::InterpretResult;
use lexer::Lexer;
use std::path::PathBuf;

pub struct VM {
  stack: Vec<Value>,
}

impl VM {
  pub fn interpret(filepath: PathBuf, source: Vec<char>) -> InterpretResult {
    #[cfg(feature = "PLV")]
    let lexer_start = plv::get_time_millis();

    let lexer = Lexer::lex(&source);
    let tokens_list = TokenList::new(&filepath, &source, &lexer);

    #[cfg(feature = "PLV")]
    let lexer_end = plv::get_time_millis();

    #[cfg(not(feature = "PLV"))]
    let const_pool = Compiler::compile(&tokens_list);

    #[cfg(feature = "PLV")]
    let const_pool = plv::export(&tokens_list, lexer_end - lexer_start);

    match const_pool {
      Ok(x) => x,
      Err(err) => {
        report_errors_list(&tokens_list, err);
        return InterpretResult::CompileError;
      }
    };

    InterpretResult::Ok
  }
}
