use crate::gc::GcTrace;
use crate::GcObject;

/// A Hinton string object.
#[derive(PartialEq, Eq)]
pub struct RangeObj {
  pub min: i64,
  pub max: i64,
  pub closed: bool,
}

impl GcTrace for RangeObj {}

impl RangeObj {
  pub fn display_pretty(&self) -> String {
    format!(
      "(\x1b[38;5;81m{}\x1b[0m..{}\x1b[38;5;81m{}\x1b[0m)",
      self.min,
      if self.closed { "=" } else { "" },
      self.max
    )
  }

  pub fn display_plain(&self) -> String {
    format!("({}..{}{})", self.min, if self.closed { "=" } else { "" }, self.max)
  }

  pub fn equals(&self, obj: &GcObject) -> bool {
    match obj {
      GcObject::Range(r) => self == r,
      _ => false,
    }
  }
}
