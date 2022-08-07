use std::f64::consts::{E, FRAC_1_SQRT_2, LN_10, LN_2, LOG10_E, LOG2_E, PI, SQRT_2, TAU};

use hashbrown::HashMap;

use core::errors::RuntimeErrorType;
use core::RuntimeResult;

use crate::built_in::primitives::HTPrimitive;
use crate::built_in::NativeBoundMethod;
use crate::objects::class_obj::{ClassField, ClassObject};
use crate::objects::Object;
use crate::virtual_machine::VM;

/// Represents the Hinton `Math` built-in class.
pub struct MathClass(ClassObject);

/// Implements the `HTPrimitive` trait for the `Math` built-in Hinton class.
impl HTPrimitive for MathClass {
  /// Gets the name class name of this Hinton primitive.
  fn name(&self) -> String {
    self.0.name.clone()
  }

  /// Gets the non-static members of this Hinton built-in class.
  fn members(&mut self) -> &mut HashMap<String, ClassField> {
    &mut self.0.members
  }

  /// Gets the static members of this Hinton built-in class.
  fn statics(&mut self) -> &mut HashMap<String, ClassField> {
    &mut self.0.statics
  }

  /// Gets the default class object stored in this Hinton built-in class.
  fn default() -> ClassObject {
    let mut _self = MathClass(ClassObject::new("Math"));

    // Mark the class as non-constructable so that new instances of the class
    // cannot be created by the programmer.
    _self.0.is_constructable = false;

    // >>>>>>> Class fields for the "Math" built-in class to be added after this line.
    _self.bind_field("E", true, true, false, true, Object::Float(E));
    _self.bind_field("LN_10", true, true, false, true, Object::Float(LN_10));
    _self.bind_field("LN_2", true, true, false, true, Object::Float(LN_2));
    _self.bind_field("LOG10_E", true, true, false, true, Object::Float(LOG10_E));
    _self.bind_field("LOG2_E", true, true, false, true, Object::Float(LOG2_E));
    _self.bind_field("PI", true, true, false, true, Object::Float(PI));
    _self.bind_field("SQRT_1_2", true, true, false, true, Object::Float(FRAC_1_SQRT_2));
    _self.bind_field("SQRT_2", true, true, false, true, Object::Float(SQRT_2));
    _self.bind_field("TAU", true, true, false, true, Object::Float(TAU));
    _self.bind_static_method("acos", (1, 1), acos as NativeBoundMethod);
    _self.bind_static_method("acosh", (1, 1), acosh as NativeBoundMethod);
    _self.bind_static_method("asin", (1, 1), asin as NativeBoundMethod);
    _self.bind_static_method("asinh", (1, 1), asinh as NativeBoundMethod);
    _self.bind_static_method("atan", (1, 1), atan as NativeBoundMethod);
    _self.bind_static_method("atan2", (2, 2), atan2 as NativeBoundMethod);
    _self.bind_static_method("atanh", (1, 1), atanh as NativeBoundMethod);
    _self.bind_static_method("cbrt", (1, 1), cbrt as NativeBoundMethod);
    _self.bind_static_method("cos", (1, 1), cos as NativeBoundMethod);
    _self.bind_static_method("cosh", (1, 1), cosh as NativeBoundMethod);
    _self.bind_static_method("exp", (1, 1), exp as NativeBoundMethod);
    _self.bind_static_method("ln", (1, 1), ln as NativeBoundMethod);
    _self.bind_static_method("log", (2, 2), log as NativeBoundMethod);
    _self.bind_static_method("log10", (1, 1), log10 as NativeBoundMethod);
    _self.bind_static_method("log2", (1, 1), log2 as NativeBoundMethod);
    _self.bind_static_method("sin", (1, 1), sin as NativeBoundMethod);
    _self.bind_static_method("sinh", (1, 1), sinh as NativeBoundMethod);
    _self.bind_static_method("sqrt", (1, 1), sqrt as NativeBoundMethod);
    _self.bind_static_method("tan", (1, 1), tan as NativeBoundMethod);
    _self.bind_static_method("tanh", (1, 1), tanh as NativeBoundMethod);
    // <<<<<<< Class fields for the "Math" built-in class to be added before this line.

    _self.0
  }
}

macro_rules! check_int_or_float {
  ($o: expr) => {
    match $o {
      Object::Float(f) => f,
      Object::Int(i) => i as f64,
      other => {
        return RuntimeResult::Error {
          error: RuntimeErrorType::TypeError,
          message: format!(
            "Expected argument of type 'Int' or 'Float'. Got '{}' instead.",
            other.type_name()
          ),
        }
      }
    }
  };
}

/// Computes the sine of a Hinton number.
///
/// # Arguments
/// * `vm`: A mutable reference to the virtual machine.
/// * `_`:  *Not applicable.*
/// * `args`: A vector of objects that will serve as arguments to this method call.
///
/// # Returns:
/// RuntimeResult
fn sin(vm: &mut VM, _: Object, args: Vec<Object>) -> RuntimeResult {
  let arg = check_int_or_float!(args[0].clone());
  vm.push_stack(Object::Float(arg.sin()))
}

