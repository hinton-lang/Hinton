#[cfg(feature = "bench_time")]
use std::time::Instant;

use crate::{
    chunk::OpCode,
    compiler::Compiler,
    exec_time,
    natives::NativeFunctions,
    objects::{self, FunctionObject, Object},
    parser::Parser,
};
use std::rc::Rc;

// Submodules
mod run;

/// The types of results the interpreter can return
pub enum InterpretResult {
    CompileError,
    Ok,
    ParseError,
    RuntimeError,
}

/// Represents a single ongoing function call.
pub struct CallFrame {
    function: FunctionObject,
    ip: usize,
    base_pointer: usize,
}

impl CallFrame {
    fn get_next_op_code(&mut self) -> Option<OpCode> {
        let code = self.function.chunk.get_op_code(self.ip);
        self.ip += 1;
        return code;
    }

    fn get_next_byte(&mut self) -> Option<u8> {
        let code = self.function.chunk.get_byte(self.ip);
        self.ip += 1;
        return code;
    }

    fn get_next_short(&mut self) -> Option<u16> {
        let next_short = self.function.chunk.get_short(self.ip);
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
    natives: NativeFunctions,
}

impl<'a> VirtualMachine {
    /// Creates a new instance of the virtual machine.
    ///
    /// ## Returns
    /// * `VirtualMachine` – a new instance of the virtual machine.
    pub fn new() -> Self {
        Self {
            stack: Vec::with_capacity(256),
            frames: Vec::with_capacity(256),
            natives: Default::default(),
        }
    }

    /// Interprets a chuck of code.
    ///
    /// ## Returns
    /// * `InterpretResult` – The result of the source interpretation.
    pub(crate) fn interpret(&mut self, filepath: &String, source: &'a str) -> InterpretResult {
        // Parses the program into an AST and calculates the parser's execution time
        let parsing = exec_time(|| Parser::parse(source));

        let ast = match parsing.0 {
            Ok(x) => Rc::new(x),
            Err(e) => return e,
        };

        // Compiles the program into bytecode and calculates the compiler's execution time
        let compiling =
            exec_time(|| Compiler::compile_file(&filepath, &ast, self.natives.names.clone()));

        // Executes the program
        return match compiling.0 {
            Ok(main_func) => {
                self.stack.push(Object::Function(main_func.clone()));

                match self.call(main_func, 0) {
                    Ok(_) => {
                        #[cfg(feature = "bench_time")]
                        let start = Instant::now();

                        // Runs the program.
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
                    Err(_) => return InterpretResult::RuntimeError,
                }
            }
            Err(e) => e,
        };
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

    fn peek_stack(&self, pos: usize) -> &Object {
        &self.stack[pos]
    }

    fn peek_stack_mut(&mut self, pos: usize) -> &mut Object {
        &mut self.stack[pos]
    }

    fn read_constant(&self, idx: usize) -> &Object {
        return self.current_frame().get_constant(idx);
    }

    fn call_value(&mut self, callee: Object, arg_count: u8) -> Result<(), ()> {
        return match callee {
            Object::Function(obj) => self.call(obj, arg_count),
            Object::NativeFunction(obj) => {
                let mut args: Vec<Object> = vec![];
                for _ in 0..arg_count {
                    let val = self.pop_stack();
                    args.push(val);
                }
                args.reverse();

                match self.natives.call_native(&obj.name, args) {
                    Ok(x) => {
                        // Pop the native function call off the stack
                        self.pop_stack();
                        // Place the result of the call on top of the stack
                        self.push_stack(x);
                        Ok(())
                    }
                    Err(e) => {
                        self.report_runtime_error(e.as_str());
                        Err(())
                    }
                }
            }
            _ => {
                self.report_runtime_error(&format!(
                    "Cannot call object of type '{}'.",
                    callee.type_name()
                ));
                Err(())
            }
        };
    }

    /// Throws a runtime error to the console
    pub fn report_runtime_error(&self, message: &'a str) {
        let frame = self.current_frame();
        let line = frame.function.chunk.get_line_info(frame.ip).unwrap();
        eprintln!(
            "\x1b[31;1mRuntimeError\x1b[0m at [{}:{}] – {}",
            line.0, line.1, message
        );

        // Print stack trace
        for frame in self.frames.iter().rev() {
            let func = &frame.function;
            let line = frame.function.chunk.get_line_info(frame.ip).unwrap();

            if func.name != "" {
                eprintln!(
                    "Stack trace: [{}:{}] at '{}(...)'",
                    line.0, line.1, func.name
                );
            }
        }
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
    pub fn check_integer_operands(&self, left: &Object, right: &Object, opr: &str) -> bool {
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
