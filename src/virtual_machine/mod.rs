pub mod run;
use std::{collections::HashMap, rc::Rc};

use crate::{
    compiler::Compiler,
    objects::{self, CallFrame, Object},
};

/// The types of results the interpreter can return
#[allow(non_camel_case_types)]
#[derive(PartialEq)]
pub enum InterpretResult {
    INTERPRET_OK,
    INTERPRET_COMPILE_ERROR,
    INTERPRET_RUNTIME_ERROR,
}

/// Represents a virtual machine
pub struct VirtualMachine<'a> {
    // is_in_global_frame: bool,
    frames: Vec<objects::CallFrame<'a>>,
    stack: Vec<Rc<objects::Object<'a>>>,
    globals: HashMap<String, Rc<objects::Object<'a>>>
}

impl<'a> VirtualMachine<'a> {
    /// Creates a new instance of the virtual machine.
    ///
    /// ## Returns
    /// * `VirtualMachine` – a new instance of the virtual machine.
    pub fn new() -> Self {
        Self {
            frames: Vec::new(),
            stack: Vec::new(),
            globals: HashMap::new(),
        }
    }

    /// Interprets a chuck of code.
    ///
    /// ## Returns
    /// * `InterpretResult` – The result of the source interpretation.
    pub(crate) fn interpret(&'a mut self, source: &'a str) -> InterpretResult {
        return match Compiler::compile(source) {
            Ok(c) => {
                let c = Rc::new(c);

                self.stack.push(Rc::new(Object::Function(Rc::clone(&c))));

                self.frames.push(CallFrame { function: c, ip: 0 });

                return self.run();
                // return InterpretResult::INTERPRET_OK;
            }
            Err(e) => e,
        };
    }

    /// Throws a runtime error to the console
    pub(super) fn report_runtime_error(&self, message: &'a str) {
        let frame = self.frames.get(self.frames.len() - 1).unwrap();
        let line = frame.function.chunk.locations.get(frame.ip + 1).unwrap();

        eprintln!("RuntimeError at [{}:{}] – {}", line.0, line.1, message);
    }

    /// Checks that both operands of a binary operand are numeric.
    ///
    /// ## Arguments
    /// * `left` – The left operand.
    /// * `right` – The right operand.
    /// * `operator` – A string representation of the operator (for error reporting)
    ///
    /// ## Returns
    /// `bool` – True if both operands are numeric, false otherwise.
    pub(super) fn check_numeric_operands(&self, left: Rc<Object<'a>>, right: Rc<Object<'a>>, opr: &str) -> bool {
        // If the operands are not numeric, throw a runtime error.
        if !left.is_numeric() || !right.is_numeric() {
            self.report_runtime_error(&format!(
                "Operation '{}' not defined for operands of type '{}' and '{}'.",
                opr.to_string(),
                left.type_name(),
                right.type_name()
            ));
            return false;
        }

        return true;
    }

    /// Checks that both operands of a binary operand are numeric.
    ///
    /// ## Arguments
    /// * `left` – The left operand.
    /// * `right` – The right operand.
    /// * `operator` – A string representation of the operator (for error reporting)
    ///
    /// ## Returns
    /// `bool` – True if both operands are numeric, false otherwise.
    pub(super) fn check_integer_operands(&self, left: Rc<Object<'a>>, right: Rc<Object<'a>>, opr: &str) -> bool {
        // If the operands are not numeric, throw a runtime error.
        if !left.is_int() || !right.is_int() {
            self.report_runtime_error(&format!(
                "Operation '{}' not defined for operands of type '{}' and '{}'.",
                opr.to_string(),
                left.type_name(),
                right.type_name()
            ));
            return false;
        }

        return true;
    }
}
