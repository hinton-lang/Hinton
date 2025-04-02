use core::errors::RuntimeErrMsg;
use core::utils::to_wrapping_index;

use crate::gc::{GarbageCollector, GcId, GcObject, GcObjectKind};
use crate::native_func_obj::NativeFuncObj;

pub mod array_obj;
pub mod func_obj;
pub mod gc;
pub mod native_func_obj;
pub mod range_obj;
pub mod str_obj;
pub mod tuple_obj;

/// An object (or first-class citizen) in Hinton.
#[derive(Copy, Clone)]
pub enum Object {
  None,
  Int(i64),
  Float(f64),
  Bool(bool),
  Str(GcId),
  Array(GcId),
  Tuple(GcId),
  Range(GcId),
  Func(GcId),
  // We only store the index of the native function we are referring to.
  NativeFunc(NativeFuncObj),
}

/// The kinds of objects available in Hinton.
pub enum ObjKind {
  None,
  Int,
  Float,
  Bool,
  Str,
  Array,
  Tuple,
  Range,
  Func,
  NativeFunc,
}

/// An constant representing the atomic object "none" in Hinton.
pub const OBJ_NONE: Object = Object::None;
/// An constant representing the atomic object "true" in Hinton.
pub const OBJ_TRUE: Object = Object::Bool(true);
/// An constant representing the atomic object "false" in Hinton.
pub const OBJ_FALSE: Object = Object::Bool(false);

impl Object {
  /// Gets the kind of Hinton object associated with this variant.
  pub fn kind(&self) -> ObjKind {
    match self {
      Object::None => ObjKind::None,
      Object::Int(_) => ObjKind::Int,
      Object::Float(_) => ObjKind::Float,
      Object::Bool(_) => ObjKind::Bool,
      Object::Str(_) => ObjKind::Str,
      Object::Array(_) => ObjKind::Array,
      Object::Tuple(_) => ObjKind::Tuple,
      Object::Range(_) => ObjKind::Range,
      Object::Func(_) => ObjKind::Func,
      Object::NativeFunc(_) => ObjKind::NativeFunc,
    }
  }

  /// Gets the typename of a Hinton object.
  pub fn type_name(&self) -> &str {
    match self.kind() {
      ObjKind::None => "None",
      ObjKind::Int => "Int",
      ObjKind::Float => "Float",
      ObjKind::Bool => "Bool",
      ObjKind::Str => "Str",
      ObjKind::Array => "Array",
      ObjKind::Tuple => "Tuple",
      ObjKind::Range => "Range",
      ObjKind::Func | ObjKind::NativeFunc => "Func",
    }
  }

  /// Checks that this object is a Hinton integer.
  pub fn is_int(&self) -> bool {
    matches!(self, Object::Int(_))
  }

  /// Checks that this object is a Hinton float.
  pub fn is_float(&self) -> bool {
    matches!(self, Object::Float(_))
  }

  /// Checks that this object is a Hinton boolean.
  pub fn is_bool(&self) -> bool {
    matches!(self, Object::Bool(_))
  }

  /// Checks that this object hols a numeric 0 value.
  pub fn is_zero(&self) -> bool {
    self.is_int() && self.as_int().unwrap() == 0
      || self.is_float() && self.as_float().unwrap() == 0f64
      || self.is_bool() && !self.as_bool().unwrap()
  }

  /// Checks that this object is falsy.
  pub fn is_falsy(&self) -> bool {
    match self {
      Self::None => true,
      Self::Bool(val) => !val,
      Self::Int(x) if *x == 0i64 => true,
      Self::Float(x) if *x == 0f64 => true,
      // TODO: Empty collections are falsy
      _ => false,
    }
  }

  /// Tries to convert this object to a Rust i64 integer.
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

  /// Tries to convert this object to a Rust f64 float.
  pub fn as_float(&self) -> Option<f64> {
    match self {
      Object::Float(v) => Some(*v),
      _ => None,
    }
  }

  /// Tries to convert this object to a Rust boolean.
  pub fn as_bool(&self) -> Option<bool> {
    match self {
      Object::Bool(v) => Some(*v),
      _ => None,
    }
  }

  /// Tries to convert this object to Func variant's GC ID
  pub fn as_func(&self) -> Option<&GcId> {
    match self {
      Object::Func(id) => Some(id),
      _ => None,
    }
  }

  pub fn display_plain(&self, gc: &GarbageCollector) -> String {
    match self {
      Object::None => "none".into(),
      Object::Int(i) => i.to_string(),
      Object::Float(f) => f.to_string(),
      Object::Bool(true) => "true".into(),
      Object::Bool(false) => "false".into(),
      Object::Str(o) | Object::Func(o) | Object::Array(o) | Object::Range(o) | Object::Tuple(o) => {
        gc.get(o).obj.display_plain(gc)
      }
      Object::NativeFunc(n) => n.display_plain(),
    }
  }

