use super::VirtualMachine;
use super::{CallFrame, InterpretResult};
use crate::{
    chunk::OpCode,
    natives::{get_next_in_iter, iter_has_next, make_iter},
    objects::RangeObject,
};
use crate::{
    objects::{FunctionObject, Object},
    FRAMES_MAX,
};

impl<'a> VirtualMachine {
    /// Executes the instruction in a chunk. This is where the interpreter
    /// will spend most its life, therefore, optimizing every function in this file
    /// is very important.
    ///
    /// ## Returns
    /// `InterpretResult` – The result of the execution.
    pub(crate) fn run(&mut self) -> InterpretResult {
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
                        Err(e) => {
                            self.report_runtime_error(&e);
                            return InterpretResult::RuntimeError;
                        }
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

                    match target.get(&index) {
                        Ok(r) => self.push_stack(r),
                        Err(e) => {
                            self.report_runtime_error(e.as_str());
                            return InterpretResult::RuntimeError;
                        }
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
                    Err(e) => {
                        self.report_runtime_error(e.as_str());
                        return InterpretResult::RuntimeError;
                    }
                },

                OpCode::Add => {
                    let val2 = self.pop_stack();
                    let val1 = self.pop_stack();

                    match val1 + val2 {
                        Ok(r) => self.push_stack(r),
                        Err(e) => {
                            self.report_runtime_error(e.as_str());
                            return InterpretResult::RuntimeError;
                        }
                    }
                }

                OpCode::Subtract => {
                    let val2 = self.pop_stack();
                    let val1 = self.pop_stack();

                    match val1 - val2 {
                        Ok(r) => self.push_stack(r),
                        Err(e) => {
                            self.report_runtime_error(e.as_str());
                            return InterpretResult::RuntimeError;
                        }
                    }
                }

                OpCode::Multiply => {
                    let val2 = self.pop_stack();
                    let val1 = self.pop_stack();

                    match val1 * val2 {
                        Ok(r) => self.push_stack(r),
                        Err(e) => {
                            self.report_runtime_error(e.as_str());
                            return InterpretResult::RuntimeError;
                        }
                    }
                }

                OpCode::Divide => {
                    let val2 = self.pop_stack();
                    let val1 = self.pop_stack();

                    match val1 / val2 {
                        Ok(r) => self.push_stack(r),
                        Err(e) => {
                            self.report_runtime_error(e.as_str());
                            return InterpretResult::RuntimeError;
                        }
                    }
                }

                OpCode::Modulus => {
                    let val2 = self.pop_stack();
                    let val1 = self.pop_stack();

                    match val1 % val2 {
                        Ok(r) => self.push_stack(r),
                        Err(e) => {
                            self.report_runtime_error(e.as_str());
                            return InterpretResult::RuntimeError;
                        }
                    }
                }

                OpCode::Expo => {
                    let val2 = self.pop_stack();
                    let val1 = self.pop_stack();

                    match val1.pow(val2) {
                        Ok(r) => self.push_stack(r),
                        Err(e) => {
                            self.report_runtime_error(e.as_str());
                            return InterpretResult::RuntimeError;
                        }
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
                        Err(e) => {
                            self.report_runtime_error(e.as_str());
                            return InterpretResult::RuntimeError;
                        }
                    }
                }

                OpCode::LessThanEq => {
                    let val2 = self.pop_stack();
                    let val1 = self.pop_stack();

                    match val1.lteq(val2) {
                        Ok(r) => self.push_stack(r),
                        Err(e) => {
                            self.report_runtime_error(e.as_str());
                            return InterpretResult::RuntimeError;
                        }
                    }
                }

                OpCode::GreaterThan => {
                    let val2 = self.pop_stack();
                    let val1 = self.pop_stack();

                    match val1.gt(val2) {
                        Ok(r) => self.push_stack(r),
                        Err(e) => {
                            self.report_runtime_error(e.as_str());
                            return InterpretResult::RuntimeError;
                        }
                    }
                }

                OpCode::GreaterThanEq => {
                    let val2 = self.pop_stack();
                    let val1 = self.pop_stack();

                    match val1.gteq(val2) {
                        Ok(r) => self.push_stack(r),
                        Err(e) => {
                            self.report_runtime_error(e.as_str());
                            return InterpretResult::RuntimeError;
                        }
                    }
                }

                OpCode::BitwiseOr => {
                    let right = self.pop_stack();
                    let left = self.pop_stack();

                    match left | right {
                        Ok(r) => self.push_stack(r),
                        Err(e) => {
                            self.report_runtime_error(e.as_str());
                            return InterpretResult::RuntimeError;
                        }
                    }
                }

                OpCode::BitwiseXor => {
                    let right = self.pop_stack();
                    let left = self.pop_stack();

                    match left ^ right {
                        Ok(r) => self.push_stack(r),
                        Err(e) => {
                            self.report_runtime_error(e.as_str());
                            return InterpretResult::RuntimeError;
                        }
                    }
                }

                OpCode::BitwiseAnd => {
                    let right = self.pop_stack();
                    let left = self.pop_stack();

                    match left & right {
                        Ok(r) => self.push_stack(r),
                        Err(e) => {
                            self.report_runtime_error(e.as_str());
                            return InterpretResult::RuntimeError;
                        }
                    }
                }

                OpCode::BitwiseNot => match !self.pop_stack() {
                    Ok(r) => self.push_stack(r),
                    Err(e) => {
                        self.report_runtime_error(e.as_str());
                        return InterpretResult::RuntimeError;
                    }
                },

                OpCode::BitwiseShiftLeft => {
                    let right = self.pop_stack();
                    let left = self.pop_stack();

                    match left << right {
                        Ok(r) => self.push_stack(r),
                        Err(e) => {
                            self.report_runtime_error(e.as_str());
                            return InterpretResult::RuntimeError;
                        }
                    }
                }

                OpCode::BitwiseShiftRight => {
                    let right = self.pop_stack();
                    let left = self.pop_stack();

                    match left >> right {
                        Ok(r) => self.push_stack(r),
                        Err(e) => {
                            self.report_runtime_error(e.as_str());
                            return InterpretResult::RuntimeError;
                        }
                    }
                }

                OpCode::MakeRange => {
                    let right = self.pop_stack();
                    let left = self.pop_stack();

                    if self.check_integer_operands(&left, &right, "..") {
                        let a = left.as_int().unwrap();
                        let b = right.as_int().unwrap();
                        self.push_stack(Object::Range(RangeObject { min: a, max: b }));
                    } else {
                        return InterpretResult::RuntimeError;
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
                            self.report_runtime_error("Expected native function name.");
                            return InterpretResult::RuntimeError;
                        }
                    };

                    match self.natives.get_native_fn_object(&name) {
                        Ok(f) => self.push_stack(Object::NativeFunction(f)),
                        Err(e) => {
                            self.report_runtime_error(e.as_str());
                            return InterpretResult::RuntimeError;
                        }
                    }
                }

                OpCode::FuncCall => {
                    let arg_count = self.get_next_byte().unwrap();

                    let maybe_function = self
                        .peek_stack(self.stack.len() - (arg_count as usize) - 1)
                        .clone();

                    match self.call_value(maybe_function, arg_count) {
                        Ok(_) => {}
                        Err(_) => {
                            return InterpretResult::RuntimeError;
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
                        _ => unreachable!("Expected a function object on stack top."),
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
                        return InterpretResult::Ok;
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
        return InterpretResult::Ok;
    }

    pub(super) fn call(&mut self, callee: FunctionObject, arg_count: u8) -> Result<(), ()> {
        let max_arity = callee.max_arity;
        let min_arity = callee.min_arity;

        // Check that the correct number of arguments is passed to the function
        if arg_count < min_arity || arg_count > max_arity {
            if min_arity == max_arity {
                self.report_runtime_error(&format!(
                    "Expected {} arguments but got {} instead.",
                    min_arity, arg_count
                ))
            } else {
                self.report_runtime_error(&format!(
                    "Expected {} to {} arguments but got {} instead.",
                    min_arity, max_arity, arg_count
                ))
            }

            return Err(());
        }

        // Pushes the default values onto the stack
        // if they were not passed into the func call
        if arg_count != max_arity {
            let missing_args = max_arity - arg_count;

            for i in (max_arity - 1 - missing_args)..(max_arity - 1) {
                let val = callee.defaults[i as usize].clone();
                self.push_stack(val);
            }
        }

        // Check we are not overflowing the stack of frames
        if self.frames.len() >= (FRAMES_MAX as usize) {
            self.report_runtime_error("Stack overflow.");
            return Err(());
        }

        self.frames.push(CallFrame {
            function: callee,
            ip: 0,
            base_pointer: self.stack.len() - (max_arity as usize) - 1,
        });

        Ok(())
    }

    /// Prints the execution trace for the program. Useful for debugging the VM.
    ///
    /// ## Arguments
    /// * `instr` – The current OpCode to be executed.
    fn print_execution(&mut self, instr: OpCode) {
        println!("\n==========================");

        // Prints the next instruction to be executed
        println!("OpCode:\t\x1b[36m{:?}\x1b[0m ", instr);
        println!("Byte:\t{:#04X} ", instr as u8);

        // Prints the index of the current instruction
        println!("IP:\t{:>04} ", self.current_frame().ip);

        // Prints the current state of the values stack
        print!("stack\t[");
        for val in self.stack.iter() {
            print!("{}; ", val);
        }
        println!("]");

        print!("Output:\t");
    }
}
