use crate::errors::ObjectOprErrType;
use crate::objects::Object;
use crate::virtual_machine::RuntimeResult;
use hashbrown::HashMap;
use std::cell::RefCell;
use std::fmt;
use std::fmt::Formatter;
use std::rc::Rc;

#[derive(Clone)]
pub struct DictObject(pub(crate) Rc<RefCell<HashMap<String, Object>>>);

impl fmt::Display for DictObject {
   fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
      let mut arr_str = String::from("{");

      for (idx, key) in self.0.borrow().keys().enumerate() {
         if idx == self.0.borrow().keys().len() - 1 {
            arr_str += &(format!("'{}': {}", key, self.0.borrow().get(key).unwrap()))[..]
         } else {
            arr_str += &(format!("'{}': {}, ", key, self.0.borrow().get(key).unwrap()))[..]
         }
      }

      arr_str += "}";

      write!(f, "{}", arr_str)
   }
}

impl DictObject {
   pub fn new_object(x: HashMap<String, Object>) -> Object {
      Object::Dict(DictObject(Rc::new(RefCell::new(x))))
   }

   pub fn get_prop(&self, prop_name: &str) -> Result<Object, ObjectOprErrType> {
      match self.0.borrow().get(prop_name) {
         Some(o) => Ok(o.clone()),
         None => Err(ObjectOprErrType::KeyError(format!(
            "Entry with key '{}' not found in the dictionary.",
            prop_name
         ))),
      }
   }

   pub fn set_prop(&mut self, prop_name: &str, val: Object) -> RuntimeResult {
      self.0.borrow_mut().insert(prop_name.to_string(), val);
      RuntimeResult::Continue
   }
}
