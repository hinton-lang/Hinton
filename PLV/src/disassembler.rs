use std::fmt::Write as FmtWrite;

use core::bytecode::OpCode;
use objects::func_obj::FuncObj;
use objects::gc::GarbageCollector;
use objects::Object;

use crate::PLVJsonGenerator;

impl<'a> PLVJsonGenerator<'a> {
  pub fn disassemble_all(&self, gc: &GarbageCollector) -> String {
    let mut output = String::new();

    for value in self.constants {
      match &value {
        Object::Func(f) => {
          let func = gc.get(f).as_func_obj().unwrap();
          writeln!(output, "{}", self.disassemble_fn(func, gc)).expect("Could not write to output.");
        }
        _ => continue,
      }
    }

    output
  }

  fn disassemble_fn(&self, func: &FuncObj, gc: &GarbageCollector) -> String {
    let mut output = gc.get(&func.name).as_str_obj().unwrap().0.to_owned() + " ------------\n";

    let mut ip = 0;
    let mut out: Vec<(String, String, String, String, String, String, String)> = vec![];
    let mut max_instr_name = 0;
    let mut max_ln_len = 0;
    let mut max_col_len = 0;

    while ip < func.chunk.len() {
      let instr = func.chunk.instructions[ip];
      let loc = self.tokens_list.loc(func.chunk.tokens[ip]);

      let (name, effect, note) = self.describe(func, ip, instr, gc);
      if name.len() > max_instr_name {
        max_instr_name = name.len()
      }
      let line = loc.line_num.to_string();
      if line.len() > max_ln_len {
        max_ln_len = line.len()
      }
      let col = loc.col_start().to_string();
      if col.len() > max_col_len {
        max_col_len = col.len()
      }
      let instr_ptr = format!("{:0>5}", ip);
      let hex_instr = format!("{:#04x}", instr);
      let operand = match effect {
        0 => "".to_string(),
        1 => format!("--> {}", func.chunk.instructions[ip + 1]),
        2 => format!("--> {}", func.chunk.get_short(ip + 1)),
        _ => todo!("N-Operand Instructions"),
      };
      let note = if let Some(s) = note { s } else { "".into() };
      out.push((line, col, instr_ptr, hex_instr, name.into(), operand, note));

      ip += effect as usize + 1;
    }

    for line in out {
      writeln!(
        &mut output,
        "[{:w1$}:{:w2$}] {} {} {:w3$} {} {}",
        line.0,
        line.1,
        line.2,
        line.3,
        line.4,
        line.5,
        line.6,
        w1 = max_ln_len,
        w2 = max_col_len,
        w3 = max_instr_name + 2
      )
      .expect("Could not write to output.");
    }

    output
  }