/// Computes the arcsine of a Hinton number.
///
/// # Arguments
/// * `vm`: A mutable reference to the virtual machine.
/// * `_`:  *Not applicable.*
/// * `args`: A vector of objects that will serve as arguments to this method call.
///
/// # Returns:
/// RuntimeResult
fn asin(vm: &mut VM, _: Object, args: Vec<Object>) -> RuntimeResult {
  let arg = check_int_or_float!(args[0].clone());
  vm.push_stack(Object::Float(arg.asin()))
}

/// Computes the hyperbolic sine of a Hinton number.
///
/// # Arguments
/// * `vm`: A mutable reference to the virtual machine.
/// * `_`:  *Not applicable.*
/// * `args`: A vector of objects that will serve as arguments to this method call.
///
/// # Returns:
/// RuntimeResult
fn sinh(vm: &mut VM, _: Object, args: Vec<Object>) -> RuntimeResult {
  let arg = check_int_or_float!(args[0].clone());
  vm.push_stack(Object::Float(arg.sinh()))
}

/// Computes the hyperbolic arcsine of a Hinton number.
///
/// # Arguments
/// * `vm`: A mutable reference to the virtual machine.
/// * `_`:  *Not applicable.*
/// * `args`: A vector of objects that will serve as arguments to this method call.
///
/// # Returns:
/// RuntimeResult
fn asinh(vm: &mut VM, _: Object, args: Vec<Object>) -> RuntimeResult {
  let arg = check_int_or_float!(args[0].clone());
  vm.push_stack(Object::Float(arg.asinh()))
}

/// Computes the cosine of a Hinton number.
///
/// # Arguments
/// * `vm`: A mutable reference to the virtual machine.
/// * `_`:  *Not applicable.*
/// * `args`: A vector of objects that will serve as arguments to this method call.
///
/// # Returns:
/// RuntimeResult
fn cos(vm: &mut VM, _: Object, args: Vec<Object>) -> RuntimeResult {
  let arg = check_int_or_float!(args[0].clone());
  vm.push_stack(Object::Float(arg.cos()))
}

/// Computes the arccosine of a Hinton number.
///
/// # Arguments
/// * `vm`: A mutable reference to the virtual machine.
/// * `_`:  *Not applicable.*
/// * `args`: A vector of objects that will serve as arguments to this method call.
///
/// # Returns:
/// RuntimeResult
fn acos(vm: &mut VM, _: Object, args: Vec<Object>) -> RuntimeResult {
  let arg = check_int_or_float!(args[0].clone());
  vm.push_stack(Object::Float(arg.acos()))
}

/// Computes the hyperbolic cosine of a Hinton number.
///
/// # Arguments
/// * `vm`: A mutable reference to the virtual machine.
/// * `_`:  *Not applicable.*
/// * `args`: A vector of objects that will serve as arguments to this method call.
///
/// # Returns:
/// RuntimeResult
fn cosh(vm: &mut VM, _: Object, args: Vec<Object>) -> RuntimeResult {
  let arg = check_int_or_float!(args[0].clone());
  vm.push_stack(Object::Float(arg.cosh()))
}

/// Computes the hyperbolic arccosine of a Hinton number.
///
/// # Arguments
/// * `vm`: A mutable reference to the virtual machine.
/// * `_`:  *Not applicable.*
/// * `args`: A vector of objects that will serve as arguments to this method call.
///
/// # Returns:
/// RuntimeResult
fn acosh(vm: &mut VM, _: Object, args: Vec<Object>) -> RuntimeResult {
  let arg = check_int_or_float!(args[0].clone());
  vm.push_stack(Object::Float(arg.acosh()))
}

/// Computes the tangent of a Hinton number.
///
/// # Arguments
/// * `vm`: A mutable reference to the virtual machine.
/// * `_`:  *Not applicable.*
/// * `args`: A vector of objects that will serve as arguments to this method call.
///
/// # Returns:
/// RuntimeResult
fn tan(vm: &mut VM, _: Object, args: Vec<Object>) -> RuntimeResult {
  let arg = check_int_or_float!(args[0].clone());
  vm.push_stack(Object::Float(arg.tan()))
}

/// Computes the arctangent of a Hinton number.
///
/// # Arguments
/// * `vm`: A mutable reference to the virtual machine.
/// * `_`:  *Not applicable.*
/// * `args`: A vector of objects that will serve as arguments to this method call.
///
/// # Returns:
/// RuntimeResult
fn atan(vm: &mut VM, _: Object, args: Vec<Object>) -> RuntimeResult {
  let arg = check_int_or_float!(args[0].clone());
  vm.push_stack(Object::Float(arg.atan()))
}

/// Computes the hyperbolic tangent of a Hinton number.
///
/// # Arguments
/// * `vm`: A mutable reference to the virtual machine.
/// * `_`:  *Not applicable.*
/// * `args`: A vector of objects that will serve as arguments to this method call.
///
/// # Returns:
/// RuntimeResult
fn tanh(vm: &mut VM, _: Object, args: Vec<Object>) -> RuntimeResult {
  let arg = check_int_or_float!(args[0].clone());
  vm.push_stack(Object::Float(arg.tanh()))
}

