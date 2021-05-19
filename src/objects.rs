use crate::chunk::Chunk;
use std::fmt;
use std::rc::Rc;
use std::result;

/// All types of objects in Hinton
#[derive(Clone)]
pub enum Object {
    Int(i64),
    Float(f64),
    String(String),
    Bool(bool),
    Function(FunctionObject),
    NativeFunction(NativeFunctionObj),
    Array(Vec<Object>),
    Range(Rc<RangeObject>),
    Null,
}

/// Represents a Hinton range object.
pub struct RangeObject {
    pub min: i64,
    pub max: i64,
}

/// Represents a Hinton function object.
#[derive(Clone)]
pub struct FunctionObject {
    pub defaults: Vec<Object>,
    pub min_arity: u8,
    pub max_arity: u8,
    pub chunk: Chunk,
    pub name: String,
}

/// Represents a Hinton native function object.
#[derive(Clone)]
pub struct NativeFunctionObj {
    pub name: String,
    pub min_arity: u8,
    pub max_arity: u8,
    pub function: NativeFn,
}

/// Represents the body of a Hinton native function object.
pub type NativeFn = fn(Vec<Object>) -> Result<Object, String>;

impl Object {
    pub fn type_name(&self) -> &str {
        return match self {
            &Object::Bool(_) => "Bool",
            &Object::Null => "Null",
            &Object::Int(_) => "Int",
            &Object::Float(_) => "Float",
            &Object::String(_) => "String",
            &Object::Array(_) => "Array",
            &Object::Range(_) => "Range",
            &Object::Function(_) | Object::NativeFunction(_) => "Function",
        };
    }

    pub fn is_numeric(&self) -> bool {
        match self {
            Object::Int(_) | Object::Float(_) | Object::Bool(_) => true,
            _ => false,
        }
    }

    pub fn is_int(&self) -> bool {
        match self {
            Object::Int(_) => true,
            _ => false,
        }
    }

    pub fn is_float(&self) -> bool {
        match self {
            Object::Float(_) => true,
            _ => false,
        }
    }

    pub fn is_bool(&self) -> bool {
        match self {
            Object::Bool(_) => true,
            _ => false,
        }
    }

    pub fn is_string(&self) -> bool {
        match self {
            Object::String(_) => true,
            _ => false,
        }
    }

    pub fn is_stringifyable(&self) -> bool {
        match self {
            Object::String(_) | Object::Int(_) | Object::Float(_) => true,
            _ => false,
        }
    }

    pub fn is_array(&self) -> bool {
        match self {
            Object::Array(_) => true,
            _ => false,
        }
    }

    pub fn is_function(&self) -> bool {
        match self {
            Object::Function(_) => true,
            _ => false,
        }
    }

    pub fn is_null(&self) -> bool {
        match self {
            Object::Null => true,
            _ => false,
        }
    }

    pub fn is_falsey(&self) -> bool {
        match self {
            Object::Null => true,
            Object::Bool(val) => !val,
            Object::Int(x) if *x == 0i64 => true,
            Object::Float(x) if *x == 0f64 => true,
            _ => false,
        }
    }

    pub fn as_int(&self) -> Option<i64> {
        match self {
            Object::Int(v) => Some(*v),
            Object::Bool(b) => {
                if *b {
                    Some(1i64)
                } else {
                    Some(0i64)
                }
            }
            _ => None,
        }
    }

    pub fn as_float(&self) -> Option<f64> {
        match self {
            Object::Float(v) => Some(*v),
            _ => None,
        }
    }

    pub fn as_string(&self) -> Option<String> {
        match self {
            Object::String(s) => Some(String::from(s)),
            Object::Int(n) => Some(n.to_string()),
            Object::Float(n) => Some(n.to_string()),
            _ => None,
        }
    }

    pub fn as_bool(&self) -> Option<bool> {
        match self {
            Object::Bool(v) => Some(*v),
            _ => None,
        }
    }

    pub fn as_range(&self) -> Option<Rc<RangeObject>> {
        match self {
            Object::Range(v) => Some(Rc::clone(v)),
            _ => None,
        }
    }

    pub fn as_array(&self) -> Option<&Vec<Object>> {
        match self {
            Object::Array(v) => Some(v),
            _ => None,
        }
    }

    pub fn as_function(&self) -> Option<&FunctionObject> {
        match self {
            Object::Function(f) => Some(f),
            _ => None,
        }
    }

