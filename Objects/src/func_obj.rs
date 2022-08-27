use crate::gc::GcTrace;
use crate::Object;
use core::chunk::Chunk;
use core::tokens::TokenIdx;
use std::fmt::{Debug, Formatter};

/// A Hinton function object.
pub struct FuncObj {
  pub defaults: Vec<Object>,
  pub min_arity: u8,
  pub max_arity: u8,
  pub chunk: Chunk,
  pub name: TokenIdx,
}

impl PartialEq for FuncObj {
  fn eq(&self, other: &Self) -> bool {
    self.name == other.name
  }
}

impl Debug for FuncObj {
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    write!(f, "<Func at '{:p}'>", self)
  }
}

impl GcTrace for FuncObj {}
