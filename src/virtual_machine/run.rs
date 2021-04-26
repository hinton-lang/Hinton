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

                OpCode::OP_VALUE => {
                    let pos = match self.get_next_short() {
                        Some(short) => short,
                        None => {
                            self.report_runtime_error("Unexpected Runtime Error: Could not get next short.");
                            return InterpretResult::INTERPRET_RUNTIME_ERROR;
                        }
                    };

                    match self.chunk.get_constant(pos) {
                        Some(val) => self.stack.push(Rc::clone(val)),
                        None => {
                            self.report_runtime_error(&format!("InternalRuntimeError: Constant pool index '{}' out of range", pos));
                            return InterpretResult::INTERPRET_RUNTIME_ERROR;
                        }
                    }
                }

                OpCode::OP_GET_VAR => {
                    // The position of the local variable's value in the stack
                    let pos = match self.get_next_short() {
                        Some(short) => short as usize,
                        None => {
                            self.report_runtime_error("Unexpected Runtime Error: Could not get next short.");
                            return InterpretResult::INTERPRET_RUNTIME_ERROR;
                        }
                    };

                    let value = Rc::clone(&self.stack.get_mut(pos).unwrap());

                    self.stack.push(value);
                }

                OpCode::OP_SET_VAR => {
                    // The position of the local variable's value in the stack
                    let pos = match self.get_next_short() {
                        Some(short) => short as usize,
                        None => {
                            self.report_runtime_error("Unexpected Runtime Error: Could not get next short.");
                            return InterpretResult::INTERPRET_RUNTIME_ERROR;
                        }
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
                    let offset = match self.get_next_short() {
                        Some(short) => short as usize,
                        None => {
                            self.report_runtime_error("Unexpected Runtime Error: Could not get next short.");
                            return InterpretResult::INTERPRET_RUNTIME_ERROR;
                        }
                    };

                    let top = Rc::clone(&self.stack.last().unwrap());

                    if top.is_falsey() {
                        self.ip += offset;
                    }
                }

                OpCode::OP_JUMP => {
                    let offset = match self.get_next_short() {
                        Some(short) => short as usize,
                        None => {
                            self.report_runtime_error("Unexpected Runtime Error: Could not get next short.");
                            return InterpretResult::INTERPRET_RUNTIME_ERROR;
                        }
                    };

                    self.ip += offset;
                }

                OpCode::OP_LOOP => {
                    let offset = match self.get_next_short() {
                        Some(short) => short as usize,
                        None => {
                            self.report_runtime_error("Unexpected Runtime Error: Could not get next short.");
                            return InterpretResult::INTERPRET_RUNTIME_ERROR;
                        }
                    };

                    self.ip -= offset;
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

        print!("Output:\t");
    }
}