  fn describe(&self, func: &FuncObj, ip: usize, instr: u8, gc: &GarbageCollector) -> (&str, u8, Option<String>) {
    match OpCode::from(instr) {
      OpCode::Add => ("ADD", 0, None),
      OpCode::BitwiseAnd => ("BITWISE_AND", 0, None),
      OpCode::BitwiseNot => ("BITWISE_NOT", 0, None),
      OpCode::BitwiseOr => ("BITWISE_OR", 0, None),
      OpCode::BitwiseShiftLeft => ("BITWISE_SHIFT_LEFT", 0, None),
      OpCode::BitwiseShiftRight => ("BITWISE_SHIFT_RIGHT", 0, None),
      OpCode::BitwiseXor => ("BITWISE_XOR", 0, None),
      OpCode::DefineGlobal => ("DEFINE_GLOBAL", 0, None),
      OpCode::Divide => ("DIVIDE", 0, None),
      OpCode::EndVirtualMachine => ("END_VM", 0, None),
      OpCode::Equals => ("EQUALS", 0, None),
      OpCode::GreaterThan => ("GREATER_THAN", 0, None),
      OpCode::GreaterThanEq => ("GREATER_THAN_EQ", 0, None),
      OpCode::LessThan => ("LESS_THAN", 0, None),
      OpCode::LessThanEq => ("LESS_THAN_EQ", 0, None),
      OpCode::LoadImm0F => ("LOAD_IMM_0F", 0, None),
      OpCode::LoadImm0I => ("LOAD_IMM_0I", 0, None),
      OpCode::LoadImm1F => ("LOAD_IMM_1F", 0, None),
      OpCode::LoadImm1I => ("LOAD_IMM_1I", 0, None),
      OpCode::LoadImmFalse => ("LOAD_IMM_FALSE", 0, None),
      OpCode::LoadImmNone => ("LOAD_IMM_NONE", 0, None),
      OpCode::LoadImmTrue => ("LOAD_IMM_TRUE", 0, None),
      OpCode::LogicNot => ("LOGIC_NOT", 0, None),
      OpCode::MakeArrayRepeat => ("MAKE_ARRAY_REPEAT", 0, None),
      OpCode::MakeIter => ("MAKE_ITER", 0, None),
      OpCode::MakeRange => ("MAKE_RANGE", 0, None),
      OpCode::MakeRangeEq => ("MAKE_RANGE_EQ", 0, None),
      OpCode::MakeTupleRepeat => ("MAKE_TUPLE_REPEAT", 0, None),
      OpCode::Modulus => ("MODULUS", 0, None),
      OpCode::Multiply => ("MULTIPLY", 0, None),
      OpCode::Negate => ("NEGATE", 0, None),
      OpCode::Nonish => ("NONISH", 0, None),
      OpCode::NotEq => ("NOT_EQ", 0, None),
      OpCode::PopCloseUpVal => ("POP_CLOSE_UP_VAL", 0, None),
      OpCode::PopStackTop => ("POP_STACK_TOP", 0, None),
      OpCode::Pow => ("POW", 0, None),
      OpCode::Return => ("RETURN", 0, None),
      OpCode::Subscript => ("SUBSCRIPT", 0, None),
      OpCode::SubscriptAssign => ("SUBSCRIPT_ASSIGN", 0, None),
      OpCode::Subtract => ("SUBTRACT", 0, None),

      // 1-operand instructions
      OpCode::AppendClassField => ("APPEND_CLASS_FIELD", 1, None),
      OpCode::BindDefaults => ("BIND_DEFAULTS", 1, None),
      OpCode::BuildStr => ("BUILD_STR", 1, None),
      OpCode::CloseUpVal => ("CLOSE_UP_VAL", 1, None),
      OpCode::FuncCall => ("FUNC_CALL", 1, None),
      OpCode::GetGlobal => ("GET_GLOBAL", 1, None),
      OpCode::GetLocal => ("GET_LOCAL", 1, None),
      OpCode::GetProp => ("GET_PROP", 1, None),
      OpCode::GetUpVal => ("GET_UP_VAL", 1, None),
      OpCode::LoadConstant => ("LOAD_CONSTANT", 1, self.fmt_const_val(func, ip, false, gc)),
      OpCode::LoadImmN => ("LOAD_IMM_N", 1, None),
      OpCode::LoadNative => ("LOAD_NATIVE", 1, None),
      OpCode::LoadPrimitive => ("LOAD_PRIMITIVE", 1, None),
      OpCode::MakeArray => ("MAKE_ARRAY", 1, None),
      OpCode::MakeClass => ("MAKE_CLASS", 1, None),
      OpCode::MakeDict => ("MAKE_DICT", 1, None),
      OpCode::MakeInstance => ("MAKE_INSTANCE", 1, None),
      OpCode::MakeTuple => ("MAKE_TUPLE", 1, None),
      OpCode::PopStackTopN => ("POP_STACK_TOP_N", 1, None),
      OpCode::RotateTopN => ("ROTATE_TOP_N", 1, None),
      OpCode::SetGlobal => ("SET_GLOBAL", 1, None),
      OpCode::SetLocal => ("SET_LOCAL", 1, None),
      OpCode::SetProp => ("SET_PROP", 1, None),
      OpCode::SetUpVal => ("SET_UP_VAL", 1, None),
      OpCode::UnpackSeq => ("UNPACK", 1, None),

      // 2-operand instructions
      OpCode::BindDefaultsLong => ("BIND_DEFAULTS_LONG", 2, None),
      OpCode::BuildStrLong => ("BUILD_STR_LONG", 2, None),
      OpCode::CloseUpValLong => ("CLOSE_UP_VAL_LONG", 2, None),
      OpCode::ForIterNextOrJump => ("FOR_ITER_NEXT_OR_JUMP", 2, None),
      OpCode::FuncCallLong => ("FUNC_CALL_LONG", 2, None),
      OpCode::GetGlobalLong => ("GET_GLOBAL_LONG", 2, None),
      OpCode::GetLocalLong => ("GET_LOCAL_LONG", 2, None),
      OpCode::GetPropLong => ("GET_PROP_LONG", 2, None),
      OpCode::GetUpValLong => ("GET_UP_VAL_LONG", 2, None),
      OpCode::IfFalsePopJump => ("IF_FALSE_POP_JUMP", 2, None),
      OpCode::JumpForward => ("JUMP_FORWARD", 2, None),
      OpCode::JumpIfFalseOrPop => ("JUMP_IF_FALSE_OR_POP", 2, None),
      OpCode::JumpIfTrueOrPop => ("JUMP_IF_TRUE_OR_POP", 2, None),
      OpCode::LoadConstantLong => ("LOAD_CONSTANT_LONG", 2, self.fmt_const_val(func, ip, true, gc)),
      OpCode::LoadImmNLong => ("LOAD_IMM_N_LONG", 2, None),
      OpCode::LoadNativeLong => ("LOAD_NATIVE_LONG", 2, None),
      OpCode::LoopJump => ("LOOP_JUMP", 2, None),
      OpCode::MakeArrayLong => ("MAKE_ARRAY_LONG", 2, None),
      OpCode::MakeClassLong => ("MAKE_CLASS_LONG", 2, None),
      OpCode::MakeDictLong => ("MAKE_DICT_LONG", 2, None),
      OpCode::MakeTupleLong => ("MAKE_TUPLE_LONG", 2, None),
      OpCode::PopJumpIfFalse => ("POP_JUMP_IF_FALSE", 2, None),
      OpCode::PopStackTopNLong => ("POP_STACK_TOP_N_LONG", 1, None),
      OpCode::RotateTopNLong => ("ROTATE_TOP_N_LONG", 2, None),
      OpCode::SetGlobalLong => ("SET_GLOBAL_LONG", 2, None),
      OpCode::SetLocalLong => ("SET_LOCAL_LONG", 2, None),
      OpCode::SetPropLong => ("SET_PROP_LONG", 2, None),
      OpCode::SetUpValLong => ("SET_UP_VAL_LONG", 2, None),
      OpCode::UnpackAssign => ("UNPACK_ASSIGN", 2, None),
      OpCode::UnpackIgnore => ("UNPACK_IGNORE", 2, None),
      OpCode::UnpackSeqLong => ("UNPACK_SEQ_LONG", 2, None),

      // 4-operand instructions
      OpCode::UnpackAssignLong => ("UNPACK_ASSIGN_LONG", 4, None),
      OpCode::UnpackIgnoreLong => ("UNPACK_IGNORE_LONG", 4, None),

      // N-operand instructions
      OpCode::MakeClosure => ("MAKE_CLOSURE", 0, None),
      OpCode::MakeClosureLong => ("MAKE_CLOSURE_LONG", 0, None),
      OpCode::MakeClosureLarge => ("MAKE_CLOSURE_LARGE", 0, None),
      OpCode::MakeClosureLongLarge => ("MAKE_CLOSURE_LONG_LARGE", 0, None),
    }
  }

  fn fmt_const_val(&self, func: &FuncObj, idx: usize, is_long: bool, gc: &GarbageCollector) -> Option<String> {
    let pos = if is_long {
      func.chunk.get_short(idx + 1) as usize
    } else {
      func.chunk.instructions[idx + 1] as usize
    };

    Some(self.constants[pos].debug_fmt(gc))
  }
}
