use std::io::{stdin, stdout, Write};

use core::errors::RuntimeErrMsg;
use core::utils::get_time_millis;

use crate::{GarbageCollector, Object, OBJ_FALSE, OBJ_NONE, OBJ_TRUE};

/// The index of a native function.
pub type NativeFnIdx = usize;

/// A Hinton string object.
#[derive(PartialEq, Eq, Copy, Clone)]
pub struct NativeFuncObj(pub NativeFnIdx);

impl From<usize> for NativeFuncObj {
  fn from(idx: usize) -> Self {
    NativeFuncObj(idx)
  }
}

impl NativeFuncObj {
  pub fn display_plain(&self) -> String {
    let func = &NATIVES[self.0];
    format!("<NativeFunc '{}' at {:p}>", func.name, func)
  }
}

/// Represents the body of a Hinton native function object.
pub struct NativeFn<'a> {
  pub name: &'a str,
  pub min_arity: u16,
  pub max_arity: Option<u16>,
  pub body: NativeFnBody,
}

/// The signature of the body of a native function.
pub type NativeFnBody = fn(&mut GarbageCollector, &mut [Object]) -> NativeFnResult;

/// The result of a native function execution.
pub type NativeFnResult = Result<Object, RuntimeErrMsg>;

macro_rules! native {
  ($name:expr,$min_arity:expr,$max_arity:expr,$body:expr) => {{
    NativeFn {
      name: $name,
      min_arity: $min_arity,
      max_arity: $max_arity,
      body: $body,
    }
  }};
}

/// How many native functions there are.
pub const NATIVES_LEN: usize = 6;

/// The native functions of Hinton.
pub const NATIVES: [NativeFn; NATIVES_LEN] = [
  native!["print", 0, None, native_print],
  native!["println", 0, None, native_println],
  native!["input", 1, Some(1), native_input],
  native!["clock", 0, Some(0), native_clock],
  native!["id", 1, Some(1), native_id],
  native!["type_of", 1, Some(1), native_type_of],
];

/// Implements the `print(...)` native function for Hinton, which prints a value to the console.
///
/// # Arguments
/// * `gc`: A mutable reference to the garbage collector.
/// * `args`: A vector of objects that will serve as arguments to this function call.
///
/// # Returns:
/// ```NativeFnResult```
fn native_print(gc: &mut GarbageCollector, args: &mut [Object]) -> NativeFnResult {
  let mut iter = args.iter().peekable();

  // TODO: The user should be allowed to specify the separator as a keyword argument.
  let sep = " ";

  while let Some(arg) = iter.next() {
    if iter.peek().is_some() {
      print!("{}{}", arg.display_pretty(gc), sep);
    } else {
      print!("{}", arg.display_pretty(gc));
    }
  }

  Ok(OBJ_NONE)
}

/// Implements the `println(...)` native function for Hinton, which prints a value to
/// the console, ending with the new line character
///
/// # Arguments
/// * `gc`: A mutable reference to the garbage collector.
/// * `args`: A vector of objects that will serve as arguments to this function call.
///
/// # Returns:
/// ```NativeFnResult```
fn native_println(gc: &mut GarbageCollector, args: &mut [Object]) -> NativeFnResult {
  native_print(gc, args)?;
  println!();
  Ok(OBJ_NONE)
}

/// Implements the `input(...)` native function for Hinton, which gets user input from the console.
///
/// # Arguments
/// * `gc`: A mutable reference to the garbage collector.
/// * `args`: A vector of objects that will serve as arguments to this function call.
///
/// # Returns:
/// ```NativeFnResult```
fn native_input(gc: &mut GarbageCollector, args: &mut [Object]) -> NativeFnResult {
  print!("{}", args[0].debug_fmt(gc));

  // Print the programmer-provided message
  match Write::flush(&mut stdout()) {
    Ok(_) => {
      let mut input = String::new();
      match stdin().read_line(&mut input) {
        Ok(_) => {
          input.pop(); // remove added newline
          Ok(Object::Str(gc.push(input.into())))
        }
        Err(e) => {
          let err_msg = format!("Failed to read input. IO failed read line. {}", e);
          println!("{err_msg}");
          Err(RuntimeErrMsg::IO(err_msg))
        }
      }
    }
    Err(e) => {
      let err_msg = format!("Failed to read input. IO failed read line. {}", e);
      println!("{err_msg}");
      Err(RuntimeErrMsg::Type(err_msg))
    }
  }
}

/// Implements the `clock()` native function for Hinton, which retrieves the current time
/// in milliseconds from the Unix Epoch time.
///
/// # Arguments
/// * `gc`: A mutable reference to the garbage collector.
/// * `args`: A vector of objects that will serve as arguments to this function call.
///
/// # Returns:
/// ```NativeFnResult```
fn native_clock(_: &mut GarbageCollector, _: &mut [Object]) -> NativeFnResult {
  Ok(Object::Int(get_time_millis() as i64))
}

/// Implements the `id()` native function for Hinton, which retrieves
/// an object's memory address as an integer.
///
/// # Arguments
/// * `gc`: A mutable reference to the garbage collector.
/// * `args`: A vector of objects that will serve as arguments to this function call.
///
/// # Returns:
/// ```NativeFnResult```
fn native_id(gc: &mut GarbageCollector, args: &mut [Object]) -> NativeFnResult {
  // TODO: This seems to work well, but is it actually correct?
  let raw_val = match args[0] {
    Object::None => &OBJ_NONE as *const _ as *mut i64,
    Object::Int(mut i) => &mut i as *const _ as *mut i64,
    Object::Float(mut f) => &mut f as *const _ as *mut i64,
    Object::Bool(true) => &OBJ_TRUE as *const _ as *mut i64,
    Object::Bool(false) => &OBJ_FALSE as *const _ as *mut i64,
    Object::Func(id) | Object::Str(id) => &mut gc.get_mut(&id).obj as *const _ as *mut _,
    Object::NativeFunc(idx) => &NATIVES[idx.0] as *const _ as *mut _,
  };

  Ok(Object::Int(raw_val as i64))
}

/// Implements the `type_of()` native function for Hinton, which retrieves
/// an object's type name as a string
///
/// # Arguments
/// * `gc`: A mutable reference to the garbage collector.
/// * `args`: A vector of objects that will serve as arguments to this function call.
///
/// # Returns:
/// ```NativeFnResult```
fn native_type_of(gc: &mut GarbageCollector, args: &mut [Object]) -> NativeFnResult {
  let name = gc.push(args[0].type_name().into());
  Ok(Object::Str(name))
}
