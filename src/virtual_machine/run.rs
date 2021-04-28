use std::rc::Rc;

use super::InterpretResult;
use super::VirtualMachine;
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
            // Prints the execution of the program.
            // self.print_execution(&instruction);

            match instruction {
                OpCode::OP_POP_STACK => {
                    self.stack.pop();
                }

                OpCode::OP_NULL => self.stack.push(Rc::new(Object::Null)),
                OpCode::OP_TRUE => self.stack.push(Rc::new(Object::Bool(true))),
                OpCode::OP_FALSE => self.stack.push(Rc::new(Object::Bool(false))),
                OpCode::OP_LOAD_IMM_0 => self.stack.push(Rc::new(Object::Number(0f64))),
                OpCode::OP_LOAD_IMM_1 => self.stack.push(Rc::new(Object::Number(1f64))),

                OpCode::OP_LOAD_IMM | OpCode::OP_LOAD_IMM_LONG => {
                    let imm = if let OpCode::OP_LOAD_IMM = instruction {
                        self.get_next_byte().unwrap() as f64
                    } else {
                        self.get_next_short().unwrap() as f64
                    };

                    self.stack.push(Rc::new(Object::Number(imm)))
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
                    match self.chunk.get_constant(pos) {
                        Some(val) => self.stack.push(Rc::clone(val)),
                        None => {
                            self.report_runtime_error(&format!("InternalRuntimeError: Constant pool index '{}' out of range", pos));
                            return InterpretResult::INTERPRET_RUNTIME_ERROR;
                        }
                    }
                }

                OpCode::OP_ARRAY | OpCode::OP_ARRAY_LONG => {
                    // The number of values to pop from the stack. Essentially the size of the array.
                    let size = if let OpCode::OP_ARRAY = instruction {
                        self.get_next_byte().unwrap() as usize
                    } else {
                        self.get_next_short().unwrap() as usize
                    };

                    let mut arr_values: Vec<Rc<Object>> = vec![];

                    for _ in 0..size {
                        let val = match self.stack.pop() {
                            Some(v) => v,
                            None => {
                                self.report_runtime_error("Unexpected Runtime Error: Stack is empty.");
                                return InterpretResult::INTERPRET_RUNTIME_ERROR;
                            }
                        };

                        arr_values.push(val);
                    }

                    self.stack.push(Rc::new(Object::Array(arr_values)));
                }

                OpCode::OP_ARRAY_INDEXING => {
                    let index = Rc::clone(&self.stack.pop().unwrap());
                    let target = Rc::clone(&self.stack.pop().unwrap());

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
                            self.stack.push(Rc::clone(val));
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

                    let value = Rc::clone(&self.stack.get_mut(pos).unwrap());
                    self.stack.push(value);
                }

                OpCode::OP_SET_VAR | OpCode::OP_SET_VAR_LONG => {
                    // The position of the local variable's value in the stack
                    let pos = if let OpCode::OP_SET_VAR = instruction {
                        self.get_next_byte().unwrap() as usize
                    } else {
                        self.get_next_short().unwrap() as usize
                    };

                    let value = Rc::clone(&self.stack.last_mut().unwrap());
                    self.stack[pos] = value;
                }

                OpCode::OP_NEGATE => {
                    let val = Rc::clone(&self.stack.pop().unwrap());

                    if !val.is_numeric() {
                        self.report_runtime_error(&format!("Cannot negate operand of type '{}'.", val.type_name()));
                        return InterpretResult::INTERPRET_RUNTIME_ERROR;
                    } else {
                        self.stack.push(Rc::new(Object::Number(-val.as_number().unwrap())));
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
                    let val = Rc::clone(&self.stack.pop().unwrap());
                    self.stack.push(Rc::new(Object::Bool(val.is_falsey())));
                }

                OpCode::OP_EQUALS => {
                    let val2 = Rc::clone(&self.stack.pop().unwrap());
                    let val1 = Rc::clone(&self.stack.pop().unwrap());
                    self.stack.push(Rc::new(Object::Bool(val1.equals(val2))));
                }

                OpCode::OP_NOT_EQUALS => {
                    let val2 = Rc::clone(&self.stack.pop().unwrap());
                    let val1 = Rc::clone(&self.stack.pop().unwrap());
                    self.stack.push(Rc::new(Object::Bool(!val1.equals(val2))));
                }

                OpCode::OP_LESS_THAN => {
                    let val2 = Rc::clone(&self.stack.pop().unwrap());
                    let val1 = Rc::clone(&self.stack.pop().unwrap());
                    let numeric = self.check_numeric_operands(Rc::clone(&val1), Rc::clone(&val2), "<");

                    if numeric {
                        let v1 = val1.as_number().unwrap();
                        let v2 = val2.as_number().unwrap();
                        self.stack.push(Rc::new(Object::Bool(v1 < v2)));
                    } else {
                        return InterpretResult::INTERPRET_RUNTIME_ERROR;
                    }
                }

                OpCode::OP_LESS_THAN_EQ => {
                    let val2 = Rc::clone(&self.stack.pop().unwrap());
                    let val1 = Rc::clone(&self.stack.pop().unwrap());
                    let numeric = self.check_numeric_operands(Rc::clone(&val1), Rc::clone(&val2), "<=");

                    if numeric {
                        let v1 = val1.as_number().unwrap();
                        let v2 = val2.as_number().unwrap();
                        self.stack.push(Rc::new(Object::Bool(v1 <= v2)));
                    } else {
                        return InterpretResult::INTERPRET_RUNTIME_ERROR;
                    }
                }

                OpCode::OP_GREATER_THAN => {
                    let val2 = Rc::clone(&self.stack.pop().unwrap());
                    let val1 = Rc::clone(&self.stack.pop().unwrap());
                    let numeric = self.check_numeric_operands(Rc::clone(&val1), Rc::clone(&val2), ">");

                    if numeric {
                        let v1 = val1.as_number().unwrap();
                        let v2 = val2.as_number().unwrap();
                        self.stack.push(Rc::new(Object::Bool(v1 > v2)));
                    } else {
                        return InterpretResult::INTERPRET_RUNTIME_ERROR;
                    }
                }

                OpCode::OP_GREATER_THAN_EQ => {
                    let val2 = Rc::clone(&self.stack.pop().unwrap());
                    let val1 = Rc::clone(&self.stack.pop().unwrap());
                    let numeric = self.check_numeric_operands(Rc::clone(&val1), Rc::clone(&val2), ">=");

                    if numeric {
                        let v1 = val1.as_number().unwrap();
                        let v2 = val2.as_number().unwrap();
                        self.stack.push(Rc::new(Object::Bool(v1 >= v2)));
                    } else {
                        return InterpretResult::INTERPRET_RUNTIME_ERROR;
                    }
                }

                OpCode::OP_BITWISE_OR => {
                    let right = Rc::clone(&self.stack.pop().unwrap());
                    let left = Rc::clone(&self.stack.pop().unwrap());

                    if self.check_integer_operands(Rc::clone(&left), Rc::clone(&right), "|") {
                        self.stack.push(Rc::new(Object::Number(
                            ((left.as_number().unwrap() as isize) | (right.as_number().unwrap() as isize)) as f64,
                        )));
                    } else {
                        return InterpretResult::INTERPRET_RUNTIME_ERROR;
                    }
                }

                OpCode::OP_BITWISE_XOR => {
                    let right = Rc::clone(&self.stack.pop().unwrap());
                    let left = Rc::clone(&self.stack.pop().unwrap());

                    if self.check_integer_operands(Rc::clone(&left), Rc::clone(&right), "^") {
                        self.stack.push(Rc::new(Object::Number(
                            ((left.as_number().unwrap() as isize) ^ (right.as_number().unwrap() as isize)) as f64,
                        )));
                    } else {
                        return InterpretResult::INTERPRET_RUNTIME_ERROR;
                    }
                }

                OpCode::OP_BITWISE_AND => {
                    let right = Rc::clone(&self.stack.pop().unwrap());
                    let left = Rc::clone(&self.stack.pop().unwrap());

                    if self.check_integer_operands(Rc::clone(&left), Rc::clone(&right), "&") {
                        self.stack.push(Rc::new(Object::Number(
                            ((left.as_number().unwrap() as isize) & (right.as_number().unwrap() as isize)) as f64,
                        )));
                    } else {
                        return InterpretResult::INTERPRET_RUNTIME_ERROR;
                    }
                }

                OpCode::OP_BITWISE_NOT => {
                    let operand = Rc::clone(&self.stack.pop().unwrap());

                    if operand.is_numeric() {
                        self.stack.push(Rc::new(Object::Number(!(operand.as_number().unwrap() as isize) as f64)));
                    } else {
                        self.report_runtime_error(&format!("Operation '~' not defined for operand of type '{}'.", operand.type_name()));
                        return InterpretResult::INTERPRET_RUNTIME_ERROR;
                    }
                }

                OpCode::OP_BITWISE_L_SHIFT => {
                    let right = Rc::clone(&self.stack.pop().unwrap());
                    let left = Rc::clone(&self.stack.pop().unwrap());

                    if self.check_integer_operands(Rc::clone(&left), Rc::clone(&right), "<<") {
                        self.stack.push(Rc::new(Object::Number(
                            ((left.as_number().unwrap() as isize) << (right.as_number().unwrap() as isize)) as f64,
                        )));
                    } else {
                        return InterpretResult::INTERPRET_RUNTIME_ERROR;
                    }
                }

                OpCode::OP_BITWISE_R_SHIFT => {
                    let right = Rc::clone(&self.stack.pop().unwrap());
                    let left = Rc::clone(&self.stack.pop().unwrap());

                    if self.check_integer_operands(Rc::clone(&left), Rc::clone(&right), ">>") {
                        self.stack.push(Rc::new(Object::Number(
                            ((left.as_number().unwrap() as isize) >> (right.as_number().unwrap() as isize)) as f64,
                        )));
                    } else {
                        return InterpretResult::INTERPRET_RUNTIME_ERROR;
                    }
                }

                OpCode::OP_GENERATE_RANGE => {
                    let right = Rc::clone(&self.stack.pop().unwrap());
                    let left = Rc::clone(&self.stack.pop().unwrap());

                    if self.check_integer_operands(Rc::clone(&left), Rc::clone(&right), "..") {
                        let a = left.as_number().unwrap() as isize;
                        let b = right.as_number().unwrap() as isize;
                        self.stack.push(Rc::new(Object::Range(Rc::new(RangeObject { min: a, max: b, step: 1 }))));
                    } else {
                        return InterpretResult::INTERPRET_RUNTIME_ERROR;
                    }
                }

                OpCode::OP_NULLISH_COALESCING => {
                    let value = Rc::clone(&self.stack.pop().unwrap());
                    let nullish = Rc::clone(&self.stack.pop().unwrap());

                    if nullish.is_null() {
                        self.stack.push(value);
                    } else {
                        self.stack.push(nullish);
                    }
                }

                OpCode::OP_JUMP_IF_FALSE => {
                    // The OP_JUMP_IF_FALSE instruction always has a short as its operand.
                    let offset = self.get_next_short().unwrap() as usize;

                    let top = Rc::clone(&self.stack.last().unwrap());

                    if top.is_falsey() {
                        self.ip += offset;
                    }
                }

                OpCode::OP_JUMP => {
                    // The OP_JUM instruction always has a short as its operand.
                    let offset = self.get_next_short().unwrap() as usize;
                    self.ip += offset;
                }

                OpCode::OP_LOOP_JUMP | OpCode::OP_LOOP_JUMP_LONG => {
                    let offset = if let OpCode::OP_LOOP_JUMP = instruction {
                        self.get_next_byte().unwrap() as usize
                    } else {
                        self.get_next_short().unwrap() as usize
                    };

                    self.ip -= offset;
                }

                OpCode::OP_POST_INCREMENT => {
                    let pos = if let OpCode::OP_POST_INCREMENT = instruction {
                        self.get_next_byte().unwrap() as usize
                    } else {
                        self.get_next_short().unwrap() as usize
                    };

                    let value = Rc::clone(&self.stack.get_mut(pos).unwrap());

                    if !value.is_numeric() {
                        self.report_runtime_error(&format!("Cannot increment object of type '{}'.", value.type_name()));
                        return InterpretResult::INTERPRET_RUNTIME_ERROR;
                    }

                    self.stack[pos] = Rc::new(Object::Number(value.as_number().unwrap() + 1f64));
                    self.stack.push(value);
                }

                OpCode::OP_POST_DECREMENT => {
                    let pos = if let OpCode::OP_POST_DECREMENT = instruction {
                        self.get_next_byte().unwrap() as usize
                    } else {
                        self.get_next_short().unwrap() as usize
                    };

                    let value = Rc::clone(&self.stack.get_mut(pos).unwrap());

                    if !value.is_numeric() {
                        self.report_runtime_error(&format!("Cannot decrement object of type '{}'.", value.type_name()));
                        return InterpretResult::INTERPRET_RUNTIME_ERROR;
                    }

                    self.stack[pos] = Rc::new(Object::Number(value.as_number().unwrap() - 1f64));
                    self.stack.push(value);
                }

                OpCode::OP_PRINT => {
                    let val = self.stack.pop();
                    println!("{}", val.unwrap());
                }

                // OpCode::OP_RETURN => { }

                // Should be removed so that there are no unimplemented OpCodes
                _ => unreachable!("OpCode Not implemented! {:?}", instruction),
            }
        }

        // If the compiler reaches this point, that means there were no errors
        // to return (because errors are returned by the match rules), so we can
        // safely return an `INTERPRET_OK` result.
        return InterpretResult::INTERPRET_OK;
    }

    /// Gets the next OpCode to be executed, incrementing the
    /// instruction pointer by one.
    ///
    /// ## Returns
    /// * `Option<OpCode>` – The next OpCode to be executed, if the
    /// instruction pointer is within bounds.
    fn get_next_op_code(&mut self) -> Option<OpCode> {
        let code = self.chunk.codes.get_op_code(self.ip);
        self.ip += 1;
        return code;
    }

    /// Gets the next byte to be executed, incrementing the
    /// instruction pointer by one.
    ///
    /// ## Returns
    /// * `Option<OpCode>` – The next byte to be executed, if the
    /// instruction pointer is within bounds.
    fn get_next_byte(&mut self) -> Option<u8> {
        let code = self.chunk.codes.get_byte(self.ip);
        self.ip += 1;
        return code;
    }

    /// Gets the next short (next two bytes) to be executed, incrementing the
    /// instruction pointer by 2.
    ///
    /// ## Returns
    /// * `Option<u16>` – The next two bytes as a 16-bit unsigned integer, if the
    /// instruction pointer is within bounds.
    fn get_next_short(&mut self) -> Option<u16> {
        let next_short = self.chunk.codes.get_short(self.ip);
        self.ip += 2;
        return next_short;
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
        println!("IP:\t{:>04} ", self.ip);

        // Prints the current state of the values stack
        print!("stack\t[");
        for val in self.stack.iter() {
            print!("{}; ", val);
        }
        println!("]");

        // Prints the number of objects currently present in the heap
        println!("Heap Size: {}", self.chunk.constants.len());

        print!("Output:\t");
    }
}
