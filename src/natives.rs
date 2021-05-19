use crate::objects::{IterObject, NativeFunctionObj, Object};
use std::{borrow::Borrow, cell::RefCell, collections::HashMap, rc::Rc, time::SystemTime};

/// Represents the body of a Hinton native function object.
pub type NativeFn = fn(Vec<Object>) -> Result<Object, String>;

/// Represents the list of native functions available through a Hinton program.
pub struct NativeFunctions {
    pub functions_list: HashMap<String, NativeFunctionObj>,
    pub names: Vec<String>,
}

impl NativeFunctions {
    // Adds a native function definition to the native functions list.
    pub fn add_native_function(
        &mut self,
        name: &str,
        min_arity: u8,
        max_arity: u8,
        body: NativeFn,
    ) {
        let name = String::from(name);

        let f = NativeFunctionObj {
            name: name.clone(),
            min_arity,
            max_arity,
            function: body,
        };

        self.functions_list.insert(name.clone(), f);
        self.names.push(name);
    }

    /// Finds and executes a native function by name.
    pub fn call_native(&mut self, name: &str, args: Vec<Object>) -> Result<Object, String> {
        match self.functions_list.get(name) {
            Some(f) => {
                let args_len = args.len() as u8;

                // Checks the argument arity for the function call.
                if args_len < f.min_arity || args_len > f.max_arity {
                    if f.min_arity == f.max_arity {
                        return Err(format!(
                            "Expected {} arguments but got {} instead.",
                            f.min_arity, args_len
                        ));
                    } else {
                        return Err(format!(
                            "Expected {} to {} arguments but got {} instead.",
                            f.min_arity, f.max_arity, args_len
                        ));
                    }
                }

                // Calls the native function
                let call_result = (f.function)(args);

                // Returns the result of the call
                return call_result;
            }
            None => Err(format!("No native function named '{}'.", name)),
        }
    }

    /// Obtains the NativeFunctionObj associated with a native function name.
    pub fn get_native_fn_object(&self, name: &String) -> Result<NativeFunctionObj, String> {
        match self.functions_list.get(name) {
            Some(f) => Ok(f.clone().to_owned()),
            None => Err(format!("No native function named '{}'.", name)),
        }
    }
}

/// The default implementation of a native function list.
impl Default for NativeFunctions {
    fn default() -> Self {
        let mut natives = NativeFunctions {
            functions_list: Default::default(),
            names: Default::default(),
        };

        // >>>>>>>>>>>>>>>> Native functions to be added after this line
        natives.add_native_function("print", 1, 1, native_print as NativeFn);
        natives.add_native_function("clock", 0, 0, native_clock as NativeFn);
        natives.add_native_function("iter", 1, 1, native_iter as NativeFn);
        natives.add_native_function("next", 1, 1, native_next as NativeFn);
        // <<<<<<<<<<<<<<<< Native functions to be added before this line

        return natives;
    }
}

/// Implements the `print(...)` native function for Hinton,
/// which prints a value to the console.
fn native_print(args: Vec<Object>) -> Result<Object, String> {
    println!("{}", args[0]);
    Ok(Object::Null)
}

/// Implements the `clock()` native function for Hinton, which
/// retrieves the current time from the Unix Epoch time.
fn native_clock(_: Vec<Object>) -> Result<Object, String> {
    let now = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH);

    match now {
        Ok(t) => {
            let time = t.as_millis();
            Ok(Object::Int(time as i64))
        }
        Err(_) => Err(String::from("System's time before UNIX EPOCH.")),
    }
}

/// Implements the `iter(...)` native function for Hinton, which
/// converts the give object to an iterable.
fn native_iter(args: Vec<Object>) -> Result<Object, String> {
    let arg = args[0].clone();

    // Check that the object can be iterable.
    match arg {
        Object::String(_) => {}
        Object::Array(_) => {}
        Object::Range(_) => {}
        _ => {
            return Err(format!(
                "Cannot create iterable from object of type '{}'.",
                args[0].type_name()
            ))
        }
    };

    return Ok(Object::Iterable(Rc::new(RefCell::new(IterObject {
        iter: Box::new(arg),
        index: 0,
    }))));
}

/// Implements the `next(...)` native function for Hinton, which
/// retrieves the next item in an iterable object.
fn native_next(args: Vec<Object>) -> Result<Object, String> {
    match args[0].borrow() {
        Object::Iterable(iter) => {
            let mut iter = iter.borrow_mut();
            let current_index = Object::Int(iter.index as i64);

            // Since we are passing an integer into the `Object.get(...)` method,
            // the only error that can occur is an `IndexOutOfBounds` error, which
            // in terms of iterators means there are no more items left to iterate.
            let obj = match iter.iter.get(&current_index) {
                Ok(o) => o,
                Err(_) => return Err(String::from("End of Iterator.")),
            };

            // Increment to the next position of the iterator.
            iter.index += 1;

            return Ok(obj);
        }
        _ => Err(format!(
            "Object of type '{}' is not iterable.",
            args[0].type_name()
        )),
    }
}
