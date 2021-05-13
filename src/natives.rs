use crate::objects::{NativeFn, NativeFunctionObj, Object};
use std::{collections::HashMap, time::SystemTime};

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
