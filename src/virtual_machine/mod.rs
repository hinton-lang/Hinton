#[cfg(feature = "bench_time")]
use std::time::Instant;

use std::rc::Rc;

mod arithmetic;
mod run;

use crate::{
    chunk::op_codes::OpCode,
    exec_time,
    intermediate::{compiler::Compiler, parser::Parser},
    objects::{self, FunctionObject, Object},
};

/// The types of results the interpreter can return
#[allow(non_camel_case_types)]
pub enum InterpretResult {
    INTERPRET_OK,
    INTERPRET_PARSE_ERROR,
    INTERPRET_COMPILE_ERROR,
    INTERPRET_RUNTIME_ERROR,
}

/// Represents a single ongoing function call.
pub struct CallFrame {
    function: FunctionObject,
    /// IP of the CallFrame's chunk
    ip: usize,
    /// The offset of this CallFrame in the VM's stack
    slots_base: usize,
}

impl CallFrame {
    /// Gets the next OpCode to be executed, incrementing the
    /// instruction pointer by one.
    ///
    /// ## Returns
    /// * `Option<OpCode>` – The next OpCode to be executed, if the
    /// instruction pointer is within bounds.
    fn get_next_op_code(&mut self) -> Option<OpCode> {
        let code = self.function.chunk.codes.get_op_code(self.ip);
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
        let code = self.function.chunk.codes.get_byte(self.ip);
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
        let next_short = self.function.chunk.codes.get_short(self.ip);
        self.ip += 2;
        return next_short;
    }

    fn get_constant(&self, idx: usize) -> &Object {
        self.function.chunk.get_constant(idx).unwrap()
    }
}

/// Represents a virtual machine
pub struct VirtualMachine {
    stack: Vec<objects::Object>,
    frames: Vec<CallFrame>,
}

impl<'a> VirtualMachine {
    /// Creates a new instance of the virtual machine.
    ///
    /// ## Returns
    /// * `VirtualMachine` – a new instance of the virtual machine.
    pub fn new() -> Self {
        Self {
            stack: vec![],
            frames: vec![],
        }
    }

    /// Interprets a chuck of code.
    ///
    /// ## Returns
    /// * `InterpretResult` – The result of the source interpretation.
    pub(crate) fn interpret(&mut self, source: &'a str) -> InterpretResult {
        // Parses the program into an AST and calculates the parser's execution time
        let parsing = exec_time(|| Parser::parse(source));

        let ast = match parsing.0 {
            Ok(x) => Rc::new(x),
            Err(e) => return e,
        };

        // This is where different static analysis of the
        // AST would take place
        // analyzer::analyze_module(Rc::clone(&ast));

        // Compiles the program into bytecode and calculates the compiler's execution time
        let compiling = exec_time(|| Compiler::compile(Rc::clone(&ast)));

        // Executes the program
        return match compiling.0 {
            Ok(c) => {
                self.stack.push(Object::Function(c.clone()));
                self.frames.push(CallFrame {
                    function: c,
                    ip: 0,
                    slots_base: 0,
                });

                #[cfg(feature = "bench_time")]
                let start = Instant::now();

                // Rus the program.
                let runtime_result = self.run();

                #[cfg(feature = "bench_time")]
                {
                    let run_time = start.elapsed();

                    println!("\n======= ⚠️  Execution Results ⚠️  =======");
                    println!("Parse Time:\t{:?}", parsing.1);
                    println!("Compile Time:\t{:?}", compiling.1);
                    println!("Run Time:\t{:?}", run_time);
                    println!("=======================================");
                }

                return runtime_result;
            }
            Err(e) => e,
        };
    }

    /// Throws a runtime error to the console
    pub(super) fn report_runtime_error(&self, message: &'a str) {
        let frame = self.frames.last().unwrap();

        let line = frame.function.chunk.locations.get(frame.ip).unwrap();
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
    pub(super) fn check_numeric_operands(&self, left: &Object, right: &Object, opr: &str) -> bool {
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
    pub(super) fn check_integer_operands(&self, left: &Object, right: &Object, opr: &str) -> bool {
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