  pub fn display_pretty(&self, gc: &GarbageCollector) -> String {
    match self {
      Object::None => "none".into(),
      Object::Int(i) => format!("\x1b[38;5;81m{}\x1b[0m", i),
      Object::Float(f) => format!("\x1b[38;5;81m{}{}\x1b[0m", f, if f.fract() == 0.0 { ".0" } else { "" }),
      Object::Bool(true) => "\x1b[38;5;3mtrue\x1b[0m".into(),
      Object::Bool(false) => "\x1b[38;5;3mfalse\x1b[0m".into(),
      Object::Str(o) | Object::Func(o) | Object::Array(o) | Object::Range(o) | Object::Tuple(o) => {
        gc.get(o).obj.display_pretty(gc)
      }
      Object::NativeFunc(n) => n.display_plain(),
    }
  }
}

impl From<i64> for Object {
  fn from(v: i64) -> Self {
    Object::Int(v)
  }
}

impl From<f64> for Object {
  fn from(v: f64) -> Self {
    Object::Float(v)
  }
}

impl From<bool> for Object {
  fn from(v: bool) -> Self {
    if v {
      OBJ_TRUE
    } else {
      OBJ_FALSE
    }
  }
}

/// Construct a type-mismatch error message for a binary operation.
#[macro_export]
macro_rules! binary_opr_error_msg {
  ($opr: expr, $lhs_type: expr, $rhs_type: expr) => {
    Err(RuntimeErrMsg::Type(format!(
      "Operation '{}' not defined for objects of type '{}' and '{}'.",
      $opr, $lhs_type, $rhs_type
    )))
  };
}

pub fn try_convert_to_idx(obj: &Object, name: &str) -> Result<i64, RuntimeErrMsg> {
  match obj {
    Object::Int(i) => Ok(*i),
    Object::Bool(i) => Ok(*i as i64),
    _ => Err(RuntimeErrMsg::Type(format!(
      "{} index must be an Int or a Slice. Found '{}' instead.",
      name,
      obj.type_name()
    ))),
  }
}

impl Object {
  /// Defines addition of Hinton objects.
  pub fn add(&self, rhs: &Object, gc: &mut GarbageCollector) -> Result<Object, RuntimeErrMsg> {
    match self {
      Object::Int(lhs) => match rhs {
        Object::Int(rhs) => Ok(Object::Int(lhs + rhs)),
        Object::Float(rhs) => Ok(Object::Float(*lhs as f64 + rhs)),
        Object::Bool(rhs) => Ok(Object::Int(lhs + *rhs as i64)),
        Object::Str(rhs) => {
          let rhs = gc.get(rhs).as_str_obj().unwrap();
          Ok(Object::Str(gc.push(format!("{}{}", lhs, rhs.0).into())))
        }
        _ => binary_opr_error_msg!("+", "Int", rhs.type_name()),
      },
      Object::Float(lhs) => match rhs {
        Object::Int(rhs) => Ok(Object::Float(lhs + *rhs as f64)),
        Object::Float(rhs) => Ok(Object::Float(lhs + rhs)),
        Object::Bool(rhs) => Ok(Object::Float(lhs + *rhs as i64 as f64)),
        Object::Str(rhs) => {
          let rhs = gc.get(rhs).as_str_obj().unwrap();
          let frac = if lhs.fract() == 0.0 { ".0" } else { "" };
          Ok(Object::Str(gc.push(format!("{}{}{}", lhs, frac, rhs.0).into())))
        }
        _ => binary_opr_error_msg!("+", "Float", rhs.type_name()),
      },
      Object::Str(lhs) => {
        let lhs = gc.get(lhs).as_str_obj().unwrap();
        let new_str = match rhs {
          Object::Int(rhs) => format!("{}{}", lhs.0, rhs),
          Object::Float(rhs) => format!("{}{}{}", lhs.0, rhs, if rhs.fract() == 0.0 { ".0" } else { "" }),
          Object::Str(rhs) => format!("{}{}", lhs.0, gc.get(rhs).as_str_obj().unwrap().0),
          _ => return binary_opr_error_msg!("+", "Str", rhs.type_name()),
        };
        Ok(Object::Str(gc.push(new_str.into())))
      }
      Object::Bool(lhs) if *lhs => match rhs {
        Object::Int(rhs) => Ok(Object::Int(rhs + 1i64)),
        Object::Float(rhs) => Ok(Object::Float(rhs + 1f64)),
        Object::Bool(rhs) => Ok(Object::Int(1 + *rhs as i64)),
        _ => binary_opr_error_msg!("+", "Bool", rhs.type_name()),
      },
      Object::Bool(lhs) if !lhs => match rhs {
        Object::Int(rhs) => Ok(Object::Int(*rhs)),
        Object::Float(rhs) => Ok(Object::Float(*rhs)),
        Object::Bool(rhs) => Ok(Object::Int(*rhs as i64)),
        _ => binary_opr_error_msg!("+", "Bool", rhs.type_name()),
      },
      _ => binary_opr_error_msg!("+", self.type_name(), rhs.type_name()),
    }
  }

