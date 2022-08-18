use hashbrown::HashMap;

use core::errors::RuntimeErrMsg;
use core::RuntimeResult;

use crate::built_in::primitives::HTPrimitive;
use crate::built_in::NativeBoundMethod;
use crate::objects::class_obj::{ClassField, ClassObject};
use crate::objects::Object;
use crate::virtual_machine::VM;

/// Represents the Hinton `String` primitive class.
pub struct StringClass(ClassObject);

/// Implements the `HTPrimitive` trait for the `StringClass` primitive Hinton class.
impl HTPrimitive for StringClass {
  /// Gets the name class name of this Hinton primitive.
  fn name(&self) -> String {
    self.0.name.clone()
  }

  /// Gets the non-static members of this Hinton primitive.
  fn members(&mut self) -> &mut HashMap<String, ClassField> {
    &mut self.0.members
  }

  /// Gets the static members of this Hinton primitive.
  fn statics(&mut self) -> &mut HashMap<String, ClassField> {
    &mut self.0.statics
  }

  /// Gets the default class object stored in this Hinton primitive.
  fn default() -> ClassObject {
    let mut _self = StringClass(ClassObject::new("String"));

    // >>>>>>> Class fields for the "String" primitive type to be added after this line
    _self.bind_non_static_method("ends_with", (1, 1), ends_with as NativeBoundMethod);
    _self.bind_non_static_method("len", (0, 0), len as NativeBoundMethod);
    _self.bind_non_static_method("starts_with", (1, 1), starts_with as NativeBoundMethod);
    _self.bind_non_static_method("to_lower", (0, 0), to_lower as NativeBoundMethod);
    // <<<<<<< Class fields for the "String" primitive type to be added before this line

    _self.0
  }
}

macro_rules! verify_string_object {
  ($maybe_string: expr, $prop_name: expr) => {
    match $maybe_string {
      Object::String(a) => a,
      _ => {
        return RuntimeResult::Error(RuntimeErrMsg::Type(format!(
          "Property 'String.{}' requires that 'self' be a String. Found '{}' instead.",
          $prop_name,
          $maybe_string.type_name()
        )))
      }
    }
  };
}

/// Gets the length of a Hinton string.
///
/// # Arguments
/// * `vm`: A mutable reference to the virtual machine.
/// * `this`: The array object.
/// * `_`: A vector of objects that will serve as arguments to this method call.
///
/// # Returns:
/// RuntimeResult
fn len(vm: &mut VM, this: Object, _: Vec<Object>) -> RuntimeResult {
  vm.push_stack(Object::from(verify_string_object!(this, "len").len()))
}

/// Creates a copy of a string and lowercase it.
///
/// # Arguments
/// * `vm`: A mutable reference to the virtual machine.
/// * `this`: The array object.
/// * `_`: A vector of objects that will serve as arguments to this method call.
///
/// # Returns:
/// RuntimeResult
fn to_lower(vm: &mut VM, this: Object, _: Vec<Object>) -> RuntimeResult {
  vm.push_stack(Object::from(verify_string_object!(this, "to_lower").to_lowercase()))
}

/// Checks if a string ends with another string.
///
/// # Arguments
/// * `vm`: A mutable reference to the virtual machine.
/// * `this`: The array object.
/// * `args`: A vector of objects that will serve as arguments to this method call.
///
/// # Returns:
/// RuntimeResult
fn ends_with(vm: &mut VM, this: Object, args: Vec<Object>) -> RuntimeResult {
  match args[0].clone() {
    Object::String(s) => vm.push_stack(Object::Bool(verify_string_object!(this, "ends_with").ends_with(&s))),
    other => RuntimeResult::Error(RuntimeErrMsg::Type(format!(
      "Expected argument of type 'String'. Got '{}' instead.",
      other.type_name()
    ))),
  }
}

/// Checks if a string starts with another string.
///
/// # Arguments
/// * `vm`: A mutable reference to the virtual machine.
/// * `this`: The array object.
/// * `args`: A vector of objects that will serve as arguments to this method call.
///
/// # Returns:
/// RuntimeResult
fn starts_with(vm: &mut VM, this: Object, args: Vec<Object>) -> RuntimeResult {
  match args[0].clone() {
    Object::String(s) => vm.push_stack(Object::Bool(verify_string_object!(this, "ends_with").starts_with(&s))),
    other => RuntimeResult::Error(RuntimeErrMsg::Type(format!(
      "Expected argument of type 'String'. Got '{}' instead.",
      other.type_name()
    ))),
  }
}
