use std::fmt::{Debug, Formatter};

use crate::gc::GcTrace;

/// A Hinton string object.
#[derive(PartialEq, Eq, Clone)]
pub struct StrObj(pub String);

impl Debug for StrObj {
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    write!(f, "'{}'", self.0)
  }
}

impl GcTrace for StrObj {}

impl StrObj {
  pub fn display_plain(&self) -> &String {
    &self.0
  }
}