  /// Defines subtraction of Hinton objects.
  pub fn sub(&self, rhs: &Object) -> Result<Object, RuntimeErrMsg> {
    match self {
      Object::Int(lhs) => match rhs {
        Object::Int(rhs) => Ok(Object::Int(lhs - rhs)),
        Object::Float(rhs) => Ok(Object::Float(*lhs as f64 - rhs)),
        Object::Bool(rhs) => Ok(Object::Int(lhs - *rhs as i64)),
        _ => binary_opr_error_msg!("-", "Int", rhs.type_name()),
      },
      Object::Float(lhs) => match rhs {
        Object::Int(rhs) => Ok(Object::Float(lhs - *rhs as f64)),
        Object::Float(rhs) => Ok(Object::Float(lhs - rhs)),
        Object::Bool(rhs) => Ok(Object::Float(lhs - *rhs as i64 as f64)),
        _ => binary_opr_error_msg!("-", "Float", rhs.type_name()),
      },
      Object::Bool(lhs) if *lhs => match rhs {
        Object::Int(rhs) => Ok(Object::Int(1i64 - rhs)),
        Object::Float(rhs) => Ok(Object::Float(1f64 - rhs)),
        Object::Bool(rhs) => Ok(Object::Int(1 - *rhs as i64)),
        _ => binary_opr_error_msg!("-", "Bool", rhs.type_name()),
      },
      Object::Bool(lhs) if !lhs => match rhs {
        Object::Int(rhs) => Ok(Object::Int(-*rhs)),
        Object::Float(rhs) => Ok(Object::Float(-*rhs)),
        Object::Bool(rhs) => Ok(Object::Int(*rhs as i64)),
        _ => binary_opr_error_msg!("-", "Bool", rhs.type_name()),
      },
      _ => binary_opr_error_msg!("-", self.type_name(), rhs.type_name()),
    }
  }

  /// Defines multiplication of Hinton objects.
  pub fn mul(&self, rhs: &Object, gc: &mut GarbageCollector) -> Result<Object, RuntimeErrMsg> {
    match self {
      Object::Int(lhs) => match rhs {
        Object::Int(rhs) => Ok(Object::Int(lhs.saturating_mul(*rhs))),
        Object::Float(rhs) => Ok(Object::Float(*lhs as f64 * rhs)),
        Object::Bool(rhs) => Ok(Object::Int(*lhs * *rhs as i64)),
        Object::Str(rhs) => {
          let rhs = gc.get(rhs).as_str_obj().unwrap();
          Ok(Object::Str(gc.push(rhs.0.repeat(*lhs as usize).into())))
        }
        _ => binary_opr_error_msg!("*", "Int", rhs.type_name()),
      },
      Object::Float(lhs) => match rhs {
        Object::Int(rhs) => Ok(Object::Float(lhs * *rhs as f64)),
        Object::Float(rhs) => Ok(Object::Float(lhs * rhs)),
        Object::Bool(rhs) => Ok(Object::Float(*lhs * *rhs as i64 as f64)),
        _ => binary_opr_error_msg!("*", "Float", rhs.type_name()),
      },
      Object::Str(lhs) => {
        let lhs = gc.get(lhs).as_str_obj().unwrap();
        match rhs {
          Object::Int(rhs) => Ok(Object::Str(gc.push(lhs.0.repeat(*rhs as usize).into()))),
          _ => binary_opr_error_msg!("*", "Str", rhs.type_name()),
        }
      }
      Object::Bool(lhs) if *lhs => match rhs {
        Object::Int(_) | Object::Float(_) | Object::Bool(_) => Ok(*rhs),
        _ => binary_opr_error_msg!("*", "Bool", rhs.type_name()),
      },
      Object::Bool(lhs) if !lhs => match rhs {
        Object::Int(_) => Ok(Object::Int(0)),
        Object::Float(_) => Ok(Object::Float(0f64)),
        Object::Bool(_) => Ok(Object::Int(0)),
        _ => binary_opr_error_msg!("*", "Bool", rhs.type_name()),
      },
      // TODO: List can be multiplied by integers
      _ => binary_opr_error_msg!("*", self.type_name(), rhs.type_name()),
    }
  }

