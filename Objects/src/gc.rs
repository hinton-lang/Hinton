use crate::array_obj::ArrayObj;
use crate::func_obj::FuncObj;
use crate::range_obj::RangeObj;
use crate::str_obj::StrObj;
use crate::tuple_obj::TupleObj;

/// Garbage collected objects.
pub enum GcObject {
  Str(StrObj),
  Array(ArrayObj),
  Tuple(TupleObj),
  Range(RangeObj),
  Func(FuncObj),
}

/// The kinds of garbage-collected objects.
#[derive(Copy, Clone)]
pub enum GcObjectKind {
  Str,
  Array,
  Tuple,
  Range,
  Func,
}

impl From<String> for GcObject {
  fn from(s: String) -> Self {
    GcObject::Str(StrObj(s))
  }
}

impl From<&str> for GcObject {
  fn from(s: &str) -> Self {
    GcObject::Str(StrObj(s.into()))
  }
}

impl From<char> for GcObject {
  fn from(s: char) -> Self {
    GcObject::Str(StrObj(s.to_string()))
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
      GcObject::Array(_) => GcObjectKind::Array,
      GcObject::Tuple(_) => GcObjectKind::Tuple,
      GcObject::Range(_) => GcObjectKind::Range,
      GcObject::Func(_) => GcObjectKind::Func,
    }
  }

  pub fn equals(&self, obj: &GcObject, gc: &GarbageCollector) -> bool {
    match self {
      GcObject::Str(str) => str.equals(obj),
      GcObject::Array(arr) => arr.equals(obj, gc),
      GcObject::Tuple(tup) => tup.equals(obj, gc),
      _ => false,
    }
  }

  pub fn display_plain(&self, gc: &GarbageCollector) -> String {
    match self {
      GcObject::Str(s) => s.display_plain().to_owned(),
      GcObject::Array(a) => a.display_plain(gc),
      GcObject::Tuple(t) => t.display_plain(gc),
      GcObject::Range(r) => r.display_plain(),
      GcObject::Func(f) => f.display_plain(gc),
    }
  }

  pub fn display_pretty(&self, gc: &GarbageCollector) -> String {
    match self {
      GcObject::Str(s) => s.display_plain().to_owned(),
      GcObject::Array(a) => a.display_pretty(gc),
      GcObject::Tuple(t) => t.display_pretty(gc),
      GcObject::Range(r) => r.display_pretty(),
      GcObject::Func(f) => f.display_plain(gc),
    }
  }
}

/// An object stored in the garbage collector.
pub struct GcVal {
  pub obj: GcObject,
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

impl From<char> for GcVal {
  fn from(c: char) -> Self {
    GcVal { obj: c.into() }
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
  /// underlying `ArrayObj` in this `GcVal`.
  pub fn as_array_obj(&self) -> Option<&ArrayObj> {
    match &self.obj {
      GcObject::Array(obj) => Some(obj),
      _ => None,
    }
  }

  /// Tries to extract a mutable reference to the
  /// underlying `ArrayObj` in this `GcVal`.
  pub fn as_array_obj_mut(&mut self) -> Option<&mut ArrayObj> {
    match &mut self.obj {
      GcObject::Array(obj) => Some(obj),
      _ => None,
    }
  }

  /// Tries to extract an immutable reference to the
  /// underlying `TupleObj` in this `GcVal`.
  pub fn as_tuple_obj(&self) -> Option<&TupleObj> {
    match &self.obj {
      GcObject::Tuple(obj) => Some(obj),
      _ => None,
    }
  }

  /// Tries to extract an immutable reference to the
  /// underlying `RangeObj` in this `GcVal`.
  pub fn as_range_obj(&self) -> Option<&RangeObj> {
    match &self.obj {
      GcObject::Range(obj) => Some(obj),
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
    // TODO: This is painfully slow, and only needed for some objects.
    if matches!(obj.kind(), GcObjectKind::Str | GcObjectKind::Range) {
      if let Some(idx) = self.objects.iter().position(|o| o.obj.equals(&obj, self)) {
        return GcId(idx);
      }
    }

    if let Some(idx) = self.tombstones.pop() {
      self.objects[idx] = GcVal::from(obj);
      GcId(idx)
    } else {
      self.objects.push(GcVal::from(obj));
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
