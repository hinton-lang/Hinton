use std::rc::Rc;

use super::VirtualMachine;
use super::{CallFrame, InterpretResult};
use crate::objects::Object;
use crate::{chunk::op_codes::OpCode, objects::RangeObject};

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
                OpCode::OP_PRINT => println!("{}", self.pop_stack()),

                OpCode::OP_POP_STACK => {
                    self.pop_stack();
                }

                OpCode::OP_NULL => self.push_stack(Object::Null),
                OpCode::OP_TRUE => self.push_stack(Object::Bool(true)),
                OpCode::OP_FALSE => self.push_stack(Object::Bool(false)),
                OpCode::OP_LOAD_IMM_0 => self.push_stack(Object::Number(0f64)),
                OpCode::OP_LOAD_IMM_1 => self.push_stack(Object::Number(1f64)),

                OpCode::OP_LOAD_IMM | OpCode::OP_LOAD_IMM_LONG => {
                    let imm = if let OpCode::OP_LOAD_IMM = instruction {
                        self.get_next_byte().unwrap() as f64
                    } else {
                        self.get_next_short().unwrap() as f64
                    };

                    self.push_stack(Object::Number(imm))
                }

                OpCode::OP_LOAD_CONST | OpCode::OP_LOAD_CONST_LONG => {
                    // Either gets the next byte or the next short based on the instruction
                    // The compiler makes sure that the structure of the bytecode is correct
                    // for the VM to execute, so unwrapping without check should be fine.
                    let pos = if let OpCode::OP_LOAD_CONST = instruction {
                        self.get_next_byte().unwrap() as usize
                    } else {
                        self.get_next_short().unwrap() as usize
                    };

                    // Gets the value from the pool and places it on top of the stack
                    let val = self.read_constant(pos).clone();
                    self.push_stack(val)

                    // match c {
                    //     Some(val) => self.push_stack(Rc::clone(val)),
                    //     None => {
                    //         self.report_runtime_error(&format!("InternalRuntimeError: Constant pool index '{}' out of range", pos));
                    //         return InterpretResult::INTERPRET_RUNTIME_ERROR;
                    //     }
                    // }
                }

                OpCode::OP_ARRAY | OpCode::OP_ARRAY_LONG => {
                    // The number of values to pop from the stack. Essentially the size of the array.
                    let size = if let OpCode::OP_ARRAY = instruction {
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

                OpCode::OP_ARRAY_INDEXING => {
                    let index = self.pop_stack();
                    let target = self.pop_stack();

                    if !index.is_int() {
                        self.report_runtime_error("Array index must be an integer.");
                        return InterpretResult::INTERPRET_RUNTIME_ERROR;
                    }

                    if !target.is_array() {
                        self.report_runtime_error(&format!("Cannot index object of type '{}'.", target.type_name()));
                        return InterpretResult::INTERPRET_RUNTIME_ERROR;
                    }

                    let array = target.as_array().unwrap();
                    let idx = index.as_number().unwrap();
                    let idx = if idx >= 0f64 {
                        idx as usize
                    } else {
                        self.report_runtime_error("Array index out of bounds.");
                        return InterpretResult::INTERPRET_RUNTIME_ERROR;
                    };

                    match array.get(idx) {
                        Some(val) => {
                            self.push_stack(val.clone());
                        }
                        None => {
                            self.report_runtime_error("Array index out of bounds.");
                            return InterpretResult::INTERPRET_RUNTIME_ERROR;
                        }
                    }
                }

                OpCode::OP_GET_VAR | OpCode::OP_GET_VAR_LONG => {
                    // The position of the local variable's value in the stack
                    let pos = if let OpCode::OP_GET_VAR = instruction {
                        self.get_next_byte().unwrap() as usize
                    } else {
                        self.get_next_short().unwrap() as usize
                    };

                    let idx = self.current_frame().slots_base + pos;
                    let value = self.stack[idx].clone();
                    self.push_stack(value);
                }

                OpCode::OP_SET_VAR | OpCode::OP_SET_VAR_LONG => {
                    // The position of the local variable's value in the stack
                    let pos = if let OpCode::OP_SET_VAR = instruction {
                        self.get_next_byte().unwrap() as usize
                    } else {
                        self.get_next_short().unwrap() as usize
                    };

                    let value = self.stack.last().unwrap();
                    let offset = self.current_frame().slots_base;

                    self.stack[pos + offset] = value.clone();
                }

                OpCode::OP_NEGATE => {
                    let val = self.pop_stack();

                    if !val.is_numeric() {
                        self.report_runtime_error(&format!("Cannot negate operand of type '{}'.", val.type_name()));
                        return InterpretResult::INTERPRET_RUNTIME_ERROR;
                    } else {
                        self.push_stack(Object::Number(-val.as_number().unwrap()));
                    }
                }

                OpCode::OP_ADD => match self.perform_addition() {
                    Ok(_) => (),
                    Err(e) => return e,
                },

                OpCode::OP_SUBTRACT => match self.perform_subtraction() {
                    Ok(_) => (),
                    Err(e) => return e,
                },

                OpCode::OP_MULTIPLY => match self.perform_multiplication() {
                    Ok(_) => (),
                    Err(e) => return e,
                },

                OpCode::OP_DIVIDE => match self.perform_division() {
                    Ok(_) => (),
                    Err(e) => return e,
                },

                OpCode::OP_MODULUS => match self.perform_modulus() {
                    Ok(_) => (),
                    Err(e) => return e,
                },

                OpCode::OP_EXPO => match self.perform_exponentiation() {
                    Ok(_) => (),
                    Err(e) => return e,
                },

                OpCode::OP_LOGIC_NOT => {
                    let val = self.pop_stack();
                    self.push_stack(Object::Bool(val.is_falsey()));
                }

                OpCode::OP_EQUALS => {
                    let val2 = self.pop_stack();
                    let val1 = self.pop_stack();
                    self.push_stack(Object::Bool(val1.equals(&val2)));
                }

                OpCode::OP_NOT_EQUALS => {
                    let val2 = self.pop_stack();
                    let val1 = self.pop_stack();
                    self.push_stack(Object::Bool(!val1.equals(&val2)));
                }

                OpCode::OP_LESS_THAN => {
                    let val2 = self.pop_stack();
                    let val1 = self.pop_stack();
                    let numeric = self.check_numeric_operands(&val1, &val2, "<");

                    if numeric {
                        let v1 = val1.as_number().unwrap();
                        let v2 = val2.as_number().unwrap();
                        self.push_stack(Object::Bool(v1 < v2));
                    } else {
                        return InterpretResult::INTERPRET_RUNTIME_ERROR;
                    }
                }

                OpCode::OP_LESS_THAN_EQ => {
                    let val2 = self.pop_stack();
                    let val1 = self.pop_stack();
                    let numeric = self.check_numeric_operands(&val1, &val2, "<=");

                    if numeric {
                        let v1 = val1.as_number().unwrap();
                        let v2 = val2.as_number().unwrap();
                        self.push_stack(Object::Bool(v1 <= v2));
                    } else {
                        return InterpretResult::INTERPRET_RUNTIME_ERROR;
                    }
                }

                OpCode::OP_GREATER_THAN => {
                    let val2 = self.pop_stack();
                    let val1 = self.pop_stack();
                    let numeric = self.check_numeric_operands(&val1, &val2, ">");

                    if numeric {
                        let v1 = val1.as_number().unwrap();
                        let v2 = val2.as_number().unwrap();
                        self.push_stack(Object::Bool(v1 > v2));
                    } else {
                        return InterpretResult::INTERPRET_RUNTIME_ERROR;
                    }
                }

                OpCode::OP_GREATER_THAN_EQ => {
                    let val2 = self.pop_stack();
                    let val1 = self.pop_stack();
                    let numeric = self.check_numeric_operands(&val1, &val2, ">=");

                    if numeric {
                        let v1 = val1.as_number().unwrap();
                        let v2 = val2.as_number().unwrap();
                        self.push_stack(Object::Bool(v1 >= v2));
                    } else {
                        return InterpretResult::INTERPRET_RUNTIME_ERROR;
                    }
                }

                OpCode::OP_BITWISE_OR => {
                    let right = self.pop_stack();
                    let left = self.pop_stack();

                    if self.check_integer_operands(&left, &right, "|") {
                        self.push_stack(Object::Number(
                            ((left.as_number().unwrap() as isize) | (right.as_number().unwrap() as isize)) as f64,
                        ));
                    } else {
                        return InterpretResult::INTERPRET_RUNTIME_ERROR;
                    }
                }

                OpCode::OP_BITWISE_XOR => {
                    let right = self.pop_stack();
                    let left = self.pop_stack();

                    if self.check_integer_operands(&left, &right, "^") {
                        self.push_stack(Object::Number(
                            ((left.as_number().unwrap() as isize) ^ (right.as_number().unwrap() as isize)) as f64,
                        ));
                    } else {
                        return InterpretResult::INTERPRET_RUNTIME_ERROR;
                    }
                }

                OpCode::OP_BITWISE_AND => {
                    let right = self.pop_stack();
                    let left = self.pop_stack();

                    if self.check_integer_operands(&left, &right, "&") {
                        self.push_stack(Object::Number(
                            ((left.as_number().unwrap() as isize) & (right.as_number().unwrap() as isize)) as f64,
                        ));
                    } else {
                        return InterpretResult::INTERPRET_RUNTIME_ERROR;
                    }
                }

                OpCode::OP_BITWISE_NOT => {
                    let operand = self.pop_stack();

                    if operand.is_numeric() {
                        self.push_stack(Object::Number(!(operand.as_number().unwrap() as isize) as f64));
                    } else {
                        self.report_runtime_error(&format!("Operation '~' not defined for operand of type '{}'.", operand.type_name()));
                        return InterpretResult::INTERPRET_RUNTIME_ERROR;
                    }
                }

                OpCode::OP_BITWISE_L_SHIFT => {
                    let right = self.pop_stack();
                    let left = self.pop_stack();

                    if self.check_integer_operands(&left, &right, "<<") {
                        self.push_stack(Object::Number(
                            ((left.as_number().unwrap() as isize) << (right.as_number().unwrap() as isize)) as f64,
                        ));
                    } else {
                        return InterpretResult::INTERPRET_RUNTIME_ERROR;
                    }
                }

                OpCode::OP_BITWISE_R_SHIFT => {
                    let right = self.pop_stack();
                    let left = self.pop_stack();

                    if self.check_integer_operands(&left, &right, ">>") {
                        self.push_stack(Object::Number(
                            ((left.as_number().unwrap() as isize) >> (right.as_number().unwrap() as isize)) as f64,
                        ));
                    } else {
                        return InterpretResult::INTERPRET_RUNTIME_ERROR;
                    }
                }

                OpCode::OP_GENERATE_RANGE => {
                    let right = self.pop_stack();
                    let left = self.pop_stack();

                    if self.check_integer_operands(&left, &right, "..") {
                        let a = left.as_number().unwrap() as isize;
                        let b = right.as_number().unwrap() as isize;
                        self.push_stack(Object::Range(Rc::new(RangeObject { min: a, max: b, step: 1 })));
                    } else {
                        return InterpretResult::INTERPRET_RUNTIME_ERROR;
                    }
                }

                OpCode::OP_NULLISH_COALESCING => {
                    let value = self.pop_stack();
                    let nullish = self.pop_stack();

                    if nullish.is_null() {
                        self.push_stack(value);
                    } else {
                        self.push_stack(nullish);
                    }
                }

                OpCode::OP_JUMP_IF_FALSE => {
                    // The OP_JUMP_IF_FALSE instruction always has a short as its operand.
                    let offset = self.get_next_short().unwrap() as usize;

                    let top = self.stack.last().unwrap();

                    if top.is_falsey() {
                        self.current_frame_mut().ip += offset;
                    }
                }

                OpCode::OP_JUMP => {
                    // The OP_JUM instruction always has a short as its operand.
                    let offset = self.get_next_short().unwrap() as usize;
                    self.current_frame_mut().ip += offset;
                }

                OpCode::OP_LOOP_JUMP | OpCode::OP_LOOP_JUMP_LONG => {
                    let offset = if let OpCode::OP_LOOP_JUMP = instruction {
                        self.get_next_byte().unwrap() as usize
                    } else {
                        self.get_next_short().unwrap() as usize
                    };

                    self.current_frame_mut().ip -= offset;
                }

                OpCode::OP_POST_INCREMENT => {
                    let pos = if let OpCode::OP_POST_INCREMENT = instruction {
                        self.get_next_byte().unwrap() as usize
                    } else {
                        self.get_next_short().unwrap() as usize
                    };

                    let value = self.stack[pos].clone();

                    if !value.is_numeric() {
                        self.report_runtime_error(&format!("Cannot increment object of type '{}'.", value.type_name()));
                        return InterpretResult::INTERPRET_RUNTIME_ERROR;
                    }

                    self.stack[pos] = Object::Number(value.as_number().unwrap() + 1f64);
                    self.push_stack(value);
                }

                OpCode::OP_POST_DECREMENT => {
                    let pos = if let OpCode::OP_POST_DECREMENT = instruction {
                        self.get_next_byte().unwrap() as usize
                    } else {
                        self.get_next_short().unwrap() as usize
                    };

                    let value = self.stack[pos].clone();

                    if !value.is_numeric() {
                        self.report_runtime_error(&format!("Cannot decrement object of type '{}'.", value.type_name()));
                        return InterpretResult::INTERPRET_RUNTIME_ERROR;
                    }

                    self.stack[pos] = Object::Number(value.as_number().unwrap() - 1f64);
                    self.push_stack(value);
                }

                // OpCode::OP_RETURN => { }

                // Should be removed so that there are no unimplemented OpCodes
                _ => unreachable!("OpCode Not implemented! {:?}", instruction),
            }

            // Prints the execution of the program.
            // self.print_execution(&instruction);
        }

        // If the compiler reaches this point, that means there were no errors
        // to return (because errors are returned by the match rules), so we can
        // safely return an `INTERPRET_OK` result.
        return InterpretResult::INTERPRET_OK;
    }

    fn current_frame(&self) -> &CallFrame {
        self.frames.last().unwrap()
    }

    fn current_frame_mut(&mut self) -> &mut CallFrame {
        let frames_len = self.frames.len();
        &mut self.frames[frames_len - 1]
    }

    fn get_next_op_code(&mut self) -> Option<OpCode> {
        self.current_frame_mut().get_next_op_code()
    }

    fn get_next_byte(&mut self) -> Option<u8> {
        self.current_frame_mut().get_next_byte()
    }

    fn get_next_short(&mut self) -> Option<u16> {
        self.current_frame_mut().get_next_short()
    }

    fn pop_stack(&mut self) -> Object {
        match self.stack.pop() {
            Some(obj) => obj,
            None => {
                panic!("Stack is empty!")
            }
        }
    }

    fn push_stack(&mut self, new_val: Object) {
        self.stack.push(new_val)
    }

    fn read_constant(&self, idx: usize) -> &Object {
        return self.current_frame().get_constant(idx);
    }

    /// Prints the execution trace for the program. Useful for debugging the VM.
    ///
    /// ## Arguments
    /// * `instr` – The current OpCode to be executed.
    fn print_execution(&mut self, instr: &OpCode) {
        println!("\n==========================");

        // Prints the next instruction to be executed
        println!("OpCode:\t\x1b[36m{:?}\x1b[0m ", instr);
        println!("Byte:\t{:#04X} ", instr.clone() as u8);

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
