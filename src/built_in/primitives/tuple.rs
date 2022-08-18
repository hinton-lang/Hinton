use std::cell::RefCell;
use std::rc::Rc;

use hashbrown::HashMap;

use core::errors::RuntimeErrMsg;
use core::RuntimeResult;

use crate::built_in::primitives::HTPrimitive;
use crate::built_in::NativeBoundMethod;
use crate::objects::class_obj::{ClassField, ClassObject};
use crate::objects::Object;
use crate::virtual_machine::VM;

/// Represents the Hinton `Tuple` primitive class.
pub struct TupleClass(ClassObject);

/// Implements the `HTPrimitive` trait for the `TupleClass` primitive Hinton class.
impl HTPrimitive for TupleClass {
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
    let mut _self = TupleClass(ClassObject::new("Tuple"));

    // >>>>>>> Class fields for the "Tuple" primitive type to be added after this line
    _self.bind_non_static_method("index_of", (1, 1), index_of as NativeBoundMethod);
    _self.bind_non_static_method("len", (0, 0), len as NativeBoundMethod);
    _self.bind_non_static_method("to_array", (0, 0), to_array as NativeBoundMethod);
    // <<<<<<< Class fields for the "Tuple" primitive type to be added before this line

    _self.0
  }
}

macro_rules! verify_tuple_object {
  ($maybe_tuple: expr, $prop_name: expr) => {
    match $maybe_tuple {
      Object::Tuple(a) => a,
      _ => {
        return RuntimeResult::Error(RuntimeErrMsg::Type(format!(
          "Property 'Tuple.{}' requires that 'self' be a Tuple. Found '{}' instead.",
          $prop_name,
          $maybe_tuple.type_name()
        )));
      }
    }
  };
}

/// Gets the length of a Hinton tuple.
///
/// # Arguments
/// * `vm`: A mutable reference to the virtual machine.
/// * `this`: The tuple object.
/// * `_`: A vector of objects that will serve as arguments to this method call.
///
/// # Returns:
/// RuntimeResult
fn len(vm: &mut VM, this: Object, _: Vec<Object>) -> RuntimeResult {
  vm.push_stack(Object::from(verify_tuple_object!(this, "len").len()))
}

/// Gets the index of the first occurrence of an object in this Hinton tuple.
///
/// # Arguments
/// * `vm`: A mutable reference to the virtual machine.
/// * `this`: The tuple object.
/// * `args`: A vector of objects that will serve as arguments to this method call.
///
/// # Returns:
/// RuntimeResult
fn index_of(vm: &mut VM, this: Object, args: Vec<Object>) -> RuntimeResult {
  let obj = args[0].clone();

  match verify_tuple_object!(this, "index_of").iter().position(|x| x == &obj) {
    Some(i) => vm.push_stack(Object::Int(i as i64)),
    None => vm.push_stack(Object::None),
  }
}

/// Converts this Hinton tuple into a Hinton array.
///
/// # Arguments
/// * `vm`: A mutable reference to the virtual machine.
/// * `this`: The tuple object.
/// * `_`: A vector of objects that will serve as arguments to this method call.
///
/// # Returns:
/// RuntimeResult
fn to_array(vm: &mut VM, this: Object, _: Vec<Object>) -> RuntimeResult {
  let tup = &*verify_tuple_object!(this, "to_tuple");
  vm.push_stack(Object::Array(Rc::new(RefCell::new(tup.clone()))))
}
