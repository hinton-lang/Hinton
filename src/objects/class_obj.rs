use std::cell::RefCell;
use std::fmt;
use std::rc::Rc;

use hashbrown::HashMap;

use core::errors::RuntimeErrorType;
use core::RuntimeResult;

use crate::objects::{ClosureObject, Object};
use crate::virtual_machine::call_frame::CallFrameType;
use crate::virtual_machine::VM;

/// Represents a Hinton class object.
#[derive(Clone)]
pub struct ClassObject {
  pub name: String,
  pub members: HashMap<String, ClassField>,
  pub statics: HashMap<String, ClassField>,
  pub is_constructable: bool,
}

/// Implements the display trait for Hinton class objects.
/// This is how class objects will be displayed when printed from a Hinton program.
impl fmt::Display for ClassObject {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
    write!(f, "<Class '{}' at {:p}>", self.name, &*self as *const _)
  }
}

impl ClassObject {
  /// Creates an empty class object with the given name.
  ///
  /// # Arguments
  /// * `name`: The name of the class.
  ///
  /// # Returns:
  /// ClassObject
  ///
  /// # Examples
  ///
  /// ```
  /// ClassObject::new("Vec2D");
  /// ```
  pub fn new(name: &str) -> Self {
    Self {
      name: name.to_string(),
      members: HashMap::new(),
      statics: HashMap::new(),
      is_constructable: true,
    }
  }

  /// Gets a non-static property from this class.
  ///
  /// # Arguments
  /// * `prop_name`: The name of the property.
  ///
  /// # Returns:
  /// Result<Object, RuntimeResult>
  ///
  /// # Examples
  ///
  /// ```
  /// let vec_2d = Class::new("Vec2D");
  /// // ...
  /// let prop_obj = vec_2d.get_prop("magnitude".to_string());
  /// ```
  pub fn get_non_static_prop(&self, prop_name: String) -> Result<Object, RuntimeResult> {
    match self.members.get(&prop_name) {
      Some(field) => {
        if !field.is_public() {
          Err(RuntimeResult::Error {
            error: RuntimeErrorType::ReferenceError,
            message: format!(
              "Cannot access private property '{}' in object of type '{}'.",
              prop_name, self.name
            ),
          })
        } else {
          Ok(*field.value.clone())
        }
      }
      None => Err(RuntimeResult::Error {
        error: RuntimeErrorType::ReferenceError,
        message: format!(
          "Property '{}' not defined in object of type '{}'.",
          prop_name, self.name
        ),
      }),
    }
  }

  /// Gets a static property from this class.
  ///
  /// # Arguments
  /// * `prop_name`: The name of the property.
  ///
  /// # Returns:
  /// Result<Object, RuntimeResult>
  ///
  /// # Examples
  ///
  /// ```
  /// let vec_2d = Class::new("Vec2D");
  /// // ...
  /// let prop_obj = vec_2d.get_prop("magnitude".to_string());
  /// ```
  pub fn get_static_prop(&self, prop_name: String) -> Result<Object, RuntimeResult> {
    match self.statics.get(&prop_name) {
      Some(field) => {
        if !field.is_public() {
          Err(RuntimeResult::Error {
            error: RuntimeErrorType::ReferenceError,
            message: format!(
              "Cannot access private property '{}' in class '{}'.",
              prop_name, self.name
            ),
          })
        } else {
          Ok(*field.value.clone())
        }
      }
      None => Err(RuntimeResult::Error {
        error: RuntimeErrorType::ReferenceError,
        message: format!("Property '{}' not defined in class '{}'.", prop_name, self.name),
      }),
    }
  }
}

/// Represents a Hinton class field.
#[derive(Clone)]
pub struct ClassField {
  /// The field's value
  pub value: Box<Object>,
  /// The field's configuration mode.                                 \
  /// [public, override, constant]                                    \
  /// [0, 0, 0] = 0 -> (private,    non-override,    non-constant)    \
  /// [0, 0, 1] = 1 -> (private,    non-override,    constant)        \
  /// [0, 1, 0] = 2 -> (private,    override,        non-constant)    \
  /// [0, 1, 1] = 3 -> (private,    override,        constant)        \
  /// [1, 0, 0] = 4 -> (public,     non-override,    non-constant)    \
  /// [1, 0, 1] = 5 -> (public,     non-override,    constant)        \
  /// [1, 1, 0] = 6 -> (public,     override,        non-constant)    \
  /// [1, 1, 1] = 7 -> (public,     override,        constant)
  pub mode: u8,
}

impl ClassField {
  /// Checks that this field is constant.
  pub fn is_constant(&self) -> bool {
    (self.mode & 0b_0000_0001) == 1
  }

  /// Checks that this field is an override.
  pub fn _is_override(&self) -> bool {
    (self.mode & 0b_0000_0010) == 2
  }

  /// Checks that this field is public.
  pub fn is_public(&self) -> bool {
    (self.mode & 0b_0000_0100) == 4
  }
}

