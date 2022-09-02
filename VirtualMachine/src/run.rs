use std::ops::ControlFlow;

use core::ast::BinaryExprKind;
use core::bytecode::OpCode;
use core::utils::to_wrapping_index;
use objects::array_obj::ArrayObj;
use objects::gc::GcObject;
use objects::{OBJ_FALSE, OBJ_NONE, OBJ_TRUE};

use crate::{Object, OpRes, RuntimeErrMsg, RuntimeResult, VM};

/// Gets the operand of an instruction.
macro_rules! operand {
  ($s:ident, $is_long:expr) => {{
    if $is_long {
      $s.next_short() as usize
    } else {
      $s.next_byte() as usize
    }
  }};
}

#[allow(clippy::unit_arg)]
impl VM {
  /// The execution loop that dispatches each operations for each instruction in the current chunk.
  pub fn run(&mut self) -> RuntimeResult {
    loop {
      let exec = match self.next_byte().into() {
        // Object loaders
        OpCode::LoadConstant => self.op_load_constant_or_long(false),
        OpCode::LoadConstantLong => self.op_load_constant_or_long(true),
        OpCode::LoadImm0F => self.op_load_imm0f(),
        OpCode::LoadImm0I => self.op_load_imm0i(),
        OpCode::LoadImm1F => self.op_load_imm1f(),
        OpCode::LoadImm1I => self.op_load_imm1i(),
        OpCode::LoadImmFalse => self.op_load_imm_false(),
        OpCode::LoadImmN => self.op_load_imm_n_or_long(false),
        OpCode::LoadImmNLong => self.op_load_imm_n_or_long(true),
        OpCode::LoadImmNone => self.op_load_imm_none(),
        OpCode::LoadImmTrue => self.op_load_imm_true(),
        OpCode::LoadNative => self.op_load_native_or_long(false),
        OpCode::LoadNativeLong => self.op_load_native_or_long(true),
        OpCode::LoadPrimitive => self.op_load_primitive(),

        // Operators
        OpCode::Add => self.op_binary_expression(BinaryExprKind::Add),
        OpCode::BinaryIn => self.op_binary_expression(BinaryExprKind::In),
        OpCode::BitwiseAnd => self.op_binary_expression(BinaryExprKind::BitAND),
        OpCode::BitwiseNot => self.op_bitwise_not(),
        OpCode::BitwiseOr => self.op_binary_expression(BinaryExprKind::BitOR),
        OpCode::BitwiseShiftLeft => self.op_binary_expression(BinaryExprKind::BitShiftLeft),
        OpCode::BitwiseShiftRight => self.op_binary_expression(BinaryExprKind::BitShiftRight),
        OpCode::BitwiseXor => self.op_binary_expression(BinaryExprKind::BitXOR),
        OpCode::Divide => self.op_binary_expression(BinaryExprKind::Div),
        OpCode::Equals => self.op_binary_expression(BinaryExprKind::Equals),
        OpCode::GreaterThan => self.op_binary_expression(BinaryExprKind::GreaterThan),
        OpCode::GreaterThanEq => self.op_binary_expression(BinaryExprKind::GreaterThanEQ),
        OpCode::LessThan => self.op_binary_expression(BinaryExprKind::LessThan),
        OpCode::LessThanEq => self.op_binary_expression(BinaryExprKind::LessThanEQ),
        OpCode::LogicNot => self.op_logic_not(),
        OpCode::Modulus => self.op_binary_expression(BinaryExprKind::Mod),
        OpCode::Multiply => self.op_binary_expression(BinaryExprKind::Mul),
        OpCode::Negate => self.op_negate(),
        OpCode::Nonish => self.op_binary_expression(BinaryExprKind::Nonish),
        OpCode::NotEq => self.op_binary_expression(BinaryExprKind::NotEquals),
        OpCode::Pow => self.op_binary_expression(BinaryExprKind::Pow),
        OpCode::Subscript => self.op_subscript(),
        OpCode::SubscriptAssign => self.op_subscript_assign(),
        OpCode::Subtract => self.op_binary_expression(BinaryExprKind::Subtract),

        // Declarations
        OpCode::DefineGlobal => self.op_define_global(),
        OpCode::GetGlobal => self.op_get_global_or_long(false),
        OpCode::GetGlobalLong => self.op_get_global_or_long(true),
        OpCode::GetLocal => self.op_get_local_or_long(false),
        OpCode::GetLocalLong => self.op_get_local_or_long(true),
        OpCode::SetGlobal => self.op_set_global_or_long(false),
        OpCode::SetGlobalLong => self.op_set_global_or_long(true),
        OpCode::SetLocal => self.op_set_local_or_long(false),
        OpCode::SetLocalLong => self.op_set_local_or_long(true),
        OpCode::SetProp => self.op_set_prop(),
        OpCode::UnpackAssign => self.op_unpack_assign(),
        OpCode::UnpackAssignLong => self.op_unpack_assign_long(),
        OpCode::UnpackIgnore => self.op_unpack_ignore(),
        OpCode::UnpackIgnoreLong => self.op_unpack_ignore_long(),
        OpCode::UnpackSeq => self.op_unpack_seq(),
        OpCode::UnpackSeqLong => self.op_unpack_seq_long(),

        // Object makers/builders
        OpCode::BuildStr => self.op_build_str_or_long(false),
        OpCode::BuildStrLong => self.op_build_str_or_long(true),
        OpCode::MakeArray => self.op_make_array_or_long(false),
        OpCode::MakeArrayLong => self.op_make_array_or_long(true),
        OpCode::MakeArrayRepeat => self.op_make_array_repeat(),
        OpCode::MakeClass => self.op_make_class(),
        OpCode::MakeDict => self.op_make_dict(),
        OpCode::MakeDictLong => self.op_make_dict_long(),
        OpCode::MakeInstance => self.op_make_instance(),
        OpCode::MakeIter => self.op_make_iter(),
        OpCode::MakeRange => self.op_make_range(),
        OpCode::MakeRangeEq => self.op_make_range_eq(),
        OpCode::MakeTuple => self.op_make_tuple_or_long(false),
        OpCode::MakeTupleLong => self.op_make_tuple_or_long(true),
        OpCode::MakeTupleRepeat => self.op_make_tuple_repeat(),

        // Control Flow and Jumps
        OpCode::ForIterNextOrJump => self.op_for_iter_next_or_jump(),
        OpCode::IfFalsePopJump => self.op_if_false_pop_jump(),
        OpCode::JumpForward => self.op_jump_forward(),
        OpCode::JumpIfFalseOrPop => self.op_jump_if_false_or_pop(),
        OpCode::JumpIfTrueOrPop => self.op_jump_if_true_or_pop(),
        OpCode::LoopJump => self.op_loop_jump(),
        OpCode::PopJumpIfFalse => self.op_pop_then_jump_if_false(),

        // Functions and Closures
        OpCode::BindDefaults | OpCode::BindDefaultsLong => self.op_bind_defaults_or_long(),
        OpCode::CloseUpVal => self.op_close_up_val(),
        OpCode::CloseUpValLong => self.op_close_up_val_long(),
        OpCode::FuncCall => self.op_func_call_or_long(false),
        OpCode::FuncCallLong => self.op_func_call_or_long(true),
        OpCode::GetUpVal => self.op_get_up_val(),
        OpCode::GetUpValLong => self.op_get_up_val_long(),
        OpCode::MakeClosure => self.op_make_closure(),
        OpCode::MakeClosureLarge => self.op_make_closure_large(),
        OpCode::MakeClosureLong => self.op_make_closure_long(),
        OpCode::MakeClosureLongLarge => self.op_make_closure_long_large(),
        OpCode::PopCloseUpVal => self.op_pop_close_up_val(),
        OpCode::Return => self.op_return(),
        OpCode::SetUpVal => self.op_set_up_val(),
        OpCode::SetUpValLong => self.op_set_up_val_long(),

        // Classes & Instances
        OpCode::AppendClassField => self.op_append_class_field(),
        OpCode::GetProp => self.op_get_prop(),
        OpCode::GetPropLong => self.op_get_prop_long(),
        OpCode::MakeClassLong => self.op_make_class_long(),
        OpCode::SetPropLong => self.op_set_prop_long(),

        // Direct stack manipulation
        OpCode::DupTop => self.op_dup_top(),
        OpCode::DupTopN => self.op_dup_top_n_or_long(false),
        OpCode::DupTopNLong => self.op_dup_top_n_or_long(true),
        OpCode::DupTopTwo => self.op_dup_top_two(),
        OpCode::PopStackTop => self.op_pop_stack_top(),
        OpCode::PopStackTopN => self.op_pop_stack_top_n_or_long(false),
        OpCode::PopStackTopNLong => self.op_pop_stack_top_n_or_long(true),
        OpCode::RotateTopN => self.op_rotate_top_n_or_long(false),
        OpCode::RotateTopNLong => self.op_rotate_top_n_or_long(true),

        // Others
        OpCode::EndVirtualMachine => self.op_end_virtual_machine(),
      };

      match exec {
        OpRes::Continue(_) => continue,
        OpRes::Break(r) => return r,
      }
    }
  }

