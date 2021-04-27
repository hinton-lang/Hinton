use crate::chunk;
use std::fmt;
use std::rc::Rc;
use std::result;

/// All types of objects in Hinton
pub enum Object {
    Number(f64),
    String(String),
    Bool(bool),
    Function(Rc<FunctionObject>),
    Array(Vec<Rc<Object>>),
    Range(Rc<RangeObject>),
    Null,
}

impl Object {
    /// Gets the type name of an object.
    ///
    /// ## Returns
    /// `&str` – The string type name.
    pub fn type_name(&self) -> &str {
        return match self {
            &Object::Bool(_) => "Bool",
            &Object::Null => "Null",
            &Object::Number(x) => {
                if x.fract() == 0.0 {
                    "Int"
                } else {
                    "Float"
                }
            }
            &Object::String(_) => "String",
            &Object::Array(_) => "Array",
            &Object::Range(_) => "Range",
            &Object::Function(_) => "Function",
        };
    }

    /// Checks that this is a Hinton numeric object.
    ///
    /// ## Returns
    /// `bool` – True if the object is a Hinton numeric, false otherwise.
    pub fn is_numeric(&self) -> bool {
        match self {
            Object::Number(_) | Object::Bool(_) => true,
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
            Object::Bool(_) => true,
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

    /// Checks that this object can be converted to a Hinton string.
    ///
    /// ## Returns
    /// `bool` – True if the object can be converted to a Hinton string, false otherwise.
    pub fn is_stringifyable(&self) -> bool {
        match self {
            Object::String(_) | Object::Number(_) => true,
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
            Object::Null => true,
            _ => false,
        }
    }

    /// Checks that the type of this object is falsey in Hinton
    ///
    /// ## Returns
    /// `bool` – True if the object is falsey, false otherwise.
    pub fn is_falsey(&self) -> bool {
        match self {
            Object::Null => true,
            Object::Bool(val) => !val,
            Object::Number(x) => {
                if *x == 0.0 {
                    true
                } else {
                    false
                }
            }
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
            Object::Bool(b) => {
                if *b {
                    Some(1f64)
                } else {
                    Some(0f64)
                }
            }
            _ => None,
        }
    }

    /// Converts the Hinton string object to a Rust string.
    ///
    /// ## Returns
    /// `Option<String>` – The underlying Rust string.
    pub fn as_string(&self) -> Option<String> {
        match self {
            Object::String(s) => Some(String::from(s)),
            Object::Number(n) => Some(n.to_string()),
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

    /// Obtains the wrapped Hinton range from this object.
    ///
    /// ## Returns
    /// `Option<Rc<RangeObject>>` – The underlying RangeObject.
    pub fn as_range(&self) -> Option<Rc<RangeObject>> {
        match self {
            Object::Range(v) => Some(Rc::clone(v)),
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
    pub fn equals(&self, b: Rc<Object>) -> bool {
        // If the operands differ in type, we can safely assume
        // they are not equal in value.
        if std::mem::discriminant(self) != std::mem::discriminant(&b) {
            return false;
        }

        // At this point, the operands have the same type, so we
        // proceed to check if they match in value.
        return match self {
            Object::Bool(a) => (*a == b.as_bool().unwrap()),
            Object::Null => true,
            Object::Number(a) => (*a == b.as_number().unwrap()),
            Object::String(a) => (*a == b.as_string().unwrap()),
            _ => false,
        };
    }
}

/// Implements the `Display` trait so that objects can be printed
/// in a user-friendly way.
impl<'a> fmt::Display for Object {
    fn fmt(&self, f: &mut fmt::Formatter) -> result::Result<(), fmt::Error> {
        match *self {
            Object::Number(ref inner) => {
                let str = String::from("\x1b[38;5;81m") + inner.to_string().as_str() + String::from("\x1b[0m").as_str();
                fmt::Display::fmt(&str, f)
            }
            Object::String(ref inner) => {
                let inner = format!("\"{}\"", inner);
                fmt::Display::fmt(&inner, f)
            },
            Object::Bool(ref inner) => {
                let str = if *inner {
                    String::from("\x1b[38;5;3mtrue\x1b[0m")
                } else {
                    String::from("\x1b[38;5;3mfalse\x1b[0m")
                };

                fmt::Display::fmt(&str, f)
            }
            Object::Array(ref inner) => {
                let mut arr_str = String::from("[");
                for (idx, obj) in inner.iter().enumerate() {
                    if idx == inner.len() - 1 {
                        arr_str += &(format!("{}", obj))[..]
                    } else {
                        arr_str += &(format!("{}, ", obj))[..];
                    }
                }
                arr_str += "]";

                write!(f, "{}", arr_str)
            }
            Object::Range(ref inner) => write!(f, "[\x1b[38;5;81m{}\x1b[0m..\x1b[38;5;81m{}\x1b[0m]", inner.min, inner.max),
            Object::Function(ref inner) => write!(f, "<Func '{}'>", inner.name),
            Object::Null => f.write_str("\x1b[37;1mnull\x1b[0m"),
        }
    }
}

/// Represents a Hinton function object.
pub struct FunctionObject {
    pub min_arity: i32,
    pub max_arity: i32,
    pub chunk: chunk::Chunk,
    pub name: String,
}

/// Represents a Hinton range object.
pub struct RangeObject {
    pub min: isize,
    pub max: isize,
    pub step: isize,
}
