use crate::chunk::Chunk;
use crate::objects::Object;
use crate::tokens::TokenIdx;
use crate::values::Value;

#[derive(PartialEq, Debug)]
pub struct FuncObj {
  pub defaults: Vec<Value>,
  pub min_arity: u8,
  pub max_arity: u8,
  pub chunk: Chunk,
  pub name: TokenIdx,
  pub up_val_count: usize,
}

impl From<FuncObj> for Value {
  fn from(v: FuncObj) -> Self {
    Value::Obj(Object::Func(v))
  }
}