  fn op_load_constant_or_long(&mut self, is_long: bool) -> OpRes {
    let pos = operand![self, is_long];
    let val = self.constants[pos];
    ControlFlow::Continue(self.stack.push(val))
  }

  fn op_load_imm0f(&mut self) -> OpRes {
    ControlFlow::Continue(self.stack.push(0f64.into()))
  }

  fn op_load_imm0i(&mut self) -> OpRes {
    ControlFlow::Continue(self.stack.push(0i64.into()))
  }

  fn op_load_imm1f(&mut self) -> OpRes {
    ControlFlow::Continue(self.stack.push(1f64.into()))
  }

  fn op_load_imm1i(&mut self) -> OpRes {
    ControlFlow::Continue(self.stack.push(1i64.into()))
  }

  fn op_load_imm_false(&mut self) -> OpRes {
    ControlFlow::Continue(self.stack.push(OBJ_FALSE))
  }

  fn op_load_imm_n_or_long(&mut self, is_long: bool) -> OpRes {
    let imm = operand![self, is_long] as i64;
    ControlFlow::Continue(self.stack.push(imm.into()))
  }

  fn op_load_imm_none(&mut self) -> OpRes {
    ControlFlow::Continue(self.stack.push(OBJ_NONE))
  }

  fn op_load_imm_true(&mut self) -> OpRes {
    ControlFlow::Continue(self.stack.push(OBJ_TRUE))
  }

