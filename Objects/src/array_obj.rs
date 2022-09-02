use crate::gc::GcTrace;
use crate::{to_wrapping_index, try_convert_to_idx, GarbageCollector, GcObject, Object, RuntimeErrMsg};

/// A Hinton array object.
pub struct ArrayObj(pub Vec<Object>);

impl GcTrace for ArrayObj {}

impl ArrayObj {
  pub fn equals(&self, obj: &GcObject, gc: &GarbageCollector) -> bool {
    match obj {
      GcObject::Array(r) => {
        // If the arrays are different in length, then they are not equal.
        if self.0.len() != r.0.len() {
          return false;
        }

        // check all elements of both arrays are equal
        for idx in 0..self.0.len() {
          if !&self.0[idx].equals(&self.0[idx], gc) {
            return false;
          }
        }

        true
      }
      _ => false,
    }
  }

  pub fn assign_at(&mut self, idx: Object, val: Object) -> Result<Object, RuntimeErrMsg> {
    let index = try_convert_to_idx(&idx, "Array")?;

    if let Some(idx) = to_wrapping_index(index, self.0.len()) {
      self.0[idx] = val;
      Ok(val)
    } else {
      Err(RuntimeErrMsg::Index("Array index out of bounds.".into()))
    }
  }

  pub fn display_plain(&self, gc: &GarbageCollector) -> String {
    let mut display = String::from("[");

    if !self.0.is_empty() {
      for v in &self.0[..self.0.len() - 1] {
        display += &*(v.display_plain(gc) + ", ");
      }

      display += &*self.0[self.0.len() - 1].display_plain(gc);
    }

    display.push(']');
    display
  }

  pub fn display_pretty(&self, gc: &GarbageCollector) -> String {
    let mut display = String::from("[");

    if !self.0.is_empty() {
      for v in &self.0[..self.0.len() - 1] {
        display += &*(v.display_pretty(gc) + ", ");
      }

      display += &*self.0[self.0.len() - 1].display_pretty(gc);
    }

    display.push(']');
    display
  }
}
