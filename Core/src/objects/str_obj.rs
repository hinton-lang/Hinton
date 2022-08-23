use crate::objects::Object;
use crate::values::Value;
use std::fmt::{Debug, Formatter};

#[derive(PartialEq, Eq)]
pub struct StrObj(pub String);

impl From<StrObj> for Value {
  fn from(v: StrObj) -> Self {
    Value::Obj(Object::Str(v))
  }
}

impl Debug for StrObj {
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    write!(f, "'{}'", self.0)
  }
}
