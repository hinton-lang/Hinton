use super::chunk;
use std::rc::Rc;
use std::result;
use std::{fmt, usize};

/// All types of objects in Hinton
pub enum Object<'a> {
    Number(f64),
    String(String),
    Bool(bool),
    Function(Rc<FunctionObject<'a>>),
    Array(Rc<Vec<Object<'a>>>),
    Null(),
}

impl<'a> Object<'a> {
    /// Gets the type name of an object.
    ///
    /// ## Returns
    /// `&str` – The string type name.
    pub fn type_name(&self) -> &str {
        return match self {
            &Object::Bool(_) => "Bool",
            &Object::Null() => "Null",
            &Object::Number(x) => {
                if x.fract() == 0.0 {
                    "Int"
                } else {
                    "Float"
                }
            }
            &Object::String(_) => "String",
            &Object::Array(_) => "Array",
            &Object::Function(_) => "Function",
        };
    }

    /// Checks that this is a Hinton numeric object.
    ///
    /// ## Returns
    /// `bool` – True if the object is a Hinton numeric, false otherwise.
    pub fn is_numeric(&self) -> bool {
        match self {
            Object::Number(_) => true,
            _ => false,
        }
    }

    /// Checks that this is a Hinton numeric object, and that the
    /// underlying number is an integer.
    ///
    /// ## Returns
    /// `bool` – True if the object is a Hinton integer, false otherwise.
    pub fn is_int(&self) -> bool {
        match self {
            Object::Number(x) => x.fract() == 0.0,
            _ => false,
        }
    }

    /// Checks that this is a Hinton numeric object, and that the
    /// underlying number is a float.
    ///
    /// ## Returns
    /// `bool` – True if the object is a Hinton float, false otherwise.
    pub fn is_float(&self) -> bool {
        match self {
            Object::Number(x) => x.fract() != 0.0,
            _ => false,
        }
    }

    /// Checks that this object is a Hinton boolean.
    ///
    /// ## Returns
    /// `bool` – True if the object is a Hinton boolean, false otherwise.
    pub fn is_bool(&self) -> bool {
        match self {
            Object::Bool(_) => true,
            _ => false,
        }
    }

    /// Checks that this object is a Hinton string.
    ///
    /// ## Returns
    /// `bool` – True if the object is a Hinton string, false otherwise.
    pub fn is_string(&self) -> bool {
        match self {
            Object::String(_) => true,
            _ => false,
        }
    }

    /// Checks that this object is a Hinton array.
    ///
    /// ## Returns
    /// `bool` – True if the object is a Hinton array, false otherwise.
    pub fn is_array(&self) -> bool {
        match self {
            Object::Array(_) => true,
            _ => false,
        }
    }

    /// Checks that this object is a Hinton function.
    ///
    /// ## Returns
    /// `bool` – True if the object is a Hinton function, false otherwise.
    pub fn is_function(&self) -> bool {
        match self {
            Object::Function(_) => true,
            _ => false,
        }
    }

    /// Checks that this object is a Hinton null.
    ///
    /// ## Returns
    /// `bool` – True if the object is a Hinton null, false otherwise.
    pub fn is_null(&self) -> bool {
        match self {
            Object::Null() => true,
            _ => false,
        }
    }

    /// Checks that the type of this object is falsey in Hinton
    ///
    /// ## Returns
    /// `bool` – True if the object is falsey, false otherwise.
    pub fn is_falsey(&self) -> bool {
        match self {
            Object::Null() => true,
            Object::Bool(val) => !val,
            _ => false,
        }
    }

    /// Converts the Hinton numeric object to a Rust float.
    ///
    /// ## Returns
    /// `Option<Rc<f64>>` – The underlying Rust float.
    pub fn as_number(&self) -> Option<f64> {
        match self {
            Object::Number(v) => Some(*v),
            _ => None,
        }
    }

    /// Converts the Hinton string object to a Rust string.
    ///
    /// ## Returns
    /// `Option<String>` – The underlying Rust string.
    pub fn as_string(&self) -> Option<String> {
        match self {
            Object::String(v) => Some(String::from(v.clone())),
            _ => None,
        }
    }

    /// Converts the Hinton bool object to Rust boolean.
    ///
    /// ## Returns
    /// `Option<bool>` – The underlying Rust boolean.
    pub fn as_bool(&self) -> Option<bool> {
        match self {
            Object::Bool(v) => Some(*v),
            _ => None,
        }
    }

    /// Checks that this object equals some other object based on Hinton's rules
    /// for object equality.
    ///
    /// ## Arguments
    /// * `b` – The object to be checked for Hinton equality against this object.
    ///
    /// ## Returns
    /// `bool` – True if the objects match based on Hinton rules for equality,
    /// false otherwise.
    pub fn equals(&self, b: Rc<Object<'a>>) -> bool {
        // If the operands differ in type, we can safely assume
        // they are not equal in value.
        if std::mem::discriminant(self) != std::mem::discriminant(&b) {
            return false;
        }

        // At this point, the operands have the same type, so we
        // proceed to check if they match in value.
        match self {
            Object::Bool(a) => *a == b.as_bool().unwrap(),
            Object::Null() => true,
            Object::Number(a) => *a == b.as_number().unwrap(),
            Object::String(a) => *a == b.as_string().unwrap(),
            // TODO: Implement array check
            _ => return false, // Unreachable.
        }
    }
}

/// Implements the `Display` trait so that objects can be printed
/// in a user-friendly way.
impl<'a> fmt::Display for Object<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> result::Result<(), fmt::Error> {
        match *self {
            Object::Number(ref inner) => fmt::Display::fmt(inner, f),
            Object::String(ref inner) => fmt::Display::fmt(inner, f),
            Object::Bool(ref inner) => write!(f, "{}", inner),
            Object::Array(ref inner) => {
                let mut arr_str = String::from("[");
                for obj in inner.iter() {
                    arr_str += &(format!("{}, ", obj))[..];
                }
                write!(f, "]")
            }
            Object::Function(ref inner) => write!(f, "<Func '{}'>", inner.name),
            Object::Null() => f.write_str("null"),
        }
    }
}

/// Represents a Hinton function object.
pub struct FunctionObject<'a> {
    pub min_arity: i32,
    pub max_arity: i32,
    pub chunk: chunk::Chunk<'a>,
    pub name: &'a str,
}

/// Represents a function call frame
pub struct CallFrame<'a> {
    /// The function chunk associated with this call frame
    pub function: Rc<FunctionObject<'a>>,
    // The instruction pointer for this call frame
    pub ip: usize,
    // TODO: What does this do?
    // pub slots: Vec<Object<'a>>
}
