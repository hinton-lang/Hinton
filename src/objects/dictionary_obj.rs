use crate::errors::ObjectOprErrType;
use crate::objects::Object;
use crate::virtual_machine::RuntimeResult;
use hashbrown::HashMap;
use std::cell::RefCell;
use std::fmt;
use std::fmt::Formatter;
use std::rc::Rc;

/// Represents a Hinton dictionary object.
#[derive(Clone)]
pub struct DictObject(pub(crate) Rc<RefCell<HashMap<String, Object>>>);

/// Implements the display trait for Hinton dictionary objects.
/// This is how dictionary objects will be displayed when printed from a Hinton program.
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
   /// Creates a new Dictionary object from a HashMap.
   ///
   /// # Arguments
   /// * `map`: The HashMap to be used for the dictionary.
   ///
   /// # Returns:
   /// Object
   ///
   /// # Examples
   ///
   /// ```
   /// let my_obj_hash_map: HashMap<String, Object> = HashMap::new();
   /// let my_dict_obj = DictObject::from_hashmap(my_obj_hash_map);
   /// ```
   pub fn from_hashmap(map: HashMap<String, Object>) -> Object {
      Object::Dict(DictObject(Rc::new(RefCell::new(map))))
   }

   /// Gets the object associated with a property in this dictionary.
   ///
   /// # Arguments
   /// * `prop_name`: The property's name.
   ///
   /// # Returns:
   /// Result<Object, ObjectOprErrType>
   pub fn get_prop(&self, prop_name: &str) -> Result<Object, ObjectOprErrType> {
      match self.0.borrow().get(prop_name) {
         Some(o) => Ok(o.clone()),
         None => Err(ObjectOprErrType::KeyError(format!(
            "Entry with key '{}' not found in the dictionary.",
            prop_name
         ))),
      }
   }

   /// Inserts a new property into this dictionary. If the property already exists, the value
   /// gets overwritten by the new value.
   ///
   /// # Arguments
   /// * `prop_name`: The name of the property.
   /// * `val`: The value for the property.
   ///
   /// # Returns:
   /// RuntimeResult
   pub fn set_prop(&mut self, prop_name: &str, val: Object) -> RuntimeResult {
      self.0.borrow_mut().insert(prop_name.to_string(), val);
      RuntimeResult::Continue
   }
}
