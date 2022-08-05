use std::cell::RefCell;
use std::rc::Rc;

use crate::core::bytecode::OpCode;
use crate::objects::class_obj::BoundMethod;
use crate::objects::{ClosureObject, FuncObject, Object};

#[derive(Clone)]
pub enum CallFrameType {
  Closure(ClosureObject),
  Function(Rc<RefCell<FuncObject>>),
  Method(BoundMethod),
}

/// Represents a single ongoing function call.
pub struct CallFrame {
  /// A closure call frame.
  pub(crate) callee: CallFrameType,
  pub(crate) ip: usize,
  pub(crate) return_index: usize,
}

impl CallFrame {
  /// Gets the current instruction without incrementing the instruction pointer.
  pub fn peek_current_op_code(&self) -> OpCode {
    match &self.callee {
      CallFrameType::Closure(c) => c.function.borrow().chunk.get_op_code(self.ip - 1),
      CallFrameType::Function(f) => f.borrow().chunk.get_op_code(self.ip - 1),
      CallFrameType::Method(m) => m.method.function.borrow().chunk.get_op_code(self.ip - 1),
    }
  }

  /// Gets the current instruction and advances the instruction pointer to the next instruction.
  pub fn next_op_code(&mut self) -> OpCode {
    self.ip += 1;
    self.peek_current_op_code()
  }

  /// Gets the current raw byte and advances the instruction pointer to the next instruction.
  pub fn next_byte(&mut self) -> u8 {
    self.ip += 1;

    match &self.callee {
      CallFrameType::Closure(c) => c.function.borrow().chunk.get_byte(self.ip - 1),
      CallFrameType::Function(f) => f.borrow().chunk.get_byte(self.ip - 1),
      CallFrameType::Method(m) => m.method.function.borrow().chunk.get_byte(self.ip - 1),
    }
  }

  /// Gets the current two raw bytes and advances the instruction pointer by 2 instructions.
  pub fn next_short(&mut self) -> u16 {
    self.ip += 2;

    match &self.callee {
      CallFrameType::Closure(c) => c.function.borrow().chunk.get_short(self.ip - 2),
      CallFrameType::Function(f) => f.borrow().chunk.get_short(self.ip - 2),
      CallFrameType::Method(m) => m.method.function.borrow().chunk.get_short(self.ip - 2),
    }
  }

  /// Gets an object from the current call frame's constant pool.
  pub fn get_constant(&self, idx: usize) -> Object {
    match &self.callee {
      CallFrameType::Closure(c) => c.function.borrow().chunk.get_constant(idx).clone(),
      CallFrameType::Function(f) => f.borrow().chunk.get_constant(idx).clone(),
      CallFrameType::Method(m) => m.method.function.borrow().chunk.get_constant(idx).clone(),
    }
  }
}
