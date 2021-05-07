use crate::objects::Object;

use super::{InterpretResult, VirtualMachine};

impl<'a> VirtualMachine {
    /// Adds the two values on top of the stack according to Hinton's addition rules.
    ///
    /// ## Returns
    /// * `Result<(), InterpretResult>` – Nothing if the values could be successfully added;
    /// `InterpretResult::INTERPRET_RUNTIME_ERROR` otherwise.
    pub(super) fn perform_addition(&mut self) -> Result<(), InterpretResult> {
        let val2 = self.stack.pop().unwrap();
        let val1 = self.stack.pop().unwrap();

        if (val1.is_string() && val2.is_stringifyable()) || (val1.is_stringifyable() && val2.is_string()) {
            let v1 = val1.as_string().unwrap();
            let v2 = val2.as_string().unwrap();
            self.stack.push(Object::String(v1 + v2.as_str()));

            Ok(())
        } else {
            let numeric = self.check_numeric_operands(&val1, &val2, "+");
            if numeric {
                let v1 = val1.as_number().unwrap();
                let v2 = val2.as_number().unwrap();
                self.stack.push(Object::Number(v1 + v2));

                Ok(())
            } else {
                return Err(InterpretResult::RuntimeError);
            }
        }
    }

    /// Subtracts the two values on top of the stack according to Hinton's subtraction rules.
    ///
    /// ## Returns
    /// * `Result<(), InterpretResult>` – Nothing if the values could be successfully subtracted;
    /// `InterpretResult::INTERPRET_RUNTIME_ERROR` otherwise.
    pub(super) fn perform_subtraction(&mut self) -> Result<(), InterpretResult> {
        let val2 = self.stack.pop().unwrap();
        let val1 = self.stack.pop().unwrap();
        let numeric = self.check_numeric_operands(&val1, &val2, "-");

        if numeric {
            let v1 = val1.as_number().unwrap();
            let v2 = val2.as_number().unwrap();
            self.stack.push(Object::Number(v1 - v2));

            Ok(())
        } else {
            return Err(InterpretResult::RuntimeError);
        }
    }

    /// Multiplies the two values on top of the stack according to Hinton's multiplication rules.
    ///
    /// ## Returns
    /// * `Result<(), InterpretResult>` – Nothing if the values could be successfully multiplied;
    /// `InterpretResult::INTERPRET_RUNTIME_ERROR` otherwise.
    pub(super) fn perform_multiplication(&mut self) -> Result<(), InterpretResult> {
        let val2 = self.stack.pop().unwrap();
        let val1 = self.stack.pop().unwrap();

        // Multiply Int * String
        if (val1.is_numeric() && val2.is_string()) || (val1.is_string() && val2.is_numeric()) {
            if val1.is_int() {
                let r = val1.as_number().unwrap() as isize;
                let s = if r >= 0 {
                    val2.as_string().unwrap().repeat(r as usize)
                } else {
                    String::from("")
                };
                self.stack.push(Object::String(s));

                Ok(())
            } else if val2.is_int() {
                let r = val2.as_number().unwrap() as isize;
                let s = if r >= 0 {
                    val1.as_string().unwrap().repeat(r as usize)
                } else {
                    String::from("")
                };
                self.stack.push(Object::String(s));

                Ok(())
            } else {
                self.report_runtime_error(&format!(
                    "Operation '*' not defined for operands of type '{}' and '{}'.",
                    val1.type_name(),
                    val2.type_name()
                ));
                return Err(InterpretResult::RuntimeError);
            }
        } else {
            let numeric = self.check_numeric_operands(&val1, &val2, "*");
            if numeric {
                let v1 = val1.as_number().unwrap();
                let v2 = val2.as_number().unwrap();
                self.stack.push(Object::Number(v1 * v2));

                Ok(())
            } else {
                return Err(InterpretResult::RuntimeError);
            }
        }
    }

    /// Divides the two values on top of the stack according to Hinton's division rules.
    ///
    /// ## Returns
    /// * `Result<(), InterpretResult>` – Nothing if the values could be successfully divided;
    /// `InterpretResult::INTERPRET_RUNTIME_ERROR` otherwise.
    pub(super) fn perform_division(&mut self) -> Result<(), InterpretResult> {
        let val2 = self.stack.pop().unwrap();
        let val1 = self.stack.pop().unwrap();
        let numeric = self.check_numeric_operands(&val1, &val2, "/");

        if numeric {
            let v1 = val1.as_number().unwrap();
            let v2 = val2.as_number().unwrap();

            if v2 == 0.0 {
                self.report_runtime_error("Cannot divide by zero");
                return Err(InterpretResult::RuntimeError);
            }

            self.stack.push(Object::Number(v1 / v2));
            Ok(())
        } else {
            return Err(InterpretResult::RuntimeError);
        }
    }

    /// Calculates the modulus of the two values on top of the stack.
    ///
    /// ## Returns
    /// * `Result<(), InterpretResult>` – Nothing if the modulus of the two values could be
    /// successfully computed; `InterpretResult::INTERPRET_RUNTIME_ERROR` otherwise.
    pub(super) fn perform_modulus(&mut self) -> Result<(), InterpretResult> {
        let val2 = self.stack.pop().unwrap();
        let val1 = self.stack.pop().unwrap();
        let numeric = self.check_numeric_operands(&val1, &val2, "%");

        if numeric {
            let v1 = val1.as_number().unwrap();
            let v2 = val2.as_number().unwrap();

            if v2 == 0.0 {
                self.report_runtime_error("Right-hand-size of modulus cannot be zero.");
                return Err(InterpretResult::RuntimeError);
            }

            self.stack.push(Object::Number(v1 % v2));

            Ok(())
        } else {
            return Err(InterpretResult::RuntimeError);
        }
    }

    /// Exponentiates the two values on top of the stack according to Hinton's exponentiation rules.
    ///
    /// ## Returns
    /// * `Result<(), InterpretResult>` – Nothing if the values could be successfully exponentiated;
    /// `InterpretResult::INTERPRET_RUNTIME_ERROR` otherwise.
    pub(super) fn perform_exponentiation(&mut self) -> Result<(), InterpretResult> {
        let val2 = self.stack.pop().unwrap();
        let val1 = self.stack.pop().unwrap();
        let numeric = self.check_numeric_operands(&val1, &val2, "**");

        if numeric {
            let v1 = val1.as_number().unwrap();
            let v2 = val2.as_number().unwrap();
            self.stack.push(Object::Number(v1.powf(v2)));

            Ok(())
        } else {
            return Err(InterpretResult::RuntimeError);
        }
    }
}
