pub mod ast;
pub mod bytecode;
pub mod chunk;
pub mod errors;
pub mod tokens;
pub mod utils;

/// The current version of the interpreter.
pub const VERSION: &str = "0.0.1";
/// The max number of frames in the function stack
pub const FRAMES_MAX: usize = 128;
