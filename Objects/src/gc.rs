use crate::func_obj::FuncObj;
use crate::str_obj::StrObj;
use std::fmt::{Debug, Formatter};

/// Garbage collected objects.
#[derive(PartialEq)]
pub enum GcObject {
  Str(StrObj),
  Func(FuncObj),
}

/// The kinds of garbage-collected objects.
#[derive(Copy, Clone)]
pub enum GcObjectKind {
  Str,
  Func,
}

impl Debug for GcObject {
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    match self {
      GcObject::Str(s) => write!(f, "{:?}", s),
      GcObject::Func(c) => write!(f, "{:?}", c),
    }
  }
}

impl From<String> for GcObject {
  fn from(s: String) -> Self {
    GcObject::Str(StrObj(s))
  }
}

impl From<FuncObj> for GcObject {
  fn from(f: FuncObj) -> Self {
    GcObject::Func(f)
  }
}

impl GcObject {
  /// Gets the kind of garbage-collected object associated with this variant.
  pub fn kind(&self) -> GcObjectKind {
    match self {
      GcObject::Str(_) => GcObjectKind::Str,
      GcObject::Func(_) => GcObjectKind::Func,
    }
  }
}

/// An object stored in the garbage collector.
#[derive(PartialEq)]
pub struct GcVal {
  pub obj: GcObject,
}

impl Debug for GcVal {
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    write!(f, "{:?}", self.obj)
  }
}

impl From<GcObject> for GcVal {
  fn from(o: GcObject) -> Self {
    GcVal { obj: o }
  }
}

impl From<String> for GcVal {
  fn from(s: String) -> Self {
    GcVal { obj: s.into() }
  }
}

impl From<FuncObj> for GcVal {
  fn from(f: FuncObj) -> Self {
    GcVal { obj: f.into() }
  }
}

impl GcVal {
  /// Tries to extract an immutable reference to the
  /// underlying `StrObj` in this `GcVal`.
  pub fn as_str_obj(&self) -> Option<&StrObj> {
    match &self.obj {
      GcObject::Str(obj) => Some(obj),
      _ => None,
    }
  }

  /// Tries to extract an immutable reference to the
  /// underlying `FuncObj` in this `GcVal`.
  pub fn as_func_obj(&self) -> Option<&FuncObj> {
    match &self.obj {
      GcObject::Func(obj) => Some(obj),
      _ => None,
    }
  }

  /// Tries to extract a mutable reference to the
  /// underlying `FuncObj` in this `GcVal`.
  pub fn as_func_obj_mut(&mut self) -> Option<&mut FuncObj> {
    match &mut self.obj {
      GcObject::Func(obj) => Some(obj),
      _ => None,
    }
  }
}

/// A trait for tracing garbage-collected objects.
pub trait GcTrace {}

/// The identifier of an object in the garbage collector.
#[derive(Default, Copy, Clone, Eq, PartialEq)]
pub struct GcId(pub usize);

/// The garbage collector.
/// Stores the memory-allocated objects, as well as tombstones that can be reused.
#[derive(Default)]
pub struct GarbageCollector {
  pub objects: Vec<GcVal>,
  tombstones: Vec<usize>,
}

impl GarbageCollector {
  /// Pushes a new object into the garbage collector.
  ///
  /// # Arguments
  ///
  /// * `obj`: The new object to add to the garbage collector.
  ///
  /// # Returns:
  /// ```GcId```
  pub fn push(&mut self, obj: GcObject) -> GcId {
    let obj = GcVal::from(obj);

    // TODO: This is painfully slow, and only needed for string interning.
    if let Some(idx) = self.objects.iter().position(|o| *o == obj) {
      return GcId(idx);
    }

    if let Some(idx) = self.tombstones.pop() {
      self.objects[idx] = obj;
      GcId(idx)
    } else {
      self.objects.push(obj);
      GcId(self.objects.len() - 1)
    }
  }

  /// Gets an immutable reference into the `GcVal` associated with the give `GcId`.
  ///
  /// # Arguments
  ///
  /// * `id`: The index of the object.
  ///
  /// # Returns:
  /// ```&GcVal```
  pub fn get(&self, id: &GcId) -> &GcVal {
    &self.objects[id.0]
  }

  /// Gets a mutable reference into the `GcVal` associated with the give `GcId`.
  ///
  /// # Arguments
  ///
  /// * `id`: The index of the object.
  ///
  /// # Returns:
  /// ```&mut GcVal```
  pub fn get_mut(&mut self, id: &GcId) -> &mut GcVal {
    &mut self.objects[id.0]
  }
}