/// Computes the hyperbolic arctangent of a Hinton number.
///
/// # Arguments
/// * `vm`: A mutable reference to the virtual machine.
/// * `_`:  *Not applicable.*
/// * `args`: A vector of objects that will serve as arguments to this method call.
///
/// # Returns:
/// RuntimeResult
fn atanh(vm: &mut VM, _: Object, args: Vec<Object>) -> RuntimeResult {
  let arg = check_int_or_float!(args[0].clone());
  vm.push_stack(Object::Float(arg.atanh()))
}

/// Computes the arctangent of the quotient of two Hinton numbers.
///
/// # Arguments
/// * `vm`: A mutable reference to the virtual machine.
/// * `_`:  *Not applicable.*
/// * `args`: A vector of objects that will serve as arguments to this method call.
///
/// # Returns:
/// RuntimeResult
fn atan2(vm: &mut VM, _: Object, args: Vec<Object>) -> RuntimeResult {
  let arg1 = check_int_or_float!(args[0].clone());
  let arg2 = check_int_or_float!(args[1].clone());
  vm.push_stack(Object::Float(arg1.atan2(arg2)))
}

/// Computes the square root of a Hinton number.
///
/// # Arguments
/// * `vm`: A mutable reference to the virtual machine.
/// * `_`:  *Not applicable.*
/// * `args`: A vector of objects that will serve as arguments to this method call.
///
/// # Returns:
/// RuntimeResult
fn sqrt(vm: &mut VM, _: Object, args: Vec<Object>) -> RuntimeResult {
  let arg = check_int_or_float!(args[0].clone());
  vm.push_stack(Object::Float(arg.sqrt()))
}

/// Computes the cube root of a Hinton number.
///
/// # Arguments
/// * `vm`: A mutable reference to the virtual machine.
/// * `_`:  *Not applicable.*
/// * `args`: A vector of objects that will serve as arguments to this method call.
///
/// # Returns:
/// RuntimeResult
fn cbrt(vm: &mut VM, _: Object, args: Vec<Object>) -> RuntimeResult {
  let arg = check_int_or_float!(args[0].clone());
  vm.push_stack(Object::Float(arg.cbrt()))
}

/// Computes the exponent (e<sup>x</sup>) of a Hinton number.
///
/// # Arguments
/// * `vm`: A mutable reference to the virtual machine.
/// * `_`:  *Not applicable.*
/// * `args`: A vector of objects that will serve as arguments to this method call.
///
/// # Returns:
/// RuntimeResult
fn exp(vm: &mut VM, _: Object, args: Vec<Object>) -> RuntimeResult {
  let arg = check_int_or_float!(args[0].clone());
  vm.push_stack(Object::Float(arg.exp()))
}

/// Computes the logarithm of a Hinton number with an arbitrary base.
///
/// # Arguments
/// * `vm`: A mutable reference to the virtual machine.
/// * `_`:  *Not applicable.*
/// * `args`: A vector of objects that will serve as arguments to this method call.
///
/// # Returns:
/// RuntimeResult
fn log(vm: &mut VM, _: Object, args: Vec<Object>) -> RuntimeResult {
  let arg1 = check_int_or_float!(args[0].clone());
  let arg2 = check_int_or_float!(args[0].clone());
  vm.push_stack(Object::Float(arg1.log(arg2)))
}

/// Computes the log-base-2 (log<sub>2</sub>(x)) of a Hinton number.
///
/// # Arguments
/// * `vm`: A mutable reference to the virtual machine.
/// * `_`:  *Not applicable.*
/// * `args`: A vector of objects that will serve as arguments to this method call.
///
/// # Returns:
/// RuntimeResult
fn log2(vm: &mut VM, _: Object, args: Vec<Object>) -> RuntimeResult {
  let arg1 = check_int_or_float!(args[0].clone());
  vm.push_stack(Object::Float(arg1.log2()))
}

/// Computes the log-base-10 (log<sub>10</sub>(x)) of a Hinton number.
///
/// # Arguments
/// * `vm`: A mutable reference to the virtual machine.
/// * `_`:  *Not applicable.*
/// * `args`: A vector of objects that will serve as arguments to this method call.
///
/// # Returns:
/// RuntimeResult
fn log10(vm: &mut VM, _: Object, args: Vec<Object>) -> RuntimeResult {
  let arg1 = check_int_or_float!(args[0].clone());
  vm.push_stack(Object::Float(arg1.log10()))
}

/// Computes the natural logarithm (log<sub>e</sub>(x)) of a Hinton number.
///
/// # Arguments
/// * `vm`: A mutable reference to the virtual machine.
/// * `_`:  *Not applicable.*
/// * `args`: A vector of objects that will serve as arguments to this method call.
///
/// # Returns:
/// RuntimeResult
fn ln(vm: &mut VM, _: Object, args: Vec<Object>) -> RuntimeResult {
  let arg1 = check_int_or_float!(args[0].clone());
  vm.push_stack(Object::Float(arg1.ln()))
}
