use std::rc::Rc;

use super::VirtualMachine;
use super::{CallFrame, InterpretResult};
use crate::{chunk::OpCode, objects::RangeObject};
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
                OpCode::PopStack => {
                    self.pop_stack();
                }

                OpCode::LoadImmNull => self.push_stack(Object::Null),
                OpCode::LoadImmTrue => self.push_stack(Object::Bool(true)),
                OpCode::LoadImmFalse => self.push_stack(Object::Bool(false)),
                OpCode::LoadImm0F => self.push_stack(Object::Float(0f64)),
                OpCode::LoadImm0I => self.push_stack(Object::Int(0i64)),
                OpCode::LoadImm1F => self.push_stack(Object::Float(1f64)),
                OpCode::LoadImm1I => self.push_stack(Object::Int(1i64)),

                OpCode::LoadImm | OpCode::LoadImmLong => {
                    let imm = if let OpCode::LoadImm = instruction {
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

                OpCode::Indexing => {
                    let index = self.pop_stack();
                    let target = self.pop_stack();

                    if !index.is_int() {
                        self.report_runtime_error("Array index must be an integer.");
                        return InterpretResult::RuntimeError;
                    }

                    if !target.is_array() {
                        self.report_runtime_error(&format!(
                            "Cannot index object of type '{}'.",
                            target.type_name()
                        ));
                        return InterpretResult::RuntimeError;
                    }

                    let array = target.as_array().unwrap();
                    let idx = index.as_int().unwrap();
                    let idx = if idx >= 0i64 {
                        idx as usize
                    } else {
                        self.report_runtime_error("Array index out of bounds.");
                        return InterpretResult::RuntimeError;
                    };

                    match array.get(idx) {
                        Some(val) => {
                            self.push_stack(val.clone());
                        }
                        None => {
                            self.report_runtime_error("Array index out of bounds.");
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
                    let val2 = self.stack.pop().unwrap();
                    let val1 = self.stack.pop().unwrap();

                    match val1 + val2 {
                        Ok(r) => self.push_stack(r),
                        Err(e) => {
                            self.report_runtime_error(e.as_str());
                            return InterpretResult::RuntimeError;
                        }
                    }
                }

                OpCode::Subtract => {
                    let val2 = self.stack.pop().unwrap();
                    let val1 = self.stack.pop().unwrap();

                    match val1 - val2 {
                        Ok(r) => self.push_stack(r),
                        Err(e) => {
                            self.report_runtime_error(e.as_str());
                            return InterpretResult::RuntimeError;
                        }
                    }
                }

                OpCode::Multiply => {
                    let val2 = self.stack.pop().unwrap();
                    let val1 = self.stack.pop().unwrap();

                    match val1 * val2 {
                        Ok(r) => self.push_stack(r),
                        Err(e) => {
                            self.report_runtime_error(e.as_str());
                            return InterpretResult::RuntimeError;
                        }
                    }
                }

                OpCode::Divide => {
                    let val2 = self.stack.pop().unwrap();
                    let val1 = self.stack.pop().unwrap();

                    match val1 / val2 {
                        Ok(r) => self.push_stack(r),
                        Err(e) => {
                            self.report_runtime_error(e.as_str());
                            return InterpretResult::RuntimeError;
                        }
                    }
                }

                OpCode::Modulus => {
                    let val2 = self.stack.pop().unwrap();
                    let val1 = self.stack.pop().unwrap();

                    match val1 % val2 {
                        Ok(r) => self.push_stack(r),
                        Err(e) => {
                            self.report_runtime_error(e.as_str());
                            return InterpretResult::RuntimeError;
                        }
                    }
                }

                OpCode::Expo => {
                    let val2 = self.stack.pop().unwrap();
                    let val1 = self.stack.pop().unwrap();

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
                        let a = left.as_int().unwrap() as isize;
                        let b = right.as_int().unwrap() as isize;

                        self.push_stack(Object::Range(Rc::new(RangeObject {
                            min: a,
                            max: b,
                            step: 1,
                        })));
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

                OpCode::JumpIfFalse => {
                    // The OP_JUMP_IF_FALSE instruction always has a short as its operand.
                    let offset = self.get_next_short().unwrap() as usize;

                    let top = self.stack.last().unwrap();

                    if top.is_falsey() {
                        self.current_frame_mut().ip += offset;
                    }
                }

                OpCode::Jump => {
                    // The OP_JUM instruction always has a short as its operand.
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
            // self.print_execution(&instruction);
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
