use crate::virtual_machine::call_frame::CallFrameType;
use crate::virtual_machine::VM;
use core::errors::{print_error_snippet, RuntimeErrorType};

/// Throws a runtime error to the console.
///
/// # Parameters
/// - `vm`: A reference to the virtual machine.
/// - `error`: The generated error.
/// - `message`: The error message to be displayed.
/// - `source`: The program's source text.
pub fn report_runtime_error(vm: &VM, error: RuntimeErrorType, message: String, source: &[char]) {
  let source: String = source.iter().collect();
  let source_lines: Vec<&str> = source.split('\n').collect();

  let frame = vm.current_frame();
  let f = match &frame.callee {
    CallFrameType::Closure(c) => c.function.borrow(),
    CallFrameType::Function(f) => f.borrow(),
    CallFrameType::Method(m) => m.method.function.borrow(),
  };
  let line = f.chunk.get_line_info(frame.ip - 1);

  let error_name = match error {
    RuntimeErrorType::ArgumentError => "ArgumentError",
    RuntimeErrorType::AssertionError => "AssertionError",
    RuntimeErrorType::IndexError => "IndexError",
    RuntimeErrorType::InstanceError => "InstanceError",
    RuntimeErrorType::Internal => "InternalError",
    RuntimeErrorType::KeyError => "KeyError",
    RuntimeErrorType::RecursionError => "RecursionError",
    RuntimeErrorType::ReferenceError => "ReferenceError",
    RuntimeErrorType::StopIteration => "EndOfIterationError",
    RuntimeErrorType::TypeError => "TypeError",
    RuntimeErrorType::ZeroDivision => "ZeroDivisionError",
  };

  eprintln!("\x1b[31;1m{}:\x1b[0m\x1b[1m {}\x1b[0m", error_name, message);

  let src_line = source_lines.get(line.0 - 1).unwrap();
  print_error_snippet(line.0, line.1, (0, 1), src_line);

  // Print stack trace
  println!("Traceback (most recent call last):");
  let mut prev_err = String::new();
  let mut repeated_line_count = 0;
  let frames_list = vm.frames_stack().iter();
  let frames_list_len = frames_list.len();

  for (i, frame) in frames_list.enumerate() {
    let func = match &frame.callee {
      CallFrameType::Closure(c) => c.function.borrow(),
      CallFrameType::Function(f) => f.borrow(),
      CallFrameType::Method(m) => m.method.function.borrow(),
    };
    let line = func.chunk.get_line_info(frame.ip);

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
        if i == frames_list_len - 1 {
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

  eprintln!("\n\x1b[31;1mERROR:\x1b[0m Aborted execution due to previous errors.");
}
