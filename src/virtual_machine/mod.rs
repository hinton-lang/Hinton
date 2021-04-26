use std::rc::Rc;

mod arithmetic;
mod run;

use crate::{
    chunk,
    intermediate::{compiler::Compiler, parser::Parser},
    objects::{self, Object},
};

/// The types of results the interpreter can return
#[allow(non_camel_case_types)]
pub enum InterpretResult {
    INTERPRET_OK,
    INTERPRET_PARSE_ERROR,
    INTERPRET_COMPILE_ERROR,
    INTERPRET_RUNTIME_ERROR,
}

/// Represents a virtual machine
pub struct VirtualMachine {
    stack: Vec<Rc<objects::Object>>,
    chunk: chunk::Chunk,
    ip: usize,
}

impl<'a> VirtualMachine {
    /// Creates a new instance of the virtual machine.
    ///
    /// ## Returns
    /// * `VirtualMachine` – a new instance of the virtual machine.
    pub fn new() -> Self {
        Self {
            stack: Vec::new(),
            chunk: chunk::Chunk::new(),
            ip: 0,
        }
    }

    /// Interprets a chuck of code.
    ///
    /// ## Returns
    /// * `InterpretResult` – The result of the source interpretation.
    pub(crate) fn interpret(&mut self, source: &'a str) -> InterpretResult {
        // Parses the program
        let ast = match Parser::parse(source) {
            Ok(x) => Rc::new(x),
            Err(e) => return e,
        };

        // This is where different static analysis of the
        // AST would take place
        // analyzer::analyze_module(Rc::clone(&ast));

        // Executes the program after it has been compiled to ByteCode
        return match Compiler::compile(ast) {
            Ok(c) => {
                self.chunk = c;
                return self.run();
            }
            Err(e) => e,
        };
    }

    /// Throws a runtime error to the console
    pub(super) fn report_runtime_error(&self, message: &'a str) {
        let line = self.chunk.locations.get(self.ip).unwrap();
        eprintln!("\x1b[31;1mRuntimeError\x1b[0m at [{}:{}] – {}", line.0, line.1, message);
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
    pub(super) fn check_numeric_operands(&self, left: Rc<Object>, right: Rc<Object>, opr: &str) -> bool {
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
    pub(super) fn check_integer_operands(&self, left: Rc<Object>, right: Rc<Object>, opr: &str) -> bool {
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
