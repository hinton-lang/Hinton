/// The set of instructions supported by the virtual machine.
///
/// **NOTE:** Changing the order in which members are declared creates
/// incompatibilities between different versions of the interpreter.
#[derive(Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum OpCode {
  // Instructions with zero chunk operands.
  Add,
  BitwiseAnd,
  BitwiseNot,
  BitwiseOr,
  BitwiseShiftLeft,
  BitwiseShiftRight,
  BitwiseXor,
  Divide,
  EndVirtualMachine,
  Equals,
  GreaterThan,
  GreaterThanEq,
  LessThan,
  LessThanEq,
  LoadImm0F,
  LoadImm0I,
  LoadImm1F,
  LoadImm1I,
  LoadImmFalse,
  LoadImmNone,
  LoadImmTrue,
  LogicNot,
  MakeArrayRepeat,
  MakeIter,
  MakeRange,
  MakeRangeEq,
  MakeTupleRepeat,
  Modulus,
  Multiply,
  Negate,
  Nonish,
  NotEq,
  PopCloseUpVal,
  PopStackTop,
  Pow,
  Return,
  Subscript,
  SubscriptAssign,
  Subtract,
  TypeOf,

  // Instructions with one chunk operands.
  // These instructions use the next byte from the chunk as its operand.
  AppendClassField,
  BindDefaults,
  CloseUpVal,
  DefineGlobal,
  FuncCall,
  GetGlobal,
  GetLocal,
  GetProp,
  GetUpVal,
  LoadConstant,
  LoadImmN,
  LoadNative,
  LoadPrimitive,
  LoopJump,
  MakeArray,
  MakeClass,
  MakeDict,
  MakeInstance,
  MakeTuple,
  SetGlobal,
  SetLocal,
  SetProp,
  SetUpVal,

  // Instructions with two chunk operands.
  // These instructions use the next two bytes (a short) as their operands.
  CloseUpValLong,
  DefineGlobalLong,
  ForIterNextOrJump,
  GetGlobalLong,
  GetLocalLong,
  GetPropLong,
  GetUpValLong,
  JumpForward,
  JumpIfFalseOrPop,
  JumpIfTrueOrPop,
  LoadConstantLong,
  LoadImmNLong,
  LoopJumpLong,
  MakeArrayLong,
  MakeClassLong,
  MakeDictLong,
  MakeTupleLong,
  PopJumpIfFalse,
  SetGlobalLong,
  SetLocalLong,
  SetPropLong,
  SetUpValLong,

  // Instructions with a variable number of instructions.
  MakeClosure,
  // Byte #1 is the position of the function object in the pool.
  // --- UpValue Encoding (2 bytes per up_value) ---
  // One byte if up value is local
  // One byte for the position of the up value
  MakeClosureLong,
  // Byte #1 and Byte #2 are the position of the function object in the pool.
  // --- UpValue Encoding (2 bytes per up_value) ---
  // One byte if up value is local
  // One byte for the position of the up value
  MakeClosureLarge,
  // Byte #1 is the position of the function object in the pool.
  // --- UpValue Encoding (3 bytes per up_value) ---
  // One byte if up value is local
  // Two bytes for the position of the up value
  MakeClosureLongLarge,
  // Byte #1 and Byte #2 are the position of the function object in the pool.
  // --- UpValue Encoding (3 bytes per up_value) ---
  // One byte if up value is local
  // Two bytes for the position of the up value
}

impl From<u8> for OpCode {
  fn from(byte: u8) -> Self {
    unsafe { std::mem::transmute(byte) }
  }
}
