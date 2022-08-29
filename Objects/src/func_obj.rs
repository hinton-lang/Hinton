use std::fmt::{Debug, Formatter};

use core::chunk::Chunk;
use core::tokens::TokenList;

use crate::gc::GcTrace;
use crate::{GarbageCollector, GcId, Object};

/// A Hinton function object.
pub struct FuncObj {
  pub defaults: Vec<Object>,
  pub min_arity: u16,
  pub max_arity: Option<u16>,
  pub chunk: Chunk,
  pub name: GcId,
}

impl PartialEq for FuncObj {
  fn eq(&self, other: &Self) -> bool {
    self.name == other.name
  }
}

impl Debug for FuncObj {
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    write!(f, "<Func at {:p}>", self)
  }
}

impl GcTrace for FuncObj {}

impl FuncObj {
  pub fn display_plain(&self, gc: &GarbageCollector) -> String {
    let name = gc.get(&self.name).as_str_obj().unwrap();
    format!("<Func '{}' at {:p}>", name.0, self)
  }
}