/// Represents a Hinton Instance object.
#[derive(Clone)]
pub struct InstanceObject {
  pub class: Rc<RefCell<ClassObject>>,
  pub members: HashMap<String, ClassField>,
}

impl fmt::Display for InstanceObject {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
    write!(
      f,
      "<Instance of '{}' at {:p}>",
      self.class.borrow().name,
      &*self as *const _
    )
  }
}

impl InstanceObject {
  /// Gets a bound-member property from this instance.
  ///
  /// # Arguments
  /// * `prop_name`: The name of the property.
  ///
  /// # Returns:
  /// Result<Object, RuntimeResult>
  ///
  /// # Examples
  ///
  /// ```
  /// let vec_2d = Class::new("Vec2D");
  /// // ...
  /// let prop_obj = vec_2d.get_prop("magnitude".to_string());
  /// ```
  pub fn get_prop(&self, vm: &VM, prop_name: &str) -> Result<Object, RuntimeResult> {
    let is_internal_access = self.is_internal_access(&vm);

    match self.members.get(prop_name) {
      Some(field) => {
        if !field.is_public() & !is_internal_access {
          Err(RuntimeResult::Error {
            error: RuntimeErrorType::ReferenceError,
            message: format!(
              "Cannot access private property '{}' in object of type '{}'.",
              prop_name,
              self.class.borrow().name
            ),
          })
        } else {
          Ok(*field.value.clone())
        }
      }
      None => Err(RuntimeResult::Error {
        error: RuntimeErrorType::ReferenceError,
        message: format!(
          "Property '{}' not defined in object of type '{}'.",
          prop_name,
          self.class.borrow().name
        ),
      }),
    }
  }

  /// Modifies the value of an existing bound-member property of this instance.
  ///
  /// # Arguments
  /// * `prop_name`: The name of the property to be modified.
  /// * `val`: The new value to assign to the property.
  ///
  /// # Returns:
  /// Result<Object, RuntimeResult>
  ///
  /// # Examples
  ///
  /// ```
  /// let vec_2d = Class::new("Vec2D");
  /// // ...
  /// let prop_obj = vec_2d.set_prop("x".to_string(), Object::Int(55i64));
  /// ```
  pub fn set_prop(&mut self, vm: &VM, name: String, val: Object) -> Result<Object, RuntimeResult> {
    let is_internal_access = self.is_internal_access(&vm);

    match self.members.get_mut(&name) {
      Some(field) => {
        if !field.is_public() && !is_internal_access {
          Err(RuntimeResult::Error {
            error: RuntimeErrorType::ReferenceError,
            message: format!(
              "Cannot access private property '{}' in object of type '{}'.",
              name,
              self.class.borrow().name
            ),
          })
        } else if field.is_constant() {
          Err(RuntimeResult::Error {
            error: RuntimeErrorType::ReferenceError,
            message: format!(
              "Cannot reassign to immutable property '{}' in object of type '{}'.",
              name,
              self.class.borrow().name
            ),
          })
        } else {
          field.value = Box::new(val.clone());
          Ok(val)
        }
      }
      None => Err(RuntimeResult::Error {
        error: RuntimeErrorType::ReferenceError,
        message: format!(
          "Property '{}' not defined in object of type '{}'.",
          name,
          self.class.borrow().name
        ),
      }),
    }
  }

  /// Checks that a property access/setter happened from within the class's body.
  /// Since accessing class members from within the class itself is only allowed in the
  /// class's methods, we can check if a field accessed was made from within the class
  /// by comparing the current frame's method receiver (an instance object) against `self`
  /// (also an instance object), and if their memory locations match, then they are the same.
  ///
  /// # Arguments
  /// * `vm`: An immutable reference to the VM.
  ///
  /// # Returns:
  /// bool
  pub fn is_internal_access(&self, vm: &VM) -> bool {
    for frame in vm.frames_stack().iter().rev() {
      if let CallFrameType::Method(m) = &frame.callee {
        return match m.receiver.try_borrow() {
          Ok(r) => std::ptr::eq(&*r.class.borrow(), &*self.class.borrow()),
          // Since `&mut self` refers to "this" instance object, the borrow checker
          // will emit an error if we try to borrow this same instance object again in
          // this match arm. Therefore, we can assume that the method's receiver and
          // `&mut self` refer to the same object (i.e., it's an internal access).
          Err(_) => true,
        };
      }
    }

    false
  }
}

/// Represents a Hinton bound method.
#[derive(Clone)]
pub struct BoundMethod {
  pub receiver: Rc<RefCell<InstanceObject>>,
  pub method: ClosureObject,
}

impl fmt::Display for BoundMethod {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
    let receiver = &self.receiver.borrow();
    let class_name = &receiver.class.borrow().name;
    let method_name = &self.method.function.borrow().name;
    let prt_str = &*self.method.function.borrow() as *const _;

    write!(f, "<Method '{}.{}' at {:p}>", class_name, method_name, prt_str)
  }
}
