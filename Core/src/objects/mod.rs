use std::fmt::{Debug, Formatter};
pub mod func_obj;
pub mod str_obj;

pub enum ObjectKind {
  Str,
  Func,
}

#[derive(PartialEq)]
pub enum Object {
  Str(str_obj::StrObj),
  Func(func_obj::FuncObj),
}

impl Object {
  pub fn kind(&self) -> ObjectKind {
    match self {
      Object::Str(_) => ObjectKind::Str,
      Object::Func(_) => ObjectKind::Func,
    }
  }
}

impl Debug for Object {
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    match self {
      Object::Str(s) => write!(f, "{:?}", s),
      Object::Func(a) => write!(f, "{:?}", a),
    }
  }
}
