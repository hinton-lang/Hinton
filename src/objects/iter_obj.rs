use std::cell::RefCell;
use std::fmt;
use std::fmt::Formatter;
use std::rc::Rc;

use core::errors::RuntimeErrMsg;
use core::RuntimeResult;

use crate::objects::Object;

/// Represents a Hinton iterator object.
pub struct IterObject {
  pub iter: Box<Object>,
  pub index: usize,
}

/// Implements the display trait for Hinton iterator objects.
/// This is how iterator objects will be displayed when printed from a Hinton program.
impl fmt::Display for IterObject {
  fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), fmt::Error> {
    write!(f, "<Iterable '{}'>", self.iter.type_name())
  }
}

/// Converts a Hinton object into an Iterable object.
///
/// # Arguments
/// * `o`: The object to be converted to an iterable.
///
/// # Returns:
/// Result<Object, RuntimeResult>
pub fn make_iter(o: Object) -> Result<Object, RuntimeResult> {
  match o {
    Object::String(_) | Object::Array(_) | Object::Range(_) | Object::Tuple(_) => {
      Ok(Object::Iter(Rc::new(RefCell::new(IterObject {
        iter: Box::new(o),
        index: 0,
      }))))
    }
    // If the object is already an iterable, return that same object.
    Object::Iter(_) => Ok(o),
    _ => Err(RuntimeResult::Error(RuntimeErrMsg::Type(format!(
      "Cannot create iterable from '{}'.",
      o.type_name()
    )))),
  }
}

/// Gets the next item in a Hinton iterator.
///
/// # Arguments
/// * `o`: A mutable, counted reference into the iterator.
///
/// # Returns:
/// Result<Object, RuntimeResult>
pub fn get_next_in_iter(o: &Rc<RefCell<IterObject>>) -> Result<Object, RuntimeResult> {
  let mut iter = o.borrow_mut();
  let current_index = Object::Int(iter.index as i64);

  match iter.iter.subscript(&current_index) {
    Ok(o) => {
      iter.index += 1;
      Ok(o)
    }
    Err(_) => Err(RuntimeResult::Error(RuntimeErrMsg::IterStrop(
      "End of Iterator.".to_string(),
    ))),
  }
}
