use crate::errors::RuntimeErrMsg;

pub mod ast;
pub mod errors;
pub mod tokens;
pub mod utils;

/// The types of results the interpreter can return.
pub enum InterpretResult {
  CompileError,
  Ok,
  ParseError,
  RuntimeError,
}

/// Represents the internal state of the interpreter after some computation.
pub enum RuntimeResult {
  Error(RuntimeErrMsg),
  EndOK,
  Continue,
}