  fn op_load_native_or_long(&mut self, is_long: bool) -> OpRes {
    let native_idx = operand![self, is_long] as usize;
    ControlFlow::Continue(self.stack.push(Object::NativeFunc(native_idx.into())))
  }

  fn op_load_primitive(&mut self) -> OpRes {
    todo!()
  }

  fn op_binary_expression(&mut self, operator: BinaryExprKind) -> OpRes {
    let right = &self.pop_stack();
    let left = &self.pop_stack();

    let result = match operator {
      BinaryExprKind::Add => left.add(right, &mut self.gc),
      BinaryExprKind::BitAND => left.bit_and(right),
      BinaryExprKind::BitOR => left.bit_or(right),
      BinaryExprKind::BitShiftLeft => left.shl(right),
      BinaryExprKind::BitShiftRight => left.shr(right),
      BinaryExprKind::BitXOR => left.bit_xor(right),
      BinaryExprKind::Div => left.div(right),
      BinaryExprKind::Pow => left.pow(right),
      BinaryExprKind::Equals => Ok((left.equals(right, &self.gc)).into()),
      BinaryExprKind::GreaterThan => left.gt(right),
      BinaryExprKind::GreaterThanEQ => left.gt_eq(right),
      BinaryExprKind::LessThan => left.lt(right),
      BinaryExprKind::LessThanEQ => left.lt_eq(right),
      BinaryExprKind::NotEquals => Ok((!left.equals(right, &self.gc)).into()),
      BinaryExprKind::Subtract => left.sub(right),
      BinaryExprKind::Mod => left.rem(right),
      BinaryExprKind::Mul => left.mul(right, &mut self.gc),
      BinaryExprKind::In => left.is_in(right, &self.gc),
      BinaryExprKind::InstOf => todo!(),
      BinaryExprKind::MatMul => todo!(),
      BinaryExprKind::Pipe => todo!(),
      BinaryExprKind::Range => todo!(),
      BinaryExprKind::RangeEQ => todo!(),
      BinaryExprKind::Nonish => {
        if matches!(left, Object::None) {
          Ok(*right)
        } else {
          Ok(*left)
        }
      }
      _ => unreachable!("The other binary operations have special instruction methods."),
    };

    match result {
      Ok(r) => ControlFlow::Continue(self.stack.push(r)),
      Err(e) => ControlFlow::Break(Err(e)),
    }
  }