  /// Defines division of Hinton objects.
  pub fn div(&self, rhs: &Object) -> Result<Object, RuntimeErrMsg> {
    if rhs.is_zero() {
      return Err(RuntimeErrMsg::ZeroDivision("Cannot divide by zero.".into()));
    }

    match self {
      Object::Int(lhs) => match rhs {
        Object::Int(rhs) => Ok(Object::Float(*lhs as f64 / *rhs as f64)),
        Object::Float(rhs) => Ok(Object::Float(*lhs as f64 / rhs)),
        Object::Bool(_) => Ok(Object::Float(*lhs as f64)),
        _ => binary_opr_error_msg!("/", "Int", rhs.type_name()),
      },
      Object::Float(lhs) => match rhs {
        Object::Int(rhs) => Ok(Object::Float(lhs / *rhs as f64)),
        Object::Float(rhs) => Ok(Object::Float(lhs / rhs)),
        Object::Bool(_) => Ok(Object::Float(*lhs as f64)),
        _ => binary_opr_error_msg!("/", "Float", rhs.type_name()),
      },
      Object::Bool(lhs) if *lhs => match rhs {
        Object::Int(rhs) => Ok(Object::Float(1f64 / *rhs as f64)),
        Object::Float(rhs) => Ok(Object::Float(1f64 / rhs)),
        Object::Bool(_) => Ok(Object::Float(1f64)),
        _ => binary_opr_error_msg!("/", "Bool", rhs.type_name()),
      },
      Object::Bool(lhs) if !lhs => match rhs {
        Object::Int(_) => Ok(Object::Float(0f64)),
        Object::Float(_) => Ok(Object::Float(0f64)),
        Object::Bool(_) => Ok(Object::Float(0f64)),
        _ => binary_opr_error_msg!("/", "Bool", rhs.type_name()),
      },
      _ => binary_opr_error_msg!("/", self.type_name(), rhs.type_name()),
    }
  }

  /// Defines exponentiation of Hinton objects.
  pub fn pow(&self, rhs: &Object) -> Result<Object, RuntimeErrMsg> {
    match self {
      // TODO: These conversions seems error-prone and slow...
      Object::Int(lhs) => match rhs {
        Object::Int(rhs) => Ok(Object::Int((*lhs as f64).powf(*rhs as f64) as i64)),
        Object::Float(rhs) => Ok(Object::Float((*lhs as f64).powf(*rhs))),
        Object::Bool(rhs) if *rhs => Ok(Object::Int(*lhs)),
        Object::Bool(rhs) if !rhs => Ok(Object::Int(1)),
        _ => binary_opr_error_msg!("**", "Int", rhs.type_name()),
      },
      Object::Float(lhs) => match rhs {
        Object::Int(rhs) => Ok(Object::Float(lhs.powf(*rhs as f64))),
        Object::Float(rhs) => Ok(Object::Float(lhs.powf(*rhs))),
        Object::Bool(rhs) if *rhs => Ok(Object::Float(*lhs)),
        Object::Bool(rhs) if !rhs => Ok(Object::Float(1f64)),
        _ => binary_opr_error_msg!("**", "Float", rhs.type_name()),
      },
      Object::Bool(lhs) if *lhs => match rhs {
        Object::Int(_) => Ok(Object::Int(1i64)),
        Object::Float(_) => Ok(Object::Float(1f64)),
        Object::Bool(_) => Ok(Object::Int(1i64)),
        _ => binary_opr_error_msg!("**", "Bool", rhs.type_name()),
      },
      Object::Bool(lhs) if !lhs => match rhs {
        Object::Int(rhs) => Ok(Object::Int(0f64.powf(*rhs as f64) as i64)),
        Object::Float(rhs) => Ok(Object::Float(0f64.powf(*rhs))),
        Object::Bool(rhs) if *rhs => Ok(Object::Int(0i64)),
        Object::Bool(rhs) if !rhs => Ok(Object::Int(1i64)),
        _ => binary_opr_error_msg!("**", "Bool", rhs.type_name()),
      },
      _ => binary_opr_error_msg!("**", self.type_name(), rhs.type_name()),
    }
  }