    /// Defines equality for Hinton objects.
    pub fn equals(&self, right: &Object) -> bool {
        // Equality check for numeric types
        match self {
            Object::Int(i) => {
                return match right {
                    Object::Int(x) if i == x => true,
                    Object::Float(x) if (x - i.clone() as f64) == 0f64 => true,
                    Object::Bool(x) if (i == &0i64 && !x) || (i == &1i64 && *x) => true,
                    _ => false,
                }
            }
            Object::Float(f) => {
                return match right {
                    Object::Int(x) if (f - x.clone() as f64) == 0f64 => true,
                    Object::Float(x) if f == x => true,
                    Object::Bool(x) if (f == &0f64 && !x) || (f == &1f64 && *x) => true,
                    _ => false,
                }
            }
            Object::Bool(b) => {
                return match right {
                    Object::Int(x) if (x == &0i64 && !b) || (x == &1i64 && *b) => true,
                    Object::Float(x) if (x == &0f64 && !b) || (x == &1f64 && *b) => true,
                    Object::Bool(x) => !(b ^ x),
                    _ => false,
                }
            }
            _ => {}
        }

        // If the operands differ in type, we can safely assume
        // they are not equal in value.
        if std::mem::discriminant(self) != std::mem::discriminant(&right) {
            return false;
        }

        // At this point, the operands have the same type, so we
        // proceed to check if they match in value.
        return match self {
            Object::String(a) => (*a == right.as_string().unwrap()),
            Object::Array(a) => {
                let b = right.as_array().unwrap();
                // If arrays differ in size, they must differ in value. However, if they
                // are equal in size, then we must check that each item match.
                if a.len() != b.len() {
                    false
                } else {
                    for i in 0..a.len() {
                        let a_ith = &a[i];
                        let b_ith = &b[i];

                        // If at least one of the items differ in value,
                        // then the arrays are not equals.
                        if !a_ith.equals(&b_ith) {
                            return false;
                        }
                    }

                    true
                }
            }
            Object::Range(a) => {
                let b = right.as_range().unwrap();

                // If the ranges match in boundaries,
                // then they are equal in value.
                a.min == b.min && a.max == b.max
            }
            _ => false,
        };
    }
}