  fn op_bitwise_not(&mut self) -> OpRes {
    todo!()
  }

  fn op_logic_not(&mut self) -> OpRes {
    todo!()
  }

  fn op_negate(&mut self) -> OpRes {
    let operand = self.pop_stack();

    match operand.neg() {
      Ok(r) => ControlFlow::Continue(self.stack.push(r)),
      Err(e) => ControlFlow::Break(Err(e)),
    }
  }

  fn op_subscript(&mut self) -> OpRes {
    let index = self.pop_stack();
    let target = self.pop_stack();

    match target.subscript(&index, &mut self.gc) {
      Ok(r) => ControlFlow::Continue(self.stack.push(r)),
      Err(e) => ControlFlow::Break(Err(e)),
    }
  }

  fn op_subscript_assign(&mut self) -> OpRes {
    let value = self.pop_stack();
    let index = self.pop_stack();
    let target = self.pop_stack();

    match target {
      Object::Array(id) => match self.gc.get_mut(&id).as_array_obj_mut().unwrap().assign_at(index, value) {
        Ok(res) => ControlFlow::Continue(self.stack.push(res)),
        Err(e) => ControlFlow::Break(Err(e)),
      },
      _ => {
        let err_msg = format!("Objects of type '{}' are not subscriptable.", target.type_name());
        ControlFlow::Break(Err(RuntimeErrMsg::Type(err_msg)))
      }
    }
  }

  fn op_define_global(&mut self) -> OpRes {
    let val = self.pop_stack();
    self.globals.push(val);
    ControlFlow::Continue(())
  }

  fn op_get_global_or_long(&mut self, is_long: bool) -> OpRes {
    let pos = operand![self, is_long];
    let val = self.globals[pos];
    ControlFlow::Continue(self.stack.push(val))
  }

  fn op_get_local_or_long(&mut self, is_long: bool) -> OpRes {
    let pos = operand![self, is_long];
    let val = self.stack[self.current_frame().return_idx + pos];
    ControlFlow::Continue(self.stack.push(val))
  }

  fn op_set_global_or_long(&mut self, is_long: bool) -> OpRes {
    let val = *self.peek_stack(0);
    let pos = operand![self, is_long];
    self.globals[pos] = val;
    ControlFlow::Continue(())
  }

  fn op_set_local_or_long(&mut self, is_long: bool) -> OpRes {
    let val = *self.peek_stack(0);
    let pos = operand![self, is_long];
    self.stack[pos] = val;
    ControlFlow::Continue(())
  }

  fn op_set_prop(&mut self) -> OpRes {
    todo!()
  }

  fn op_unpack_assign(&mut self) -> OpRes {
    todo!()
  }

  fn op_unpack_assign_long(&mut self) -> OpRes {
    todo!()
  }

  fn op_unpack_ignore(&mut self) -> OpRes {
    todo!()
  }

  fn op_unpack_ignore_long(&mut self) -> OpRes {
    todo!()
  }

