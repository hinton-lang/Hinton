use super::{RuntimeErrorType, RuntimeResult, VirtualMachine};
use crate::{
    chunk::OpCode,
    natives::{get_next_in_iter, iter_has_next, make_iter},
    objects::{Object, RangeObject},
};

impl<'a> VirtualMachine {
    /// Executes the instruction in a chunk. This is where the interpreter
    /// will spend most its life, therefore, optimizing every function in this file
    /// is very important.
    ///
    /// ## Returns
    /// `InterpretResult` – The result of the execution.
    pub(crate) fn run(&mut self) -> RuntimeResult {
        while let Some(instruction) = self.get_next_op_code() {
            match instruction {
                OpCode::PopStack1 => {
                    self.pop_stack();
                }

                OpCode::PopStackN | OpCode::PopStackNLong => {
                    let n = if let OpCode::PopStackN = instruction {
                        self.get_next_byte().unwrap() as i64
                    } else {
                        self.get_next_short().unwrap() as i64
                    };

                    for _ in 0..n {
                        self.pop_stack();
                    }
                }

                OpCode::LoadImmNull => self.push_stack(Object::Null),
                OpCode::LoadImmTrue => self.push_stack(Object::Bool(true)),
                OpCode::LoadImmFalse => self.push_stack(Object::Bool(false)),
                OpCode::LoadImm0F => self.push_stack(Object::Float(0f64)),
                OpCode::LoadImm0I => self.push_stack(Object::Int(0i64)),
                OpCode::LoadImm1F => self.push_stack(Object::Float(1f64)),
                OpCode::LoadImm1I => self.push_stack(Object::Int(1i64)),

                OpCode::LoadImmN | OpCode::LoadImmNLong => {
                    let imm = if let OpCode::LoadImmN = instruction {
                        self.get_next_byte().unwrap() as i64
                    } else {
                        self.get_next_short().unwrap() as i64
                    };

                    self.push_stack(Object::Int(imm))
                }

                OpCode::LoadConstant | OpCode::LoadConstantLong => {
                    // Either gets the next byte or the next short based on the instruction
                    // The compiler makes sure that the structure of the bytecode is correct
                    // for the VM to execute, so unwrapping without check should be fine.
                    let pos = if let OpCode::LoadConstant = instruction {
                        self.get_next_byte().unwrap() as usize
                    } else {
                        self.get_next_short().unwrap() as usize
                    };

                    // Gets the value from the pool and places it on top of the stack
                    let val = self.read_constant(pos).clone();
                    self.push_stack(val)
                }

                OpCode::MakeIter => {
                    let tos = self.pop_stack();

                    match make_iter(tos) {
                        Ok(iter) => self.push_stack(iter),
                        Err(e) => return e,
                    }
                }

                OpCode::ForLoopIterNext => {
                    let tos = self.peek_stack(self.stack.len() - 1);

                    match tos {
                        Object::Iterable(iter) => match get_next_in_iter(iter) {
                            Ok(o) => self.push_stack(o),
                            Err(_) => unreachable!("No more items to iterate in for-loop"),
                        },
                        _ => unreachable!("Cannot iterate object of type '{}'.", tos.type_name()),
                    }
                }

                OpCode::JumpHasNextOrPop | OpCode::JumpHasNextOrPopLong => {
                    self.pop_stack(); // pop the current iterator item off the stack

                    let jump = if let OpCode::JumpHasNextOrPop = instruction {
                        self.get_next_byte().unwrap() as usize
                    } else {
                        self.get_next_short().unwrap() as usize
                    };

                    match self.peek_stack(self.stack.len() - 1) {
                        Object::Iterable(o) => {
                            if iter_has_next(o) {
                                self.current_frame_mut().ip -= jump;
                            } else {
                                self.pop_stack();
                            }
                        }
                        _ => unreachable!("Expected iterable object on TOS."),
                    };
                }

                OpCode::MakeArray | OpCode::MakeArrayLong => {
                    // The number of values to pop from the stack. Essentially the size of the array.
                    let size = if let OpCode::MakeArray = instruction {
                        self.get_next_byte().unwrap() as usize
                    } else {
                        self.get_next_short().unwrap() as usize
                    };

                    let mut arr_values: Vec<Object> = vec![];

                    for _ in 0..size {
                        arr_values.push(self.pop_stack());
                    }

                    self.push_stack(Object::Array(arr_values));
                }

                OpCode::MakeTuple | OpCode::MakeTupleLong => {
                    // The number of values to pop from the stack. Essentially the size of the array.
                    let size = if let OpCode::MakeTuple = instruction {
                        self.get_next_byte().unwrap() as usize
                    } else {
                        self.get_next_short().unwrap() as usize
                    };

                    let mut tuple_values: Vec<Object> = Vec::with_capacity(size);

                    for _ in 0..size {
                        tuple_values.push(self.pop_stack());
                    }

                    self.push_stack(Object::Tuple(tuple_values));
                }

                OpCode::Indexing => {
                    let index = self.pop_stack();
                    let target = self.pop_stack();

                    match target.get_at_index(&index) {
                        Ok(r) => self.push_stack(r),
                        Err(e) => return e.to_runtime_error(),
                    }
                }

                OpCode::GetVar | OpCode::GetVarLong => {
                    // The position of the local variable's value in the stack
                    let pos = if let OpCode::GetVar = instruction {
                        self.get_next_byte().unwrap() as usize
                    } else {
                        self.get_next_short().unwrap() as usize
                    };

                    let idx = self.current_frame().base_pointer + pos;
                    let value = self.peek_stack(idx).clone();
                    self.push_stack(value);
                }

                OpCode::SetVar | OpCode::SetVarLong => {
                    // The position of the local variable's value in the stack
                    let pos = if let OpCode::SetVar = instruction {
                        self.get_next_byte().unwrap() as usize
                    } else {
                        self.get_next_short().unwrap() as usize
                    };

                    let value = self.stack.last().unwrap();
                    let offset = self.current_frame().base_pointer;

                    self.stack[pos + offset] = value.clone();
                }

                OpCode::Negate => match -self.pop_stack() {
                    Ok(r) => self.push_stack(r),
                    Err(e) => return e.to_runtime_error(),
                },

                OpCode::Add => {
                    let val2 = self.pop_stack();
                    let val1 = self.pop_stack();

                    match val1 + val2 {
                        Ok(r) => self.push_stack(r),
                        Err(e) => return e.to_runtime_error(),
                    }
                }

                OpCode::Subtract => {
                    let val2 = self.pop_stack();
                    let val1 = self.pop_stack();

                    match val1 - val2 {
                        Ok(r) => self.push_stack(r),
                        Err(e) => return e.to_runtime_error(),
                    }
                }

                OpCode::Multiply => {
                    let val2 = self.pop_stack();
                    let val1 = self.pop_stack();

                    match val1 * val2 {
                        Ok(r) => self.push_stack(r),
                        Err(e) => return e.to_runtime_error(),
                    }
                }

                OpCode::Divide => {
                    let val2 = self.pop_stack();
                    let val1 = self.pop_stack();

                    match val1 / val2 {
                        Ok(r) => self.push_stack(r),
                        Err(e) => return e.to_runtime_error(),
                    }
                }

                OpCode::Modulus => {
                    let val2 = self.pop_stack();
                    let val1 = self.pop_stack();

                    match val1 % val2 {
                        Ok(r) => self.push_stack(r),
                        Err(e) => return e.to_runtime_error(),
                    }
                }

                OpCode::Expo => {
                    let val2 = self.pop_stack();
                    let val1 = self.pop_stack();

                    match val1.pow(val2) {
                        Ok(r) => self.push_stack(r),
                        Err(e) => return e.to_runtime_error(),
                    }
                }

                OpCode::LogicNot => {
                    let val = self.pop_stack();
                    self.push_stack(Object::Bool(val.is_falsey()));
                }

                OpCode::Equals => {
                    let val2 = self.pop_stack();
                    let val1 = self.pop_stack();
                    self.push_stack(Object::Bool(val1.equals(&val2)));
                }

                OpCode::NotEq => {
                    let val2 = self.pop_stack();
                    let val1 = self.pop_stack();
                    self.push_stack(Object::Bool(!val1.equals(&val2)));
                }

                OpCode::LessThan => {
                    let val2 = self.pop_stack();
                    let val1 = self.pop_stack();

                    match val1.lt(val2) {
                        Ok(r) => self.push_stack(r),
                        Err(e) => return e.to_runtime_error(),
                    }
                }

                OpCode::LessThanEq => {
                    let val2 = self.pop_stack();
                    let val1 = self.pop_stack();

                    match val1.lteq(val2) {
                        Ok(r) => self.push_stack(r),
                        Err(e) => return e.to_runtime_error(),
                    }
                }

                OpCode::GreaterThan => {
                    let val2 = self.pop_stack();
                    let val1 = self.pop_stack();

                    match val1.gt(val2) {
                        Ok(r) => self.push_stack(r),
                        Err(e) => return e.to_runtime_error(),
                    }
                }

                OpCode::GreaterThanEq => {
                    let val2 = self.pop_stack();
                    let val1 = self.pop_stack();

                    match val1.gteq(val2) {
                        Ok(r) => self.push_stack(r),
                        Err(e) => return e.to_runtime_error(),
                    }
                }

                OpCode::BitwiseOr => {
                    let right = self.pop_stack();
                    let left = self.pop_stack();

                    match left | right {
                        Ok(r) => self.push_stack(r),
                        Err(e) => return e.to_runtime_error(),
                    }
                }

                OpCode::BitwiseXor => {
                    let right = self.pop_stack();
                    let left = self.pop_stack();

                    match left ^ right {
                        Ok(r) => self.push_stack(r),
                        Err(e) => return e.to_runtime_error(),
                    }
                }

                OpCode::BitwiseAnd => {
                    let right = self.pop_stack();
                    let left = self.pop_stack();

                    match left & right {
                        Ok(r) => self.push_stack(r),
                        Err(e) => return e.to_runtime_error(),
                    }
                }

                OpCode::BitwiseNot => match !self.pop_stack() {
                    Ok(r) => self.push_stack(r),
                    Err(e) => return e.to_runtime_error(),
                },

                OpCode::BitwiseShiftLeft => {
                    let right = self.pop_stack();
                    let left = self.pop_stack();

                    match left << right {
                        Ok(r) => self.push_stack(r),
                        Err(e) => return e.to_runtime_error(),
                    }
                }

                OpCode::BitwiseShiftRight => {
                    let right = self.pop_stack();
                    let left = self.pop_stack();

                    match left >> right {
                        Ok(r) => self.push_stack(r),
                        Err(e) => return e.to_runtime_error(),
                    }
                }

                OpCode::MakeRange => {
                    let right = self.pop_stack();
                    let left = self.pop_stack();

                    if left.is_int() && right.is_int() {
                        let a = left.as_int().unwrap();
                        let b = right.as_int().unwrap();
                        self.push_stack(Object::Range(RangeObject { min: a, max: b }));
                    } else {
                        return RuntimeResult::Error {
                            error: RuntimeErrorType::TypeError,
                            message: format!(
                                "Operation '..' not defined for operands of type '{}' and '{}'.",
                                left.type_name(),
                                right.type_name()
                            ),
                        };
                    }
                }

                OpCode::NullishCoalescing => {
                    let value = self.pop_stack();
                    let nullish = self.pop_stack();

                    if nullish.is_null() {
                        self.push_stack(value);
                    } else {
                        self.push_stack(nullish);
                    }
                }

                OpCode::PopJumpIfFalse => {
                    // The OP_JUMP_IF_FALSE instruction always has a short as its operand.
                    let offset = self.get_next_short().unwrap() as usize;

                    if self.pop_stack().is_falsey() {
                        self.current_frame_mut().ip += offset;
                    }
                }

                OpCode::JumpIfFalseOrPop => {
                    // The OP_JUMP_IF_FALSE instruction always has a short as its operand.
                    let offset = self.get_next_short().unwrap() as usize;

                    if self.peek_stack(self.stack.len() - 1).is_falsey() {
                        self.current_frame_mut().ip += offset;
                    } else {
                        self.pop_stack();
                    }
                }

                OpCode::JumpIfTrueOrPop => {
                    // The OP_JUMP_IF_FALSE instruction always has a short as its operand.
                    let offset = self.get_next_short().unwrap() as usize;

                    if !self.peek_stack(self.stack.len() - 1).is_falsey() {
                        self.current_frame_mut().ip += offset;
                    } else {
                        self.pop_stack();
                    }
                }

                OpCode::JumpForward => {
                    // The OP_JUMP instruction always has a short as its operand.
                    let offset = self.get_next_short().unwrap() as usize;
                    self.current_frame_mut().ip += offset;
                }

                OpCode::LoopJump | OpCode::LoopJumpLong => {
                    let offset = if let OpCode::LoopJump = instruction {
                        self.get_next_byte().unwrap() as usize
                    } else {
                        self.get_next_short().unwrap() as usize
                    };

                    self.current_frame_mut().ip -= offset;
                }

                OpCode::LoadNative => {
                    let name = match self.pop_stack() {
                        Object::String(x) => x,
                        _ => {
                            unreachable!("Expected a native function name on TOS.");
                        }
                    };

                    match self.natives.get_native_fn_object(&name) {
                        Ok(f) => self.push_stack(Object::NativeFunction(f)),
                        Err(e) => {
                            return RuntimeResult::Error {
                                error: RuntimeErrorType::ReferenceError,
                                message: e,
                            }
                        }
                    }
                }

                OpCode::FuncCall => {
                    let arg_count = self.get_next_byte().unwrap();

                    let maybe_function = self
                        .peek_stack(self.stack.len() - (arg_count as usize) - 1)
                        .clone();

                    match self.call_value(maybe_function, arg_count) {
                        RuntimeResult::Ok => {}
                        RuntimeResult::Error { error, message } => {
                            return RuntimeResult::Error { error, message }
                        }
                    }
                }

                OpCode::BindDefaults => {
                    let param_count = self.get_next_byte().unwrap();

                    let mut defaults: Vec<Object> = vec![];
                    for _ in 0..param_count {
                        let p = self.pop_stack();
                        defaults.push(p);
                    }
                    defaults.reverse();

                    match self.peek_stack_mut(self.stack.len() - 1) {
                        Object::Function(m) => {
                            m.defaults = defaults;
                        }
                        _ => unreachable!("Expected a function object on TOS."),
                    }
                }

                OpCode::Return => {
                    let locals_to_pop = self.get_next_byte().unwrap();
                    let result = self.pop_stack();

                    // Pops local declarations from the stack
                    for _ in 0..(locals_to_pop + 1) {
                        self.pop_stack();
                    }

                    // removes the call frame
                    self.frames.pop();

                    if self.frames.len() == 0 {
                        return RuntimeResult::Ok;
                    }

                    self.push_stack(result);
                }
            }

            // Prints the execution of the program.
            // self.print_execution(instruction);
        }

        // If the compiler reaches this point, that means there were no errors
        // to return (because errors are returned by the match rules), so we can
        // safely return an `INTERPRET_OK` result.
        return RuntimeResult::Ok;
    }
}
