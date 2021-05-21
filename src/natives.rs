use crate::objects::{IterObject, NativeFunctionObj, Object};
use std::{borrow::Borrow, cell::RefCell, collections::HashMap, io, rc::Rc, time::SystemTime};

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
            Some(f) => Ok(f.to_owned()),
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
        natives.add_native_function("clock", 0, 0, native_clock as NativeFn);
        natives.add_native_function("input", 1, 1, native_input as NativeFn);
        natives.add_native_function("iter", 1, 1, native_iter as NativeFn);
        natives.add_native_function("next", 1, 1, native_next as NativeFn);
        natives.add_native_function("print", 1, 1, native_print as NativeFn);
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
    make_iter(args[0].clone())
}

/// Converts a Hinton object into an Iterable.
pub fn make_iter(o: Object) -> Result<Object, String> {
    match o {
        Object::String(_) => {}
        Object::Array(_) => {}
        Object::Range(_) => {}
        Object::Tuple(_) => {}
        // If the object is already an iterable, return that same object.
        Object::Iterable(_) => return Ok(o),
        // Object cannot be iterable.
        _ => return Err(format!("Cannot create iterable from '{}'.", o.type_name())),
    };

    return Ok(Object::Iterable(Rc::new(RefCell::new(IterObject {
        iter: Box::new(o),
        index: 0,
    }))));
}

/// Implements the `next(...)` native function for Hinton, which
/// retrieves the next item in an iterable object.
fn native_next(args: Vec<Object>) -> Result<Object, String> {
    match args[0].borrow() {
        Object::Iterable(iter) => get_next_in_iter(iter),
        _ => Err(format!(
            "Object of type '{}' is not iterable.",
            args[0].type_name()
        )),
    }
}

/// Gets the next item in a Hinton iterator.
pub fn get_next_in_iter(o: &Rc<RefCell<IterObject>>) -> Result<Object, String> {
    let mut iter = o.borrow_mut();
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

/// Checks if a Hinton iterator has a next item.
pub fn iter_has_next(o: &Rc<RefCell<IterObject>>) -> bool {
    let o = o.borrow_mut();

    let len = match o.iter.borrow() {
        Object::String(ref x) => x.len(),
        Object::Array(ref x) => x.len(),
        Object::Tuple(ref x) => x.len(),
        Object::Range(ref x) => i64::abs(x.max - x.min) as usize,
        _ => unreachable!("Object is not iterable."),
    };

    o.index < len
}

/// Implements the `input(...)` native function for Hinton, which
/// gets user input from the console.
fn native_input(args: Vec<Object>) -> Result<Object, String> {
    print!("{}", args[0]);

    // Print the programmer-provided message
    match io::Write::flush(&mut io::stdout()) {
        Ok(_) => {
            let mut input = String::new();
            // Get the user's input
            match io::stdin().read_line(&mut input) {
                Ok(_) => {
                    input.pop(); // remove added newline
                    Ok(Object::String(input))
                }
                Err(e) => Err(format!("Failed to read input. IO failed read line. {}", e)),
            }
        }
        Err(e) => Err(format!("Failed to read input. IO failed to flush. {}", e)),
    }
}
