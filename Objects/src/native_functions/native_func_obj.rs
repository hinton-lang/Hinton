use crate::native_functions::NATIVES;
use std::fmt::{Debug, Formatter};

/// The index of a native function.
pub type NativeFnIdx = usize;

/// A Hinton string object.
#[derive(PartialEq, Eq, Clone)]
pub struct NativeFuncObj(pub NativeFnIdx);

impl Debug for NativeFuncObj {
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    let func = NATIVES[self.0].name;
    write!(f, "<NativeFunc '{}' at '{:p}'>", func, func)
  }
}
