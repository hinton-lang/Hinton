use std::rc::Rc;

use super::InterpretResult;
use super::VirtualMachine;
use crate::objects::Object;
use crate::{chunk::op_codes::OpCode, objects::RangeObject};

impl<'a> VirtualMachine<'a> {
    /// Executes the instruction in a chunk. This is where the interpreter
    /// will spend most its life, therefore, optimizing every function in this file
    /// is very important.
    ///
    /// ## Returns
    /// `InterpretResult` – The result of the execution.
    pub(crate) fn run(&'a mut self) -> InterpretResult {
        let frame = self.frames.get(self.frames.len() - 1).unwrap();
        let mut frame_ip = frame.ip;

        loop {
            let current_chunk = &frame.function.chunk;
            let instruction = current_chunk.codes.get_op_code(frame_ip);

            // Executes the instructions one by one
            match instruction {
                Some(OpCode::OP_POP_STACK) => {
                    self.stack.pop();
                }

                Some(OpCode::OP_NULL) => self.stack.push(Rc::new(Object::Null())),
                Some(OpCode::OP_TRUE) => self.stack.push(Rc::new(Object::Bool(true))),
                Some(OpCode::OP_FALSE) => self.stack.push(Rc::new(Object::Bool(false))),

                Some(OpCode::OP_CONSTANT) => {
                    frame_ip += 1;
                    let pos = current_chunk.codes.get_short(frame_ip);
                    frame_ip += 1;

                    match current_chunk.get_constant(pos) {
                        Some(val) => self.stack.push(Rc::clone(val)),
                        None => {
                            self.report_runtime_error(&format!("InternalRuntimeError: Constant pool index '{}' out of range", pos));
                            return InterpretResult::INTERPRET_RUNTIME_ERROR;
                        }
                    }
                }

                Some(OpCode::OP_DEFINE_GLOBAL_VAR) => {
                    frame_ip += 1;
                    let pos = current_chunk.codes.get_short(frame_ip);
                    frame_ip += 1;

                    match current_chunk.get_constant(pos) {
                        Some(val) => {
                            if val.is_string() {
                                let str = val.as_string().unwrap();
                                self.globals
                                    .insert(String::from(str.as_str()), Rc::clone(self.stack.get(self.stack.len() - 1).unwrap()));

                                // We pop the top off the stack here in case the garbage collector
                                // was in the middle of an operation when we defined the variable
                                self.stack.pop();
                            } else {
                                self.report_runtime_error("InternalRuntimeError: Pool item is not an identifier");
                            }
                        }
                        None => {
                            self.report_runtime_error(&format!("InternalRuntimeError: Constant pool index '{}' out of range", pos));
                            return InterpretResult::INTERPRET_RUNTIME_ERROR;
                        }
                    }
                }

                Some(OpCode::OP_GET_GLOBAL_VAR) => {
                    frame_ip += 1;
                    let pos = current_chunk.codes.get_short(frame_ip);
                    frame_ip += 1;

                    match current_chunk.get_constant(pos) {
                        Some(val) => {
                            if val.is_string() {
                                let str = val.as_string().unwrap();

                                match self.globals.get(&str) {
                                    Some(obj) => self.stack.push(Rc::clone(obj)),
                                    None => {
                                        self.report_runtime_error(&format!("Undefined variable '{}'.", str));
                                    }
                                }
                            } else {
                                self.report_runtime_error("InternalRuntimeError: Pool item is not an identifier");
                            }
                        }
                        None => {
                            self.report_runtime_error(&format!("InternalRuntimeError: Constant pool index '{}' out of range", pos));
                            return InterpretResult::INTERPRET_RUNTIME_ERROR;
                        }
                    }
                }

                Some(OpCode::OP_NEGATE) => {
                    let val = Rc::clone(&self.stack.pop().unwrap());

                    if !val.is_numeric() {
                        self.report_runtime_error(&format!("Cannot negate operand of type '{}'.", val.type_name()));
                        return InterpretResult::INTERPRET_RUNTIME_ERROR;
                    } else {
                        self.stack.push(Rc::new(Object::Number(-val.as_number().unwrap())));
                    }
                }

                Some(OpCode::OP_ADD) => {
                    let val2 = Rc::clone(&self.stack.pop().unwrap());
                    let val1 = Rc::clone(&self.stack.pop().unwrap());

                    if (val1.is_string() && val2.is_stringifyable()) || (val1.is_stringifyable() && val2.is_string()) {
                        let v1 = val1.as_string().unwrap();
                        let v2 = val2.as_string().unwrap();
                        self.stack.push(Rc::new(Object::String(v1 + v2.as_str())));
                    } else {
                        let numeric = self.check_numeric_operands(Rc::clone(&val1), Rc::clone(&val2), "+");
                        if numeric {
                            let v1 = val1.as_number().unwrap();
                            let v2 = val2.as_number().unwrap();
                            self.stack.push(Rc::new(Object::Number(v1 + v2)));
                        } else {
                            return InterpretResult::INTERPRET_RUNTIME_ERROR;
                        }
                    }
                }

                Some(OpCode::OP_SUBTRACT) => {
                    let val2 = Rc::clone(&self.stack.pop().unwrap());
                    let val1 = Rc::clone(&self.stack.pop().unwrap());
                    let numeric = self.check_numeric_operands(Rc::clone(&val1), Rc::clone(&val2), "-");

                    if numeric {
                        let v1 = val1.as_number().unwrap();
                        let v2 = val2.as_number().unwrap();
                        self.stack.push(Rc::new(Object::Number(v1 - v2)));
                    } else {
                        return InterpretResult::INTERPRET_RUNTIME_ERROR;
                    }
                }

                Some(OpCode::OP_MULTIPLY) => {
                    let val2 = Rc::clone(&self.stack.pop().unwrap());
                    let val1 = Rc::clone(&self.stack.pop().unwrap());

                    // Multiply Int * String
                    if (val1.is_numeric() && val2.is_string()) || (val1.is_string() && val2.is_numeric()) {
                        if val1.is_int() {
                            let r = val1.as_number().unwrap() as isize;
                            let s = if r >= 0 {
                                val2.as_string().unwrap().repeat(r as usize)
                            } else {
                                String::from("")
                            };
                            self.stack.push(Rc::new(Object::String(s)));
                        } else if val2.is_int() {
                            let r = val2.as_number().unwrap() as isize;
                            let s = if r >= 0 {
                                val1.as_string().unwrap().repeat(r as usize)
                            } else {
                                String::from("")
                            };
                            self.stack.push(Rc::new(Object::String(s)));
                        } else {
                            self.report_runtime_error(&format!(
                                "Operation '*' not defined for operands of type '{}' and '{}'.",
                                val1.type_name(),
                                val2.type_name()
                            ));
                            return InterpretResult::INTERPRET_RUNTIME_ERROR;
                        }
                    } else {
                        let numeric = self.check_numeric_operands(Rc::clone(&val1), Rc::clone(&val2), "*");
                        if numeric {
                            let v1 = val1.as_number().unwrap();
                            let v2 = val2.as_number().unwrap();
                            self.stack.push(Rc::new(Object::Number(v1 * v2)));
                        } else {
                            return InterpretResult::INTERPRET_RUNTIME_ERROR;
                        }
                    }
                }

                Some(OpCode::OP_DIVIDE) => {
                    let val2 = Rc::clone(&self.stack.pop().unwrap());
                    let val1 = Rc::clone(&self.stack.pop().unwrap());
                    let numeric = self.check_numeric_operands(Rc::clone(&val1), Rc::clone(&val2), "/");

                    if numeric {
                        let v1 = val1.as_number().unwrap();
                        let v2 = val2.as_number().unwrap();

                        if v2 == 0.0 {
                            self.report_runtime_error("Cannot divide by zero");
                            return InterpretResult::INTERPRET_RUNTIME_ERROR;
                        }

                        self.stack.push(Rc::new(Object::Number(v1 / v2)));
                    } else {
                        return InterpretResult::INTERPRET_RUNTIME_ERROR;
                    }
                }

                Some(OpCode::OP_MODULUS) => {
                    let val2 = Rc::clone(&self.stack.pop().unwrap());
                    let val1 = Rc::clone(&self.stack.pop().unwrap());
                    let numeric = self.check_numeric_operands(Rc::clone(&val1), Rc::clone(&val2), "%");

                    if numeric {
                        let v1 = val1.as_number().unwrap();
                        let v2 = val2.as_number().unwrap();

                        if v2 == 0.0 {
                            self.report_runtime_error("Right-hand-size of modulus cannot be zero.");
                            return InterpretResult::INTERPRET_RUNTIME_ERROR;
                        }

                        self.stack.push(Rc::new(Object::Number(v1 % v2)));
                    } else {
                        return InterpretResult::INTERPRET_RUNTIME_ERROR;
                    }
                }

                Some(OpCode::OP_EXPO) => {
                    let val2 = Rc::clone(&self.stack.pop().unwrap());
                    let val1 = Rc::clone(&self.stack.pop().unwrap());
                    let numeric = self.check_numeric_operands(Rc::clone(&val1), Rc::clone(&val2), "**");

                    if numeric {
                        let v1 = val1.as_number().unwrap();
                        let v2 = val2.as_number().unwrap();
                        self.stack.push(Rc::new(Object::Number(v1.powf(v2))));
                    } else {
                        return InterpretResult::INTERPRET_RUNTIME_ERROR;
                    }
                }

                Some(OpCode::OP_LOGIC_NOT) => {
                    let val = Rc::clone(&self.stack.pop().unwrap());
                    self.stack.push(Rc::new(Object::Bool(val.is_falsey())));
                }

                Some(OpCode::OP_EQUALS) => {
                    let val2 = Rc::clone(&self.stack.pop().unwrap());
                    let val1 = Rc::clone(&self.stack.pop().unwrap());
                    self.stack.push(Rc::new(Object::Bool(val1.equals(val2))));
                }

                Some(OpCode::OP_NOT_EQUALS) => {
                    let val2 = Rc::clone(&self.stack.pop().unwrap());
                    let val1 = Rc::clone(&self.stack.pop().unwrap());
                    self.stack.push(Rc::new(Object::Bool(!val1.equals(val2))));
                }

                Some(OpCode::OP_LESS_THAN) => {
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

                Some(OpCode::OP_LESS_THAN_EQ) => {
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

                Some(OpCode::OP_GREATER_THAN) => {
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

                Some(OpCode::OP_GREATER_THAN_EQ) => {
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

                Some(OpCode::OP_BITWISE_OR) => {
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

                Some(OpCode::OP_BITWISE_XOR) => {
                    let right = Rc::clone(&self.stack.pop().unwrap());
                    let left = Rc::clone(&self.stack.pop().unwrap());

                    if self.check_integer_operands(Rc::clone(&left), Rc::clone(&right), "|") {
                        self.stack.push(Rc::new(Object::Number(
                            ((left.as_number().unwrap() as isize) ^ (right.as_number().unwrap() as isize)) as f64,
                        )));
                    } else {
                        return InterpretResult::INTERPRET_RUNTIME_ERROR;
                    }
                }

                Some(OpCode::OP_BITWISE_AND) => {
                    let right = Rc::clone(&self.stack.pop().unwrap());
                    let left = Rc::clone(&self.stack.pop().unwrap());

                    if self.check_integer_operands(Rc::clone(&left), Rc::clone(&right), "|") {
                        self.stack.push(Rc::new(Object::Number(
                            ((left.as_number().unwrap() as isize) & (right.as_number().unwrap() as isize)) as f64,
                        )));
                    } else {
                        return InterpretResult::INTERPRET_RUNTIME_ERROR;
                    }
                }

                Some(OpCode::OP_BITWISE_NOT) => {
                    let operand = Rc::clone(&self.stack.pop().unwrap());

                    if operand.is_numeric() {
                        self.stack.push(Rc::new(Object::Number(!(operand.as_number().unwrap() as isize) as f64)));
                    } else {
                        self.report_runtime_error(&format!("Operation '~' not defined for operand of type '{}'.", operand.type_name()));
                        return InterpretResult::INTERPRET_RUNTIME_ERROR;
                    }
                }

                Some(OpCode::OP_BITWISE_L_SHIFT) => {
                    let right = Rc::clone(&self.stack.pop().unwrap());
                    let left = Rc::clone(&self.stack.pop().unwrap());

                    if self.check_integer_operands(Rc::clone(&left), Rc::clone(&right), "|") {
                        self.stack.push(Rc::new(Object::Number(
                            ((left.as_number().unwrap() as isize) << (right.as_number().unwrap() as isize)) as f64,
                        )));
                    } else {
                        return InterpretResult::INTERPRET_RUNTIME_ERROR;
                    }
                }

                Some(OpCode::OP_BITWISE_R_SHIFT) => {
                    let right = Rc::clone(&self.stack.pop().unwrap());
                    let left = Rc::clone(&self.stack.pop().unwrap());

                    if self.check_integer_operands(Rc::clone(&left), Rc::clone(&right), "|") {
                        self.stack.push(Rc::new(Object::Number(
                            ((left.as_number().unwrap() as isize) >> (right.as_number().unwrap() as isize)) as f64,
                        )));
                    } else {
                        return InterpretResult::INTERPRET_RUNTIME_ERROR;
                    }
                }

                Some(OpCode::OP_GENERATE_RANGE) => {
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

                Some(OpCode::OP_TERNARY) => {
                    let branch_else = Rc::clone(&self.stack.pop().unwrap());
                    let branch_true = Rc::clone(&self.stack.pop().unwrap());
                    let condition = Rc::clone(&self.stack.pop().unwrap());

                    if !condition.is_falsey() {
                        self.stack.push(branch_true);
                    } else {
                        self.stack.push(branch_else);
                    }
                }

                Some(OpCode::OP_NULLISH_COALESCING) => {
                    let value = Rc::clone(&self.stack.pop().unwrap());
                    let nullish = Rc::clone(&self.stack.pop().unwrap());

                    if nullish.is_null() {
                        self.stack.push(value);
                    } else {
                        self.stack.push(nullish);
                    }
                }

                Some(OpCode::OP_PRINT) => {
                    let val = self.stack.pop();
                    println!("{}", val.unwrap());
                }

                Some(OpCode::OP_RETURN) => {
                    return InterpretResult::INTERPRET_OK;
                }

                None => {
                    self.report_runtime_error("InternalRuntimeError: No OpCode to execute.");
                    return InterpretResult::INTERPRET_RUNTIME_ERROR;
                }

                // Should be removed so that there are no unimplemented OpCodes
                _ => {
                    self.report_runtime_error(&format!("InternalRuntimeError: Unknown OpCode {:?}.", instruction));
                    return InterpretResult::INTERPRET_RUNTIME_ERROR;
                }
            }

            // Increment the instruction pointer
            frame_ip += 1;
        }
    }
}