  /// Defines modulo of Hinton objects.
  pub fn rem(&self, rhs: &Object) -> Result<Object, RuntimeErrMsg> {
    if rhs.is_zero() {
      return Err(RuntimeErrMsg::ZeroDivision(
        "Right-hand-side of '%' cannot be zero.".into(),
      ));
    }

    match self {
      Object::Int(lhs) => match rhs {
        Object::Int(rhs) => Ok(Object::Int(lhs % rhs)),
        Object::Float(rhs) => Ok(Object::Int(lhs % rhs.floor() as i64)),
        Object::Bool(_) => Ok(Object::Int(0i64)),
        _ => binary_opr_error_msg!("%", "Int", rhs.type_name()),
      },
      Object::Float(lhs) => match rhs {
        Object::Int(rhs) => Ok(Object::Float(lhs % *rhs as f64)),
        Object::Float(rhs) => Ok(Object::Float(lhs % rhs)),
        Object::Bool(_) => Ok(Object::Float(lhs % 1f64)),
        _ => binary_opr_error_msg!("%", "Float", rhs.type_name()),
      },
      Object::Bool(lhs) if *lhs => match rhs {
        Object::Int(rhs) => Ok(Object::Int(1i64 % rhs)),
        Object::Float(rhs) => Ok(Object::Float(1f64 % rhs)),
        Object::Bool(_) => Ok(Object::Int(0i64)),
        _ => binary_opr_error_msg!("%", "Bool", rhs.type_name()),
      },
      Object::Bool(lhs) if !lhs => match rhs {
        Object::Int(_) => Ok(Object::Int(0i64)),
        Object::Float(_) => Ok(Object::Float(0f64)),
        Object::Bool(_) => Ok(Object::Int(0i64)),
        _ => binary_opr_error_msg!("%", "Bool", rhs.type_name()),
      },
      _ => binary_opr_error_msg!("%", self.type_name(), rhs.type_name()),
    }
  }

  /// Defines negation of Hinton objects.
  pub fn neg(&self) -> Result<Object, RuntimeErrMsg> {
    match self {
      Object::Int(lhs) => Ok(Object::Int(-*lhs)),
      Object::Float(lhs) => Ok(Object::Float(-*lhs)),
      Object::Bool(lhs) if *lhs => Ok(Object::Int(-1)),
      Object::Bool(lhs) if !lhs => Ok(Object::Int(0)),
      _ => Err(RuntimeErrMsg::Type(format!(
        "Cannot negate objects of type '{}'.",
        self.type_name()
      ))),
    }
  }

  /// Defines the bitwise-and operation of Hinton objects.
  pub fn bit_and(&self, rhs: &Object) -> Result<Object, RuntimeErrMsg> {
    match self {
      Object::Int(lhs) => match rhs {
        Object::Int(rhs) => Ok(Object::Int(lhs & rhs)),
        Object::Bool(rhs) => Ok(Object::Int(lhs & *rhs as i64)),
        _ => binary_opr_error_msg!("&", "Int", rhs.type_name()),
      },
      Object::Bool(lhs) if *lhs => match rhs {
        Object::Int(rhs) => Ok(Object::Int(*lhs as i64 & rhs)),
        Object::Bool(rhs) => Ok(Object::Int(*lhs as i64 & *rhs as i64)),
        _ => binary_opr_error_msg!("&", "Bool", rhs.type_name()),
      },
      _ => binary_opr_error_msg!("&", self.type_name(), rhs.type_name()),
    }
  }

  /// Defines the bitwise-or operation of Hinton objects.
  pub fn bit_or(&self, rhs: &Object) -> Result<Object, RuntimeErrMsg> {
    match self {
      Object::Int(lhs) => match rhs {
        Object::Int(rhs) => Ok(Object::Int(lhs | rhs)),
        Object::Bool(rhs) => Ok(Object::Int(lhs | *rhs as i64)),
        _ => binary_opr_error_msg!("|", "Int", rhs.type_name()),
      },
      Object::Bool(lhs) if *lhs => match rhs {
        Object::Int(rhs) => Ok(Object::Int(*lhs as i64 | rhs)),
        Object::Bool(rhs) => Ok(Object::Int(*lhs as i64 | *rhs as i64)),
        _ => binary_opr_error_msg!("|", "Bool", rhs.type_name()),
      },
      _ => binary_opr_error_msg!("|", self.type_name(), rhs.type_name()),
    }
  }

  /// Defines the bitwise-xor operation of Hinton objects.
  pub fn bit_xor(&self, rhs: &Object) -> Result<Object, RuntimeErrMsg> {
    match self {
      Object::Int(lhs) => match rhs {
        Object::Int(rhs) => Ok(Object::Int(lhs ^ rhs)),
        Object::Bool(rhs) => Ok(Object::Int(lhs ^ *rhs as i64)),
        _ => binary_opr_error_msg!("^", "Int", rhs.type_name()),
      },
      Object::Bool(lhs) if *lhs => match rhs {
        Object::Int(rhs) => Ok(Object::Int(*lhs as i64 ^ rhs)),
        Object::Bool(rhs) => Ok(Object::Int(*lhs as i64 ^ *rhs as i64)),
        _ => binary_opr_error_msg!("^", "Bool", rhs.type_name()),
      },
      _ => binary_opr_error_msg!("^", self.type_name(), rhs.type_name()),
    }
  }

