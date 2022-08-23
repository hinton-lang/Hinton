use crate::errors::RuntimeErrMsg;

pub mod ast;
pub mod bytecode;
pub mod chunk;
pub mod errors;
pub mod objects;
pub mod tokens;
pub mod utils;
pub mod values;

/// The max number of frames in the function stack
pub static FRAMES_MAX: usize = 1000;
pub const VERSION: &str = "0.0.1";

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
