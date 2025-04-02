use crate::array_obj::{print_plain_item_list, print_pretty_item_list};
use crate::gc::GcTrace;
use crate::{GarbageCollector, GcObject, Object};

/// A Hinton tuple object.
pub struct TupleObj(pub Vec<Object>);

impl GcTrace for TupleObj {}

impl TupleObj {
  pub fn equals(&self, obj: &GcObject, gc: &GarbageCollector) -> bool {
    match obj {
      GcObject::Tuple(r) if self.0.len() != r.0.len() => false,
      GcObject::Tuple(r) => {
        for idx in 0..self.0.len() {
          if !&self.0[idx].equals(&r.0[idx], gc) {
            return false;
          }
        }

        true
      }
      _ => false,
    }
  }

  pub fn display_plain(&self, gc: &GarbageCollector) -> String {
    format!("({})", print_plain_item_list(&self.0, gc))
  }

  pub fn display_pretty(&self, gc: &GarbageCollector) -> String {
    format!("({})", print_pretty_item_list(&self.0, gc))
  }
}
