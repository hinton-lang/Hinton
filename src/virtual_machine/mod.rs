pub mod run;
use std::{collections::HashMap, rc::Rc};

use crate::{analyzer, intermediate::{compiler::Compiler, parser::Parser}, objects::{self, FunctionObject, Object}};

/// The types of results the interpreter can return
#[allow(non_camel_case_types)]
#[derive(PartialEq)]
pub enum InterpretResult {
    INTERPRET_OK,
    INTERPRET_PARSE_ERROR,
    INTERPRET_COMPILE_ERROR,
    INTERPRET_RUNTIME_ERROR,
}

/// Represents a function call frame
pub struct CallFrame<'a> {
    /// The function chunk associated with this call frame
    pub function: Rc<FunctionObject<'a>>,
    // The instruction pointer for this call frame
    pub ip: usize,
    // TODO: What does this do?
    // pub slots: Vec<Object<'a>>
}

/// Represents a virtual machine
pub struct VirtualMachine<'a> {
    frames: Vec<CallFrame<'a>>,
    stack: Vec<Rc<objects::Object<'a>>>,
    globals: HashMap<String, Rc<objects::Object<'a>>>,
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
        // Parses the program
        let ast = match Parser::parse(source) {
            Ok(x) => Rc::new(x),
            Err(e) => return e,
        };

        // This is where different static analysis of the
        // AST would take place
        analyzer::analyze_module(Rc::clone(&ast));

        // Executes the program after it has been compiled to ByteCode
        return match Compiler::compile(ast) {
            Ok(c) => {
                let c = Rc::new(c);
                self.stack.push(Rc::new(Object::Function(Rc::clone(&c))));
                self.frames.push(CallFrame { function: c, ip: 0 });
                return self.run();
            }
            Err(e) => e,
        };
    }

    /// Throws a runtime error to the console
    pub(super) fn report_runtime_error(&self, message: &'a str) {
        let frame = self.frames.get(self.frames.len() - 1).unwrap();
        // TODO: The frame's IP is not getting updated in the `run.rs`'s loop.
        // This will show the wrong line and column when an error occurs.
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