/// Implements the `Display` trait so that objects can be printed in a console-friendly way.
impl<'a> fmt::Display for Object {
    fn fmt(&self, f: &mut fmt::Formatter) -> result::Result<(), fmt::Error> {
        match *self {
            Object::Int(ref inner) => {
                let str = String::from("\x1b[38;5;81m")
                    + inner.to_string().as_str()
                    + String::from("\x1b[0m").as_str();
                fmt::Display::fmt(&str, f)
            }
            Object::Float(ref inner) => {
                let str = String::from("\x1b[38;5;81m")
                    + inner.to_string().as_str()
                    + if inner.fract() == 0.0 { ".0" } else { "" } // display the .0
                    + String::from("\x1b[0m").as_str();
                fmt::Display::fmt(&str, f)
            }
            Object::String(ref inner) => {
                let inner = format!("\"{}\"", inner);
                fmt::Display::fmt(&inner, f)
            }
            Object::Bool(inner) => {
                let str = if inner {
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
            Object::Range(ref inner) => write!(
                f,
                "[\x1b[38;5;81m{}\x1b[0m..\x1b[38;5;81m{}\x1b[0m]",
                inner.min, inner.max
            ),
            Object::Function(ref inner) => {
                let str = if inner.name == "" {
                    String::from("<script>")
                } else {
                    format!("<Func '{}'>", inner.name)
                };

                fmt::Display::fmt(&str, f)
            }
            Object::NativeFunction(ref inner) => {
                let str = format!("<NativeFn '{}'>", inner.name);
                fmt::Display::fmt(&str, f)
            }
            Object::Null => f.write_str("\x1b[37;1mnull\x1b[0m"),
        }
    }
}

/// Defines negation of Hinton objects.
impl std::ops::Neg for Object {
    type Output = Result<Object, String>;

    fn neg(self) -> Self::Output {
        match self {
            Object::Int(lhs) => Ok(Object::Int(-lhs)),
            Object::Float(lhs) => Ok(Object::Float(-lhs)),
            Object::Bool(lhs) if lhs => Ok(Object::Int(-1)),
            Object::Bool(lhs) if !lhs => Ok(Object::Int(0)),
            _ => {
                return Err(format!(
                    "Cannot negate operand of type '{}'.",
                    self.type_name()
                ))
            }
        }
    }
}

/// Defines addition of Hinton objects.
impl std::ops::Add<Object> for Object {
    type Output = Result<Object, String>;

    fn add(self, rhs: Object) -> Self::Output {
        let error_msg = Err(format!(
            "Operation '+' not defined for objects of type '{}' and '{}'.",
            self.type_name(),
            rhs.type_name()
        ));

        match self {
            Object::Int(lhs) => match rhs {
                Object::Int(rhs) => Ok(Object::Int(lhs + rhs)),
                Object::Float(rhs) => Ok(Object::Float(lhs as f64 + rhs)),
                Object::Bool(rhs) => Ok(Object::Int(lhs + if rhs { 1 } else { 0 })),
                Object::String(rhs) => Ok(Object::String(format!("{}{}", lhs, rhs))),
                _ => return error_msg,
            },
            Object::Float(lhs) => match rhs {
                Object::Int(rhs) => Ok(Object::Float(lhs + rhs as f64)),
                Object::Float(rhs) => Ok(Object::Float(lhs + rhs)),
                Object::Bool(rhs) => Ok(Object::Float(lhs + if rhs { 1f64 } else { 0f64 })),
                Object::String(rhs) => Ok(Object::String(format!(
                    "{}{}{}",
                    lhs,
                    if lhs.fract() == 0.0 { ".0" } else { "" },
                    rhs
                ))),
                _ => return error_msg,
            },
            Object::String(lhs) => match rhs {
                Object::Int(rhs) => Ok(Object::String(format!("{}{}", lhs, rhs))),
                Object::Float(rhs) => Ok(Object::String(format!(
                    "{}{}{}",
                    lhs,
                    rhs,
                    if rhs.fract() == 0.0 { ".0" } else { "" }
                ))),
                Object::String(rhs) => Ok(Object::String(format!("{}{}", lhs, rhs))),
                _ => return error_msg,
            },
            Object::Bool(lhs) if lhs => match rhs {
                Object::Int(rhs) => Ok(Object::Int(rhs + 1i64)),
                Object::Float(rhs) => Ok(Object::Float(rhs + 1f64)),
                Object::Bool(rhs) => Ok(Object::Int(1 + if rhs { 1 } else { 0 })),
                _ => return error_msg,
            },
            Object::Bool(lhs) if !lhs => match rhs {
                Object::Int(rhs) => Ok(Object::Int(rhs)),
                Object::Float(rhs) => Ok(Object::Float(rhs)),
                Object::Bool(rhs) => Ok(Object::Int(if rhs { 1 } else { 0 })),
                _ => return error_msg,
            },
            _ => return error_msg,
        }
    }
}

/// Defines subtraction of Hinton objects.
impl std::ops::Sub<Object> for Object {
    type Output = Result<Object, String>;

    fn sub(self, rhs: Object) -> Self::Output {
        let error_msg = Err(format!(
            "Operation '-' not defined for objects of type '{}' and '{}'.",
            self.type_name(),
            rhs.type_name()
        ));

        match self {
            Object::Int(lhs) => match rhs {
                Object::Int(rhs) => Ok(Object::Int(lhs - rhs)),
                Object::Float(rhs) => Ok(Object::Float(lhs as f64 - rhs)),
                Object::Bool(rhs) => Ok(Object::Int(lhs - if rhs { 1 } else { 0 })),
                _ => return error_msg,
            },
            Object::Float(lhs) => match rhs {
                Object::Int(rhs) => Ok(Object::Float(lhs - rhs as f64)),
                Object::Float(rhs) => Ok(Object::Float(lhs - rhs)),
                Object::Bool(rhs) => Ok(Object::Float(lhs - if rhs { 1f64 } else { 0f64 })),
                _ => return error_msg,
            },
            Object::Bool(lhs) if lhs => match rhs {
                Object::Int(rhs) => Ok(Object::Int(1i64 - rhs)),
                Object::Float(rhs) => Ok(Object::Float(1f64 - rhs)),
                Object::Bool(rhs) => Ok(Object::Int(1 - if rhs { 1 } else { 0 })),
                _ => return error_msg,
            },
            Object::Bool(lhs) if !lhs => match rhs {
                Object::Int(rhs) => Ok(Object::Int(-rhs)),
                Object::Float(rhs) => Ok(Object::Float(-rhs)),
                Object::Bool(rhs) => Ok(Object::Int(if rhs { -1 } else { 0 })),
                _ => return error_msg,
            },
            _ => return error_msg,
        }
    }
}

/// Defines multiplication of Hinton objects.
impl std::ops::Mul<Object> for Object {
    type Output = Result<Object, String>;

    fn mul(self, rhs: Object) -> Self::Output {
        let error_msg = Err(format!(
            "Operation '*' not defined for objects of type '{}' and '{}'.",
            self.type_name(),
            rhs.type_name()
        ));

        match self {
            Object::Int(lhs) => match rhs {
                Object::Int(rhs) => Ok(Object::Int(lhs.saturating_mul(rhs))),
                Object::Float(rhs) => Ok(Object::Float(lhs as f64 * rhs)),
                Object::Bool(rhs) => Ok(Object::Int(if rhs { lhs } else { 0 })),
                Object::String(rhs) => Ok(Object::String(rhs.repeat(lhs as usize))),
                _ => return error_msg,
            },
            Object::Float(lhs) => match rhs {
                Object::Int(rhs) => Ok(Object::Float(lhs * rhs as f64)),
                Object::Float(rhs) => Ok(Object::Float(lhs * rhs)),
                Object::Bool(rhs) => Ok(Object::Float(if rhs { lhs } else { 0f64 })),
                _ => return error_msg,
            },
            Object::String(lhs) => match rhs {
                Object::Int(rhs) => Ok(Object::String(lhs.repeat(rhs as usize))),
                _ => return error_msg,
            },
            Object::Bool(lhs) if lhs => match rhs {
                Object::Int(rhs) => Ok(Object::Int(rhs)),
                Object::Float(rhs) => Ok(Object::Float(rhs)),
                Object::Bool(rhs) => Ok(Object::Int(if rhs { 1 } else { 0 })),
                _ => return error_msg,
            },
            Object::Bool(lhs) if !lhs => match rhs {
                Object::Int(_) => Ok(Object::Int(0)),
                Object::Float(_) => Ok(Object::Float(0f64)),
                Object::Bool(_) => Ok(Object::Int(0)),
                _ => return error_msg,
            },
            _ => return error_msg,
        }
    }
}

/// Defines division of Hinton objects.
impl std::ops::Div<Object> for Object {
    type Output = Result<Object, String>;

    fn div(self, rhs: Object) -> Self::Output {
        let error_msg = Err(format!(
            "Operation '/' not defined for objects of type '{}' and '{}'.",
            self.type_name(),
            rhs.type_name()
        ));

        // Divide-by-zero errors
        if rhs.is_int() && rhs.as_int().unwrap() == 0
            || rhs.is_float() && rhs.as_float().unwrap() == 0f64
            || rhs.is_bool() && !rhs.as_bool().unwrap()
        {
            return Err(String::from("Cannot divide by zero."));
        }

        match self {
            // TODO: Is converting from i64 to f64 a lossy conversion?
            Object::Int(lhs) => match rhs {
                Object::Int(rhs) => Ok(Object::Float(lhs as f64 / rhs as f64)),
                Object::Float(rhs) => Ok(Object::Float(lhs as f64 / rhs)),
                Object::Bool(_) => Ok(Object::Float(lhs as f64)),
                _ => return error_msg,
            },
            Object::Float(lhs) => match rhs {
                Object::Int(rhs) => Ok(Object::Float(lhs / rhs as f64)),
                Object::Float(rhs) => Ok(Object::Float(lhs / rhs)),
                Object::Bool(_) => Ok(Object::Float(lhs as f64)),
                _ => return error_msg,
            },
            Object::Bool(lhs) if lhs => match rhs {
                Object::Int(rhs) => Ok(Object::Float(1f64 / rhs as f64)),
                Object::Float(rhs) => Ok(Object::Float(1f64 / rhs)),
                Object::Bool(_) => Ok(Object::Float(1f64)),
                _ => return error_msg,
            },
            Object::Bool(lhs) if !lhs => match rhs {
                Object::Int(_) => Ok(Object::Float(0f64)),
                Object::Float(_) => Ok(Object::Float(0f64)),
                Object::Bool(_) => Ok(Object::Float(0f64)),
                _ => return error_msg,
            },
            _ => return error_msg,
        }
    }
}

/// Defines modulo of Hinton objects.
impl std::ops::Rem<Object> for Object {
    type Output = Result<Object, String>;

    fn rem(self, rhs: Object) -> Self::Output {
        let error_msg = Err(format!(
            "Operation '%' not defined for objects of type '{}' and '{}'.",
            self.type_name(),
            rhs.type_name()
        ));

        // zero-modulo errors
        if rhs.is_int() && rhs.as_int().unwrap() == 0
            || rhs.is_float() && rhs.as_float().unwrap() == 0f64
            || rhs.is_bool() && !rhs.as_bool().unwrap()
        {
            return Err(String::from("Right-hand-side of modulus cannot be zero."));
        }

        match self {
            // TODO: Is converting from f64 to i64 a lossy conversion?
            Object::Int(lhs) => match rhs {
                Object::Int(rhs) => Ok(Object::Int(lhs % rhs)),
                Object::Float(rhs) => Ok(Object::Int(lhs % rhs.floor() as i64)),
                Object::Bool(_) => Ok(Object::Int(0i64)),
                _ => return error_msg,
            },
            Object::Float(lhs) => match rhs {
                Object::Int(rhs) => Ok(Object::Float(lhs % rhs as f64)),
                Object::Float(rhs) => Ok(Object::Float(lhs % rhs)),
                Object::Bool(_) => Ok(Object::Float(lhs % 1f64)),
                _ => return error_msg,
            },
            Object::Bool(lhs) if lhs => match rhs {
                Object::Int(rhs) => Ok(Object::Int(1i64 % rhs)),
                Object::Float(rhs) => Ok(Object::Float(1f64 % rhs)),
                Object::Bool(_) => Ok(Object::Int(0i64)),
                _ => return error_msg,
            },
            Object::Bool(lhs) if !lhs => match rhs {
                Object::Int(_) => Ok(Object::Int(0i64)),
                Object::Float(_) => Ok(Object::Float(0f64)),
                Object::Bool(_) => Ok(Object::Int(0i64)),
                _ => return error_msg,
            },
            _ => return error_msg,
        }
    }
}

/// Defines the bitwise-and operation of Hinton objects.
impl std::ops::BitAnd<Object> for Object {
    type Output = Result<Object, String>;

    fn bitand(self, rhs: Object) -> Self::Output {
        let error_msg = Err(format!(
            "Operation '&' not defined for objects of type '{}' and '{}'.",
            self.type_name(),
            rhs.type_name()
        ));

        match self {
            Object::Int(lhs) => match rhs {
                Object::Int(rhs) => Ok(Object::Int(lhs & rhs)),
                Object::Bool(rhs) => Ok(Object::Int(lhs & if rhs { 1 } else { 0 })),
                _ => return error_msg,
            },
            Object::Bool(lhs) if lhs => match rhs {
                Object::Int(rhs) => Ok(Object::Int(if lhs { 1 } else { 0 } & rhs)),
                Object::Bool(rhs) => Ok(Object::Int(
                    if lhs { 1 } else { 0 } & if rhs { 1 } else { 0 },
                )),
                _ => return error_msg,
            },
            _ => return error_msg,
        }
    }
}

/// Defines the bitwise-or operation of Hinton objects.
impl std::ops::BitOr<Object> for Object {
    type Output = Result<Object, String>;

    fn bitor(self, rhs: Object) -> Self::Output {
        let error_msg = Err(format!(
            "Operation '|' not defined for objects of type '{}' and '{}'.",
            self.type_name(),
            rhs.type_name()
        ));

        match self {
            Object::Int(lhs) => match rhs {
                Object::Int(rhs) => Ok(Object::Int(lhs | rhs)),
                Object::Bool(rhs) => Ok(Object::Int(lhs | if rhs { 1 } else { 0 })),
                _ => return error_msg,
            },
            Object::Bool(lhs) if lhs => match rhs {
                Object::Int(rhs) => Ok(Object::Int(if lhs { 1 } else { 0 } | rhs)),
                Object::Bool(rhs) => Ok(Object::Int(
                    if lhs { 1 } else { 0 } | if rhs { 1 } else { 0 },
                )),
                _ => return error_msg,
            },
            _ => return error_msg,
        }
    }
}

/// Defines the bitwise-xor operation of Hinton objects.
impl std::ops::BitXor<Object> for Object {
    type Output = Result<Object, String>;

    fn bitxor(self, rhs: Object) -> Self::Output {
        let error_msg = Err(format!(
            "Operation '^' not defined for objects of type '{}' and '{}'.",
            self.type_name(),
            rhs.type_name()
        ));

        match self {
            Object::Int(lhs) => match rhs {
                Object::Int(rhs) => Ok(Object::Int(lhs ^ rhs)),
                Object::Bool(rhs) => Ok(Object::Int(lhs ^ if rhs { 1 } else { 0 })),
                _ => return error_msg,
            },
            Object::Bool(lhs) if lhs => match rhs {
                Object::Int(rhs) => Ok(Object::Int(if lhs { 1 } else { 0 } ^ rhs)),
                Object::Bool(rhs) => Ok(Object::Int(
                    if lhs { 1 } else { 0 } ^ if rhs { 1 } else { 0 },
                )),
                _ => return error_msg,
            },
            _ => return error_msg,
        }
    }
}

/// Defines the bitwise-shift-left operation of Hinton objects.
impl std::ops::Shl<Object> for Object {
    type Output = Result<Object, String>;

    fn shl(self, rhs: Object) -> Self::Output {
        let error_msg = Err(format!(
            "Operation '<<' not defined for objects of type '{}' and '{}'.",
            self.type_name(),
            rhs.type_name()
        ));

        match self {
            Object::Int(lhs) => match rhs {
                Object::Int(rhs) => Ok(Object::Int(lhs << rhs)),
                Object::Bool(rhs) => Ok(Object::Int(lhs << if rhs { 1 } else { 0 })),
                _ => return error_msg,
            },
            Object::Bool(lhs) if lhs => match rhs {
                Object::Int(rhs) => Ok(Object::Int(if lhs { 1 } else { 0 } << rhs)),
                Object::Bool(rhs) => Ok(Object::Int(
                    if lhs { 1 } else { 0 } << if rhs { 1 } else { 0 },
                )),
                _ => return error_msg,
            },
            _ => return error_msg,
        }
    }
}

/// Defines the bitwise-shift-right operation of Hinton objects.
impl std::ops::Shr<Object> for Object {
    type Output = Result<Object, String>;

    fn shr(self, rhs: Object) -> Self::Output {
        let error_msg = Err(format!(
            "Operation '>>' not defined for objects of type '{}' and '{}'.",
            self.type_name(),
            rhs.type_name()
        ));

        match self {
            Object::Int(lhs) => match rhs {
                Object::Int(rhs) => Ok(Object::Int(lhs >> rhs)),
                Object::Bool(rhs) => Ok(Object::Int(lhs >> if rhs { 1 } else { 0 })),
                _ => return error_msg,
            },
            Object::Bool(lhs) if lhs => match rhs {
                Object::Int(rhs) => Ok(Object::Int(if lhs { 1 } else { 0 } >> rhs)),
                Object::Bool(rhs) => Ok(Object::Int(
                    if lhs { 1 } else { 0 } >> if rhs { 1 } else { 0 },
                )),
                _ => return error_msg,
            },
            _ => return error_msg,
        }
    }
}

/// Defines the bitwise-not operation of Hinton objects.
/// NOTE: Rust does not have a special bitwise-not (~)
/// operator, instead Rust uses the '!' for both logical-not
/// and bitwise-not. Using the '!' operator on a Hinton
/// object only applies the bitwise-not operation. For the
/// logic-not operation use the `Object.is_falsey()` method.
impl std::ops::Not for Object {
    type Output = Result<Object, String>;

    fn not(self) -> Self::Output {
        match self {
            Object::Int(opr) => Ok(Object::Int(!opr)),
            Object::Bool(opr) => Ok(Object::Int(!(opr as i64))),
            _ => {
                return Err(format!(
                    "Operation '~' not defined for object of type '{}'.",
                    self.type_name()
                ))
            }
        }
    }
}

impl Object {
    /// Defines exponentiation of Hinton objects.
    pub fn pow(self, rhs: Object) -> Result<Object, String> {
        let error_msg = Err(format!(
            "Operation '**' not defined for objects of type '{}' and '{}'.",
            self.type_name(),
            rhs.type_name()
        ));

        match self {
            // TODO: These conversions seem error-prone...
            Object::Int(lhs) => match rhs {
                Object::Int(rhs) => Ok(Object::Int((lhs as f64).powf(rhs as f64) as i64)),
                Object::Float(rhs) => Ok(Object::Float((lhs as f64).powf(rhs))),
                Object::Bool(rhs) if rhs => Ok(Object::Int(lhs)),
                Object::Bool(rhs) if !rhs => Ok(Object::Int(1)),
                _ => return error_msg,
            },
            Object::Float(lhs) => match rhs {
                Object::Int(rhs) => Ok(Object::Float(lhs.powf(rhs as f64))),
                Object::Float(rhs) => Ok(Object::Float(lhs.powf(rhs))),
                Object::Bool(rhs) if rhs => Ok(Object::Float(lhs)),
                Object::Bool(rhs) if !rhs => Ok(Object::Float(1f64)),
                _ => return error_msg,
            },
            Object::Bool(lhs) if lhs => match rhs {
                Object::Int(_) => Ok(Object::Int(1i64)),
                Object::Float(_) => Ok(Object::Float(1f64)),
                Object::Bool(_) => Ok(Object::Int(1i64)),
                _ => return error_msg,
            },
            Object::Bool(lhs) if !lhs => match rhs {
                Object::Int(rhs) => Ok(Object::Int(0f64.powf(rhs as f64) as i64)),
                Object::Float(rhs) => Ok(Object::Float(0f64.powf(rhs))),
                Object::Bool(rhs) if rhs => Ok(Object::Int(0i64)),
                Object::Bool(rhs) if !rhs => Ok(Object::Int(1i64)),
                _ => return error_msg,
            },
            _ => return error_msg,
        }
    }

    /// Defines the greater-than operation of Hinton objects.
    pub fn gt(self, rhs: Object) -> Result<Object, String> {
        let error_msg = Err(format!(
            "Operation '>' not defined for objects of type '{}' and '{}'.",
            self.type_name(),
            rhs.type_name()
        ));

        match self {
            Object::Int(lhs) => match rhs {
                Object::Int(rhs) => Ok(Object::Bool(lhs > rhs)),
                Object::Float(rhs) => Ok(Object::Bool((lhs as f64) > rhs)),
                Object::Bool(rhs) if rhs => Ok(Object::Bool(lhs > if rhs { 1 } else { 0 })),
                _ => return error_msg,
            },
            Object::Float(lhs) => match rhs {
                Object::Int(rhs) => Ok(Object::Bool(lhs > rhs as f64)),
                Object::Float(rhs) => Ok(Object::Bool(lhs > rhs)),
                Object::Bool(rhs) if rhs => Ok(Object::Bool(lhs > if rhs { 1f64 } else { 0f64 })),
                _ => return error_msg,
            },
            Object::Bool(lhs) => match rhs {
                Object::Int(rhs) => Ok(Object::Bool(if lhs { 1 } else { 0 } > rhs)),
                Object::Float(rhs) => Ok(Object::Bool(if lhs { 1f64 } else { 0f64 } > rhs)),
                Object::Bool(rhs) => Ok(Object::Bool(if lhs { 1 } else { 0 } > rhs as i64)),
                _ => return error_msg,
            },
            _ => return error_msg,
        }
    }

    /// Defines the greater-than-equals operation of Hinton objects.
    pub fn gteq(self, rhs: Object) -> Result<Object, String> {
        let error_msg = Err(format!(
            "Operation '>=' not defined for objects of type '{}' and '{}'.",
            self.type_name(),
            rhs.type_name()
        ));

        match self {
            Object::Int(lhs) => match rhs {
                Object::Int(rhs) => Ok(Object::Bool(lhs >= rhs)),
                Object::Float(rhs) => Ok(Object::Bool((lhs as f64) >= rhs)),
                Object::Bool(rhs) if rhs => Ok(Object::Bool(lhs >= if rhs { 1 } else { 0 })),
                _ => return error_msg,
            },
            Object::Float(lhs) => match rhs {
                Object::Int(rhs) => Ok(Object::Bool(lhs >= rhs as f64)),
                Object::Float(rhs) => Ok(Object::Bool(lhs >= rhs)),
                Object::Bool(rhs) if rhs => Ok(Object::Bool(lhs >= if rhs { 1f64 } else { 0f64 })),
                _ => return error_msg,
            },
            Object::Bool(lhs) => match rhs {
                Object::Int(rhs) => Ok(Object::Bool(if lhs { 1 } else { 0 } >= rhs)),
                Object::Float(rhs) => Ok(Object::Bool(if lhs { 1f64 } else { 0f64 } >= rhs)),
                Object::Bool(rhs) => Ok(Object::Bool(if lhs { 1 } else { 0 } >= rhs as i64)),
                _ => return error_msg,
            },
            _ => return error_msg,
        }
    }

    /// Defines the less-than operation of Hinton objects.
    pub fn lt(self, rhs: Object) -> Result<Object, String> {
        let error_msg = Err(format!(
            "Operation '<' not defined for objects of type '{}' and '{}'.",
            self.type_name(),
            rhs.type_name()
        ));

        match self {
            Object::Int(lhs) => match rhs {
                Object::Int(rhs) => Ok(Object::Bool(lhs < rhs)),
                Object::Float(rhs) => Ok(Object::Bool((lhs as f64) < rhs)),
                Object::Bool(rhs) if rhs => Ok(Object::Bool(lhs < if rhs { 1 } else { 0 })),
                _ => return error_msg,
            },
            Object::Float(lhs) => match rhs {
                Object::Int(rhs) => Ok(Object::Bool(lhs < rhs as f64)),
                Object::Float(rhs) => Ok(Object::Bool(lhs < rhs)),
                Object::Bool(rhs) if rhs => Ok(Object::Bool(lhs < if rhs { 1f64 } else { 0f64 })),
                _ => return error_msg,
            },
            Object::Bool(lhs) => match rhs {
                Object::Int(rhs) => Ok(Object::Bool(if lhs { 1 } else { 0 } < rhs)),
                Object::Float(rhs) => Ok(Object::Bool(if lhs { 1f64 } else { 0f64 } < rhs)),
                Object::Bool(rhs) => Ok(Object::Bool(if lhs { 1 } else { 0 } < rhs as i64)),
                _ => return error_msg,
            },
            _ => return error_msg,
        }
    }

    /// Defines the less-than-equal operation of Hinton objects.
    pub fn lteq(self, rhs: Object) -> Result<Object, String> {
        let error_msg = Err(format!(
            "Operation '<=' not defined for objects of type '{}' and '{}'.",
            self.type_name(),
            rhs.type_name()
        ));

        match self {
            Object::Int(lhs) => match rhs {
                Object::Int(rhs) => Ok(Object::Bool(lhs <= rhs)),
                Object::Float(rhs) => Ok(Object::Bool((lhs as f64) <= rhs)),
                Object::Bool(rhs) if rhs => Ok(Object::Bool(lhs <= if rhs { 1 } else { 0 })),
                _ => return error_msg,
            },
            Object::Float(lhs) => match rhs {
                Object::Int(rhs) => Ok(Object::Bool(lhs <= rhs as f64)),
                Object::Float(rhs) => Ok(Object::Bool(lhs <= rhs)),
                Object::Bool(rhs) if rhs => Ok(Object::Bool(lhs <= if rhs { 1f64 } else { 0f64 })),
                _ => return error_msg,
            },
            Object::Bool(lhs) => match rhs {
                Object::Int(rhs) => Ok(Object::Bool(if lhs { 1 } else { 0 } <= rhs)),
                Object::Float(rhs) => Ok(Object::Bool(if lhs { 1f64 } else { 0f64 } <= rhs)),
                Object::Bool(rhs) => Ok(Object::Bool(if lhs { 1 } else { 0 } <= rhs as i64)),
                _ => return error_msg,
            },
            _ => return error_msg,
        }
    }

    /// Defines the indexing operation of Hinton objects.
    pub fn get(&self, index: &Object) -> Result<Object, String> {
        let to_bounded_index = |x: &i64, len: usize| -> Option<usize> {
            if x >= &0 && (*x as usize) < len {
                Some(*x as usize)
            } else if x < &0 && (i64::abs(*x) as usize <= len) {
                Some(len - i64::abs(*x) as usize)
            } else {
                None
            }
        };

        match self {
            Object::Array(arr) => match index {
                // Indexing type: Array[Int]
                Object::Int(idx) => {
                    if let Some(pos) = to_bounded_index(idx, arr.len()) {
                        if let Some(val) = arr.get(pos) {
                            return Ok(val.clone());
                        }
                    }

                    return Err(String::from("Array index out of bounds."));
                }
                // Indexing type: Array[Range]
                Object::Range(_) => {
                    unimplemented!("Array indexing with ranges.")
                }
                _ => {
                    return Err(format!(
                        "Array index must be an Int or a Range. Found '{}' instead.",
                        index.type_name()
                    ))
                }
            },
            Object::String(str) => match index {
                // Indexing type: String[Int]
                Object::Int(idx) => {
                    let chars: Vec<char> = str.chars().collect();

                    if let Some(pos) = to_bounded_index(idx, chars.len()) {
                        if let Some(val) = chars.get(pos) {
                            return Ok(Object::String(val.to_string()));
                        }
                    }

                    return Err(String::from("String index out of bounds."));
                }
                // Indexing type: String[Range]
                Object::Range(_) => {
                    unimplemented!("String indexing with ranges.")
                }
                _ => {
                    return Err(format!(
                        "String index must be an Int or a Range. Found '{}' instead.",
                        index.type_name()
                    ))
                }
            },
            Object::Range(range) => match index {
                // Indexing type: Range[Int]
                Object::Int(idx) => {
                    let min = range.min;
                    let max = range.max;

                    if let Some(pos) = to_bounded_index(idx, i64::abs(max - min) as usize) {
                        return if max - min > 0 {
                            Ok(Object::Int(min + pos as i64))
                        } else {
                            Ok(Object::Int(min - pos as i64))
                        };
                    }

                    return Err(String::from("Range index out of bounds."));
                }
                // Indexing type: Range[Range]
                Object::Range(_) => {
                    unimplemented!("Range indexing with ranges.")
                }
                _ => {
                    return Err(format!(
                        "Range index must be an Int or a Range. Found '{}' instead.",
                        index.type_name()
                    ))
                }
            },
            _ => {
                return Err(format!(
                    "Cannot index object of type '{}'.",
                    self.type_name()
                ))
            }
        }
    }
}