  /// Defines the bitwise-shift-left operation of Hinton objects.
  pub fn shl(&self, rhs: &Object) -> Result<Object, RuntimeErrMsg> {
    match self {
      Object::Int(lhs) => match rhs {
        Object::Int(rhs) => Ok(Object::Int(lhs << rhs)),
        Object::Bool(rhs) => Ok(Object::Int(lhs << *rhs as i64)),
        _ => binary_opr_error_msg!("<<", "Int", rhs.type_name()),
      },
      Object::Bool(lhs) if *lhs => match rhs {
        Object::Int(rhs) => Ok(Object::Int((*lhs as i64) << *rhs)),
        Object::Bool(rhs) => Ok(Object::Int((*lhs as i64) << *rhs as i64)),
        _ => binary_opr_error_msg!("<<", "Bool", rhs.type_name()),
      },
      _ => binary_opr_error_msg!("<<", self.type_name(), rhs.type_name()),
    }
  }

  /// Defines the bitwise-shift-right operation of Hinton objects.
  pub fn shr(&self, rhs: &Object) -> Result<Object, RuntimeErrMsg> {
    match self {
      Object::Int(lhs) => match rhs {
        Object::Int(rhs) => Ok(Object::Int(lhs >> rhs)),
        Object::Bool(rhs) => Ok(Object::Int(lhs >> *rhs as i64)),
        _ => binary_opr_error_msg!(">>", "Int", rhs.type_name()),
      },
      Object::Bool(lhs) if *lhs => match rhs {
        Object::Int(rhs) => Ok(Object::Int(*lhs as i64 >> rhs)),
        Object::Bool(rhs) => Ok(Object::Int(*lhs as i64 >> *rhs as i64)),
        _ => binary_opr_error_msg!(">>", "Bool", rhs.type_name()),
      },
      _ => binary_opr_error_msg!(">>", self.type_name(), rhs.type_name()),
    }
  }

  /// Defines the bitwise-not operation of Hinton objects.
  /// NOTE: For the logic-not operation use the `Value.is_falsy()` method.
  pub fn bit_not(&self) -> Result<Object, RuntimeErrMsg> {
    match self {
      Object::Int(opr) => Ok(Object::Int(!*opr)),
      Object::Bool(opr) => Ok(Object::Int(!(*opr as i64))),
      _ => Err(RuntimeErrMsg::Type(format!(
        "Operation '~' not defined for objects of type '{}'.",
        self.type_name()
      ))),
    }
  }

  /// Defines the greater-than operation of Hinton objects.
  pub fn gt(&self, rhs: &Object) -> Result<Object, RuntimeErrMsg> {
    let res = match self {
      Object::Int(lhs) => match rhs {
        Object::Int(rhs) => lhs > rhs,
        Object::Float(rhs) => (*lhs as f64) > *rhs,
        Object::Bool(rhs) if *rhs => *lhs > *rhs as i64,
        _ => return binary_opr_error_msg!(">", "Int", rhs.type_name()),
      },
      Object::Float(lhs) => match rhs {
        Object::Int(rhs) => *lhs > *rhs as f64,
        Object::Float(rhs) => lhs > rhs,
        Object::Bool(rhs) if *rhs => *lhs > *rhs as i64 as f64,
        _ => return binary_opr_error_msg!(">", "Float", rhs.type_name()),
      },
      Object::Bool(lhs) => match rhs {
        Object::Int(rhs) => *lhs as i64 > *rhs,
        Object::Float(rhs) => (*lhs as i64 as f64) > *rhs,
        Object::Bool(rhs) => *lhs as i64 > *rhs as i64,
        _ => return binary_opr_error_msg!(">", "Bool", rhs.type_name()),
      },
      _ => return binary_opr_error_msg!(">", self.type_name(), rhs.type_name()),
    };

    Ok(Object::Bool(res))
  }

  /// Defines the greater-than-equals operation of Hinton objects.
  pub fn gt_eq(&self, rhs: &Object) -> Result<Object, RuntimeErrMsg> {
    let res = match self {
      Object::Int(lhs) => match rhs {
        Object::Int(rhs) => lhs >= rhs,
        Object::Float(rhs) => (*lhs as f64) >= *rhs,
        Object::Bool(rhs) if *rhs => *lhs >= *rhs as i64,
        _ => return binary_opr_error_msg!(">=", "Int", rhs.type_name()),
      },
      Object::Float(lhs) => match rhs {
        Object::Int(rhs) => *lhs >= *rhs as f64,
        Object::Float(rhs) => lhs >= rhs,
        Object::Bool(rhs) if *rhs => *lhs >= *rhs as i64 as f64,
        _ => return binary_opr_error_msg!(">=", "Float", rhs.type_name()),
      },
      Object::Bool(lhs) => match rhs {
        Object::Int(rhs) => *lhs as i64 >= *rhs,
        Object::Float(rhs) => (*lhs as i64 as f64) >= *rhs,
        Object::Bool(rhs) => *lhs as i64 >= *rhs as i64,
        _ => return binary_opr_error_msg!(">=", "Bool", rhs.type_name()),
      },
      _ => return binary_opr_error_msg!(">=", self.type_name(), rhs.type_name()),
    };

    Ok(Object::Bool(res))
  }

