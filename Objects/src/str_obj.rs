use crate::gc::GcTrace;
use crate::GcObject;

/// A Hinton string object.
#[derive(PartialEq, Eq)]
pub struct StrObj(pub String);

impl GcTrace for StrObj {}

impl StrObj {
  pub fn display_plain(&self) -> &String {
    &self.0
  }

  pub fn equals(&self, obj: &GcObject) -> bool {
    match obj {
      GcObject::Str(s) => self.0 == s.0,
      _ => false,
    }
  }
}