  fn op_unpack_seq(&mut self) -> OpRes {
    todo!()
  }

  fn op_unpack_seq_long(&mut self) -> OpRes {
    todo!()
  }

  fn op_build_str_or_long(&mut self, is_long: bool) -> OpRes {
    let stack_len = self.stack.len();
    let parts_count = operand![self, is_long];
    let objs = self.stack.drain((stack_len - parts_count)..stack_len).collect::<Vec<Object>>();

    let mut new_str = String::new();
    for o in objs {
      new_str += &*o.display_plain(&self.gc)
    }

    let s = self.gc.push(new_str.into());
    ControlFlow::Continue(self.stack.push(Object::Str(s)))
  }

  fn op_make_array_or_long(&mut self, is_long: bool) -> OpRes {
    let count = operand![self, is_long];
    let stack_len = self.stack.len();
    let objs = self.stack.drain((stack_len - count)..stack_len).collect::<Vec<Object>>();
    let a = self.gc.push(GcObject::Array(ArrayObj(objs)));
    ControlFlow::Continue(self.stack.push(Object::Array(a)))
  }

  fn op_make_array_repeat(&mut self) -> OpRes {
    let count = self.pop_stack();
    let obj = self.pop_stack();

    if let Some(count) = count.as_int() {
      let a = self.gc.push(GcObject::Array(ArrayObj(vec![obj; count as usize])));
      ControlFlow::Continue(self.stack.push(Object::Array(a)))
    } else {
      ControlFlow::Break(Err(RuntimeErrMsg::Type("Expected an integer.".into())))
    }
  }

  fn op_make_class(&mut self) -> OpRes {
    todo!()
  }

  fn op_make_dict(&mut self) -> OpRes {
    todo!()
  }

  fn op_make_dict_long(&mut self) -> OpRes {
    todo!()
  }

  fn op_make_instance(&mut self) -> OpRes {
    todo!()
  }

  fn op_make_iter(&mut self) -> OpRes {
    todo!()
  }

  fn op_make_range(&mut self) -> OpRes {
    todo!()
  }

  fn op_make_range_eq(&mut self) -> OpRes {
    todo!()
  }

  fn op_make_tuple_or_long(&mut self, _: bool) -> OpRes {
    todo!()
  }

  fn op_make_tuple_repeat(&mut self) -> OpRes {
    todo!()
  }

  fn op_for_iter_next_or_jump(&mut self) -> OpRes {
    todo!()
  }

  fn op_if_false_pop_jump(&mut self) -> OpRes {
    let jump = self.next_short() as usize;

    if self.peek_stack(0).is_falsy() {
      self.pop_stack();
      self.current_frame_mut().ip += jump;
    }

    ControlFlow::Continue(())
  }

  fn op_jump_forward(&mut self) -> OpRes {
    let jump = self.next_short() as usize;
    self.current_frame_mut().ip += jump;
    ControlFlow::Continue(())
  }

  fn op_jump_if_false_or_pop(&mut self) -> OpRes {
    let jump = self.next_short() as usize;

    if self.peek_stack(0).is_falsy() {
      self.current_frame_mut().ip += jump;
    } else {
      self.pop_stack();
    }

    ControlFlow::Continue(())
  }

  fn op_jump_if_true_or_pop(&mut self) -> OpRes {
    let jump = self.next_short() as usize;

    if !self.peek_stack(0).is_falsy() {
      self.current_frame_mut().ip += jump;
    } else {
      self.pop_stack();
    }

    ControlFlow::Continue(())
  }

  fn op_loop_jump(&mut self) -> OpRes {
    let back_jump = self.next_short() as usize;
    self.current_frame_mut().ip -= back_jump;
    ControlFlow::Continue(())
  }

  fn op_pop_then_jump_if_false(&mut self) -> OpRes {
    let jump = self.next_short() as usize;
    self.current_frame_mut().ip += (self.pop_stack().is_falsy() as usize) * jump;
    ControlFlow::Continue(())
  }

