use std::fmt::{Debug, Formatter};

use crate::objects::{Object, ObjectKind};

pub enum ValueKind {
  None,
  Int,
  Float,
  Bool,
  // From objects
  Func,
  Str,
}

#[derive(PartialEq)]
pub enum Value {
  None,
  Int(i64),
  Float(f64),
  Bool(bool),
  Obj(Object),
}

pub const VAL_TRUE: Value = Value::Bool(true);
pub const VAL_FALSE: Value = Value::Bool(false);
pub const VAL_NONE: Value = Value::None;

impl Debug for Value {
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    match self {
      Value::None => write!(f, "none"),
      Value::Int(_i) => write!(f, "{_i}"),
      Value::Float(_s) => write!(f, "{_s}"),
      Value::Bool(true) => write!(f, "true"),
      Value::Bool(false) => write!(f, "false"),
      Value::Obj(o) => write!(f, "{:?}", o),
    }
  }
}

impl Value {
  pub fn kind(&self) -> ValueKind {
    match self {
      Value::None => ValueKind::None,
      Value::Int(_) => ValueKind::Int,
      Value::Float(_) => ValueKind::Float,
      Value::Bool(_) => ValueKind::Bool,
      Value::Obj(obj) => match obj.kind() {
        ObjectKind::Str => ValueKind::Str,
        ObjectKind::Func => ValueKind::Func,
      },
    }
  }
}