  /// Defines the less-than operation of Hinton objects.
  pub fn lt(&self, rhs: &Object) -> Result<Object, RuntimeErrMsg> {
    let res = match self {
      Object::Int(lhs) => match rhs {
        Object::Int(rhs) => lhs < rhs,
        Object::Float(rhs) => (*lhs as f64) < *rhs,
        Object::Bool(rhs) if *rhs => *lhs < *rhs as i64,
        _ => return binary_opr_error_msg!("<", "Int", rhs.type_name()),
      },
      Object::Float(lhs) => match rhs {
        Object::Int(rhs) => *lhs < *rhs as f64,
        Object::Float(rhs) => lhs < rhs,
        Object::Bool(rhs) if *rhs => *lhs < *rhs as i32 as f64,
        _ => return binary_opr_error_msg!("<", "Float", rhs.type_name()),
      },
      Object::Bool(lhs) => match rhs {
        Object::Int(rhs) => (*lhs as i64) < *rhs,
        Object::Float(rhs) => (*lhs as i64 as f64) < *rhs,
        Object::Bool(rhs) => (*lhs as i64) < *rhs as i64,
        _ => return binary_opr_error_msg!("<", "Bool", rhs.type_name()),
      },
      _ => return binary_opr_error_msg!("<", self.type_name(), rhs.type_name()),
    };

    Ok(Object::Bool(res))
  }

  /// Defines the less-than-equal operation of Hinton objects.
  pub fn lt_eq(&self, rhs: &Object) -> Result<Object, RuntimeErrMsg> {
    let res = match self {
      Object::Int(lhs) => match rhs {
        Object::Int(rhs) => lhs <= rhs,
        Object::Float(rhs) => (*lhs as f64) <= *rhs,
        Object::Bool(rhs) if *rhs => *lhs <= *rhs as i64,
        _ => return binary_opr_error_msg!("<=", "Int", rhs.type_name()),
      },
      Object::Float(lhs) => match rhs {
        Object::Int(rhs) => *lhs <= *rhs as f64,
        Object::Float(rhs) => lhs <= rhs,
        Object::Bool(rhs) if *rhs => *lhs <= *rhs as i64 as f64,
        _ => return binary_opr_error_msg!("<=", "Float", rhs.type_name()),
      },
      Object::Bool(lhs) => match rhs {
        Object::Int(rhs) => *lhs as i64 <= *rhs,
        Object::Float(rhs) => (*lhs as i64 as f64) <= *rhs,
        Object::Bool(rhs) => *lhs as i64 <= *rhs as i64,
        _ => return binary_opr_error_msg!("<=", "Bool", rhs.type_name()),
      },
      _ => return binary_opr_error_msg!("<=", self.type_name(), rhs.type_name()),
    };

    Ok(Object::Bool(res))
  }

  /// Defines the equality operation for Hinton objects.
  pub fn equals(&self, rhs: &Object, gc: &GarbageCollector) -> bool {
    match self {
      Object::None => matches![rhs, Object::None],
      Object::Int(lhs) => match rhs {
        Object::Int(x) if lhs == x => true,
        Object::Float(x) if (x - *lhs as f64) == 0f64 => true,
        Object::Bool(x) if (lhs == &0i64 && !*x) || (lhs == &1i64 && *x) => true,
        _ => false,
      },
      Object::Float(f) => match rhs {
        Object::Int(x) if (f - *x as f64) == 0f64 => true,
        Object::Float(x) if f == x => true,
        Object::Bool(x) if (f == &0f64 && !*x) || (f == &1f64 && *x) => true,
        _ => false,
      },
      Object::Bool(b) => match rhs {
        Object::Int(x) if (x == &0i64 && !*b) || (x == &1i64 && *b) => true,
        Object::Float(x) if (x == &0f64 && !*b) || (x == &1f64 && *b) => true,
        Object::Bool(x) => !(b ^ x),
        _ => false,
      },
      Object::Str(lhs) => match rhs {
        // This works because strings are not repeated in the garbage collector
        Object::Str(rhs) => lhs == rhs,
        _ => false,
      },
      Object::Array(lhs) | Object::Tuple(lhs) => match rhs {
        Object::Array(rhs) | Object::Tuple(rhs) => {
          // If the GcIDs are equal, then the two objects are equal
          if lhs == rhs {
            true
          } else {
            let lhs_arr = &gc.get(lhs).obj;
            let rhs_arr = &gc.get(rhs).obj;
            lhs_arr.equals(rhs_arr, gc)
          }
        }
        _ => false,
      },
      Object::Range(lhs) => match rhs {
        // This works because ranges are not repeated in the garbage collector
        Object::Range(rhs) => lhs == rhs,
        _ => false,
      },
      _ => false,
    }
  }

