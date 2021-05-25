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

pub struct ErrorReport {
    pub column: usize,
    pub lexeme_len: usize,
    pub line: usize,
    pub message: String,
}

pub struct ErrorList(pub Vec<ErrorReport>);

/// Represents a virtual machine
pub struct VirtualMachine {
    filepath: String,
    frames: Vec<CallFrame>,
    natives: NativeFunctions,
    stack: Vec<objects::Object>,
}

/// Represents the types of errors that can occur during
/// execution of the compiled bytecode.
pub enum RuntimeErrorType {
    IndexError,
    StopIteration,
    Internal,
    TypeError,
    ZeroDivision,
    ArgumentError,
    RecursionError,
    ReferenceError,
}

pub enum RuntimeResult {
    Error {
        error: RuntimeErrorType,
        message: String,
    },
    Ok,
}

impl VirtualMachine {
    /// Interprets a chuck of code.
    ///
    /// ## Returns
    /// * `InterpretResult` – The result of the source interpretation.
    pub(crate) fn interpret(filepath: &str, source: &str) -> InterpretResult {
        // Creates a new virtual machine
        let mut _self = VirtualMachine {
            stack: Vec::with_capacity(256),
            frames: Vec::with_capacity(256),
            natives: Default::default(),
            filepath: String::from(filepath),
        };

        // Parses the program into an AST and calculates the parser's execution time
        let parsing = exec_time(|| Parser::parse(source));

        // Aborts if there are any parsing errors
        let ast = match parsing.0 {
            Ok(x) => Rc::new(x),
            Err(e) => {
                _self.report_errors_list(e, source);
                return InterpretResult::ParseError;
            }
        };

        // Compiles the program into bytecode and calculates the compiler's execution time
        let compiling =
            exec_time(|| Compiler::compile_file(filepath, _self.natives.names.clone(), &ast));
        let module = match compiling.0 {
            Ok(x) => x,
            Err(e) => {
                _self.report_errors_list(e, source);
                return InterpretResult::CompileError;
            }
        };

        // Executes the program
        _self.stack.push(Object::Function(module.clone()));
        return match _self.call(module, 0) {
            RuntimeResult::Ok => {
                #[cfg(feature = "bench_time")]
                let start = Instant::now();

                // Runs the program.
                let runtime_result = _self.run();

                #[cfg(feature = "bench_time")]
                {
                    let run_time = start.elapsed();

                    println!("\n======= ⚠️  Execution Results ⚠️  =======");
                    println!("Parse Time:\t{:?}", parsing.1);
                    println!("Compile Time:\t{:?}", compiling.1);
                    println!("Run Time:\t{:?}", run_time);
                    println!("=======================================");
                }

                match runtime_result {
                    RuntimeResult::Ok => InterpretResult::Ok,
                    RuntimeResult::Error { error, message } => {
                        _self.report_runtime_error(error, message, source);
                        InterpretResult::RuntimeError
                    }
                }
            }
            RuntimeResult::Error { error, message } => {
                _self.report_runtime_error(error, message, source);
                InterpretResult::RuntimeError
            }
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

    fn call_value(&mut self, callee: Object, arg_count: u8) -> RuntimeResult {
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
                        RuntimeResult::Ok
                    }
                    Err(e) => e,
                }
            }
            _ => RuntimeResult::Error {
                error: RuntimeErrorType::TypeError,
                message: format!("Cannot call object of type '{}'.", callee.type_name()),
            },
        };
    }

    /// Throws a runtime error to the console
    pub fn report_runtime_error(&self, error: RuntimeErrorType, message: String, source: &str) {
        let source_lines: Vec<&str> = source.split("\n").collect();

        let frame = self.current_frame();
        let line = frame.function.chunk.get_line_info(frame.ip - 1).unwrap();

        let error_name = match error {
            RuntimeErrorType::IndexError => "IndexError",
            RuntimeErrorType::StopIteration => "EndOfIterationError",
            RuntimeErrorType::Internal => "InternalError",
            RuntimeErrorType::TypeError => "TypeError",
            RuntimeErrorType::ZeroDivision => "ZeroDivisionError",
            RuntimeErrorType::ArgumentError => "ArgumentError",
            RuntimeErrorType::RecursionError => "RecursionError",
            RuntimeErrorType::ReferenceError => "ReferenceError",
        };

        eprintln!("\x1b[31;1m{}:\x1b[0m\x1b[1m {}\x1b[0m", error_name, message);

        let src_line = source_lines.get(line.0 - 1).unwrap();
        self.print_error_snippet(line.0, line.1, 1, src_line);

        // Print stack trace
        println!("Traceback (most recent call last):");
        let mut prev_err = String::new();
        let mut repeated_line_count = 0;
        for (i, frame) in self.frames.iter().enumerate() {
            let func = &frame.function;
            let line = frame.function.chunk.get_line_info(frame.ip).unwrap();

            let new_err;
            if func.name.starts_with('<') {
                new_err = format!("{:4}at [{}:{}] in {}", "", line.0, line.1, func.name);
            } else {
                new_err = format!("{:4}at [{}:{}] in '{}()'", "", line.0, line.1, func.name);
            }

            if prev_err == new_err {
                repeated_line_count += 1;

                if repeated_line_count < 3 {
                    eprintln!("{}", new_err);
                } else {
                    if i == self.frames.len() - 1 {
                        eprintln!(
                            "{:7}\x1b[1mPrevious line repeated {} more times.\x1b[0m",
                            "",
                            repeated_line_count - 2
                        );
                    }

                    continue;
                }
            } else {
                if repeated_line_count > 0 {
                    eprintln!(
                        "{:7}\x1b[1mPrevious line repeated {} more times.\x1b[0m",
                        "",
                        repeated_line_count - 2
                    );
                    repeated_line_count = 0;
                }
                eprintln!("{}", new_err);
                prev_err = new_err;
            }
        }

        eprintln!("\nAborted execution due to previous errors.");
    }

    /// Reports an error list coming from the parser or compiler.
    ///
    /// ## Arguments
    /// * `errors` – An `ErrorList` containing the errors.
    /// * `source` – A reference to the source contents.
    fn report_errors_list(&self, errors: ErrorList, source: &str) {
        let source_lines: Vec<&str> = source.split("\n").collect();

        for error in errors.0.iter() {
            eprintln!("{}", error.message);
            self.print_error_source(error.line, error.column, error.lexeme_len, &source_lines);
        }

        eprintln!("Aborted execution due to previous errors.");
    }

    /// Prints the filepath and a snippet of the source line associated with a parser or compiler error.
    ///
    /// ## Arguments
    /// * `line_num` – The source line number of the error.
    /// * `col` – The source column number of the error.
    /// * `len` – The length of the token that produced the error.
    /// * `lines` – A reference to a vector with the source lines.
    fn print_error_source(&self, line_num: usize, col: usize, len: usize, lines: &Vec<&str>) {
        let front_pad = (f64::log10(line_num as f64).floor() + 1f64) as usize;
        let line = lines.get(line_num - 1).unwrap();

        eprintln!(" {}---> File '{}'.", "-".repeat(front_pad), self.filepath);
        self.print_error_snippet(line_num, col, len, line);
    }

    /// Prints a snippet of the source line associated with an error.
    ///
    /// ## Arguments
    /// * `line_num` – The source line number of the error.
    /// * `col` – The source column number of the error.
    /// * `len` – The length of the token that produced the error.
    /// * `src` – A reference to a the source error line.
    fn print_error_snippet(&self, line_num: usize, col: usize, len: usize, src: &str) {
        let front_pad = (f64::log10(line_num as f64).floor() + 1f64) as usize;
        // +2 for one extra space at the front and one at the back
        let whitespace_pad_size = " ".repeat(front_pad + 2);

        // Compute the line colum of the error with
        // timed whitespaces from the source line.
        let mut removed_whitespace = 0;
        for c in src.chars() {
            if c == ' ' {
                removed_whitespace += 1;
            } else {
                break;
            }
        }
        let col = col - removed_whitespace;

        eprintln!("{}|", whitespace_pad_size);
        eprint!(" {} | ", line_num);
        eprintln!("{}", src.trim());
        eprint!("{}|", whitespace_pad_size);
        eprintln!(" {}\x1b[31;1m{}\x1b[0m\n", " ".repeat(col), "^".repeat(len));
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
    pub fn check_integer_operands(
        &self,
        left: &Object,
        right: &Object,
        opr: &str,
    ) -> Result<(), String> {
        return if !left.is_int() || !right.is_int() {
            Err(format!(
                "Operation '{}' not defined for operands of type '{}' and '{}'.",
                opr.to_string(),
                left.type_name(),
                right.type_name()
            ))
        } else {
            Ok(())
        };
    }
}