  fn op_bind_defaults_or_long(&mut self) -> OpRes {
    todo!()
  }

  fn op_close_up_val(&mut self) -> OpRes {
    todo!()
  }

  fn op_close_up_val_long(&mut self) -> OpRes {
    todo!()
  }

  fn op_func_call_or_long(&mut self, is_long: bool) -> OpRes {
    let args_len = operand![self, is_long];
    self.call_obj(args_len)
  }

  fn op_get_up_val(&mut self) -> OpRes {
    todo!()
  }

  fn op_get_up_val_long(&mut self) -> OpRes {
    todo!()
  }

  fn op_make_closure(&mut self) -> OpRes {
    todo!()
  }

  fn op_make_closure_large(&mut self) -> OpRes {
    todo!()
  }

  fn op_make_closure_long(&mut self) -> OpRes {
    todo!()
  }

  fn op_make_closure_long_large(&mut self) -> OpRes {
    todo!()
  }

  fn op_pop_close_up_val(&mut self) -> OpRes {
    todo!()
  }

  fn op_return(&mut self) -> OpRes {
    let result = self.pop_stack();

    let popped_frame = match self.frames.pop() {
      Some(f) => f,
      None => panic!("Stack Frames underflow."),
    };

    let stack_len = self.stack.len();
    self.stack.drain(popped_frame.return_idx..stack_len);
    self.stack.push(result); // The value returned from the func call
    ControlFlow::Continue(())
  }

  fn op_set_up_val(&mut self) -> OpRes {
    todo!()
  }

  fn op_set_up_val_long(&mut self) -> OpRes {
    todo!()
  }

  fn op_append_class_field(&mut self) -> OpRes {
    todo!()
  }

  fn op_get_prop(&mut self) -> OpRes {
    todo!()
  }

  fn op_get_prop_long(&mut self) -> OpRes {
    todo!()
  }

  fn op_make_class_long(&mut self) -> OpRes {
    todo!()
  }

  fn op_set_prop_long(&mut self) -> OpRes {
    todo!()
  }

  fn op_dup_top(&mut self) -> OpRes {
    let obj = self.peek_stack(0);
    ControlFlow::Continue(self.stack.push(*obj))
  }

  fn op_dup_top_n_or_long(&mut self, is_long: bool) -> OpRes {
    let n = operand![self, is_long];
    let objs = &self.stack[(self.stack.len() - n - 1)..self.stack.len()];
    ControlFlow::Continue(self.stack.append(&mut objs.to_vec()))
  }

  fn op_dup_top_two(&mut self) -> OpRes {
    let obj1 = *self.peek_stack(0);
    let obj2 = *self.peek_stack(1);
    self.stack.push(obj2);
    self.stack.push(obj1);
    ControlFlow::Continue(())
  }

  fn op_pop_stack_top(&mut self) -> OpRes {
    self.pop_stack();
    ControlFlow::Continue(())
  }

  fn op_pop_stack_top_n_or_long(&mut self, is_long: bool) -> OpRes {
    let n = operand![self, is_long];
    let stack_len = self.stack.len();

    if n > stack_len {
      panic!("Attempted to pop {} objects, but the stack's len is {}.", n, stack_len);
    } else {
      self.stack.drain((stack_len - n)..stack_len);
      ControlFlow::Continue(())
    }
  }

  fn op_rotate_top_n_or_long(&mut self, is_long: bool) -> OpRes {
    let n = operand![self, is_long];
    let stack_len = self.stack.len();

    if n > stack_len {
      panic!("Attempted to pop {} objects, but the stack's len is {}.", n, stack_len);
    } else {
      let els: Vec<Object> = self.stack.drain((stack_len - n)..stack_len).rev().collect();
      els.iter().for_each(|e| self.stack.push(*e));
      ControlFlow::Continue(())
    }
  }

  fn op_end_virtual_machine(&mut self) -> OpRes {
    assert![self.stack.is_empty()];
    ControlFlow::Break(Ok(()))
  }
}
