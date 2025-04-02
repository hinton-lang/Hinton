use crate::tokens::TokenIdx;

#[derive(Eq, PartialEq, Default, Debug)]
pub struct Chunk {
  /// The instructions to execute in this chuck.
  pub instructions: Vec<u8>,
  /// The token associated with each instruction.
  // NOTE: Although this is somewhat wasteful, it'll provide us with
  // enough information to report useful error messages to the user.
  pub tokens: Vec<TokenIdx>,
}

impl Chunk {
  /// Gets the size of the instructions list.
  pub fn len(&self) -> usize {
    self.instructions.len()
  }

  /// Checks whether or not the instructions list is empty.
  pub fn is_empty(&self) -> bool {
    self.instructions.is_empty()
  }

  /// Adds a raw byte into the instructions list.
  pub fn push_byte(&mut self, val: u8) {
    self.instructions.push(val);
  }

  /// Retrieves the current and next bytes at the given instructions list index, then converts
  /// and returns the bytes sequence into an u16.
  pub fn get_short(&self, idx: usize) -> u16 {
    let b1 = self.instructions[idx];
    let b2 = self.instructions[idx + 1];
    u16::from_be_bytes([b1, b2])
  }

  /// Splits a 16-bit integer into two bytes, and pushes the byte sequence into the instructions list.
  pub fn push_short(&mut self, val: u16) {
    let short = val.to_be_bytes();
    self.instructions.push(short[0]);
    self.instructions.push(short[1]);
  }

  /// Modifies the byte value at the specified chunk index.
  pub fn patch(&mut self, idx: usize, new_byte: u8) {
    self.instructions[idx] = new_byte;
  }

  /// Gets the token index associated with the byte at the given instruction index.
  pub fn get_tok(&self, idx: usize) -> TokenIdx {
    self.tokens[idx]
  }

  /// Pushes the token index associated with the last byte in the instruction list.
  pub fn push_tok(&mut self, tok: TokenIdx) {
    self.tokens.push(tok);
  }
}