  /// Define subscripting for Hinton objects.
  pub fn subscript(&self, obj: &Object, gc: &mut GarbageCollector) -> Result<Object, RuntimeErrMsg> {
    match self {
      Object::Str(a) => {
        let index = try_convert_to_idx(obj, "String")?;
        let string = gc.get(a).as_str_obj().unwrap();

        if let Some(idx) = to_wrapping_index(index, string.0.len()) {
          let char = gc.push(string.0.chars().nth(idx).unwrap().to_string().into());
          Ok(Object::Str(char))
        } else {
          Err(RuntimeErrMsg::Index("String index out of bounds.".into()))
        }
      }
      Object::Array(a) => {
        let index = try_convert_to_idx(obj, "Array")?;
        let array = gc.get(a).as_array_obj().unwrap();

        if let Some(idx) = to_wrapping_index(index, array.0.len()) {
          Ok(array.0[idx])
        } else {
          Err(RuntimeErrMsg::Index("Array index out of bounds.".into()))
        }
      }
      Object::Tuple(a) => {
        let index = try_convert_to_idx(obj, "Tuple")?;
        let tuple = gc.get(a).as_tuple_obj().unwrap();

        if let Some(idx) = to_wrapping_index(index, tuple.0.len()) {
          Ok(tuple.0[idx])
        } else {
          Err(RuntimeErrMsg::Index("Tuple index out of bounds.".into()))
        }
      }
      Object::Range(a) => {
        let index = try_convert_to_idx(obj, "Range")?;
        let range = gc.get(a).as_range_obj().unwrap();

        let upper_bound = if range.min <= range.max {
          range.max + if range.closed { 1 } else { 0 } - range.min
        } else {
          range.min - range.max + if range.closed { 1 } else { 0 }
        };

        if let Some(idx) = to_wrapping_index(index, upper_bound as usize) {
          Ok(if range.min <= range.max {
            (range.max + (idx as i64)).into()
          } else {
            (range.min - (idx as i64)).into()
          })
        } else {
          Err(RuntimeErrMsg::Index("Range index out of bounds.".into()))
        }
      }
      _ => {
        let err_msg = format!("Objects of type '{}' are not subscriptable.", self.type_name());
        Err(RuntimeErrMsg::Type(err_msg))
      }
    }
  }

  /// Checks whether an object is in `self`, iff `self` is a collection-like object.
  pub fn is_in(&self, rhs: &Object, gc: &GarbageCollector) -> Result<Object, RuntimeErrMsg> {
    match rhs {
      Object::Str(rhs) => match self {
        Object::Str(lhs) => {
          let lhs = gc.get(lhs).as_str_obj().unwrap();
          let rhs = gc.get(rhs).as_str_obj().unwrap();
          let contains = rhs.0.contains(&lhs.0);
          Ok(if contains { OBJ_TRUE } else { OBJ_FALSE })
        }
        _ => binary_opr_error_msg!("in", self.type_name(), "Str"),
      },
      Object::Array(rhs) | Object::Tuple(rhs) => {
        let obj = gc.get(rhs);
        let rhs = if let GcObjectKind::Array = obj.obj.kind() {
          &obj.as_array_obj().unwrap().0
        } else {
          &obj.as_tuple_obj().unwrap().0
        };

        let mut contains = false;

        for i in rhs {
          if i.equals(self, gc) {
            contains = true;
          }
        }

        Ok(if contains { OBJ_TRUE } else { OBJ_FALSE })
      }
      Object::Range(rhs) if matches!(self.kind(), ObjKind::Int) => {
        let lhs = self.as_int().unwrap();
        let rhs = gc.get(rhs).as_range_obj().unwrap();

        let contains = if rhs.min <= rhs.max {
          lhs >= rhs.min && ((rhs.closed && lhs <= rhs.max) || (!rhs.closed && lhs < rhs.max))
        } else {
          lhs <= rhs.min && ((rhs.closed && lhs >= rhs.max) || (!rhs.closed && lhs > rhs.max))
        };

        Ok(if contains { OBJ_TRUE } else { OBJ_FALSE })
      }
      _ => binary_opr_error_msg!("in", self.type_name(), rhs.type_name()),
    }
  }
}
