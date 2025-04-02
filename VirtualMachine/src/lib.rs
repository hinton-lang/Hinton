use std::ops::ControlFlow;
use std::path::PathBuf;

use core::errors::{report_errors_list, ErrMsg, ErrorReport, RuntimeErrMsg};
use core::tokens::TokenList;
use core::utils::get_time_millis;
use core::FRAMES_MAX;
use lexer::Lexer;
use objects::func_obj::FuncObj;
use objects::gc::{GarbageCollector, GcId, GcObject};
use objects::native_func_obj::NATIVES;
use objects::Object;

mod run;

/// The types of results the interpreter can return.
pub enum InterpretError {
  CompileError,
  RuntimeError,
}

/// The result of executing an instruction, which
/// can control the flow of the execution loop.
pub type OpRes = ControlFlow<RuntimeResult, ()>;

/// The result of executing a program, which can either
/// be okay, or a runtime error.
pub type RuntimeResult = Result<(), RuntimeErrMsg>;

/// A single frame in the call stack.
#[derive(Default, Copy, Clone)]
pub struct CallFrame {
  pub ip: usize,
  pub func_ptr: GcId,
  pub return_idx: usize,
}

/// The virtual machine.
pub struct VM {
  pub stack: Vec<Object>,
  constants: Vec<Object>,
  gc: GarbageCollector,
  globals: Vec<Object>,
  frames: Vec<CallFrame>,
  max_frames: usize,
}

impl VM {
  /// Generates a new virtual machine given the required parameters.
  ///
  /// # Arguments
  ///
  /// * `collector`: The garbage collector for this VM.
  /// * `pool`: The allocated constants for this VM.
  /// * `frames`: The max number of call stack frames for this VM.
  pub fn new(collector: GarbageCollector, pool: Vec<Object>, main_fn: GcId, frames: usize) -> Self {
    VM {
      stack: vec![],
      constants: pool,
      gc: collector,
      frames: vec![CallFrame { ip: 0, func_ptr: main_fn, return_idx: 0 }],
      globals: vec![],
      max_frames: frames,
    }
  }

  /// Executes the given source file and exits the program with the appropriate code.
  ///
  /// # Arguments
  ///
  /// * `filepath`: A `PathBuf` for the main source file.
  /// * `source`: A vector of characters from the source file.
  /// * `frames`: The user-specified max number of call stack frames.
  pub fn execute_file(filepath: PathBuf, source: Vec<char>, frames: Option<usize>) {
    // Interprets the source contents in the VM, and exit the interpreter with the appropriate code
    match VM::interpret(filepath, source, frames) {
      Ok(_) => {}
      Err(p) => match p {
        InterpretError::CompileError => std::process::exit(65),
        InterpretError::RuntimeError => std::process::exit(70),
      },
    }
  }

  /// Run the entire interpreter's pipeline on the given source file.
  ///
  /// # Arguments
  ///
  /// * `filepath`: A `PathBuf` for the main source file.
  /// * `source`: A vector of characters from the source file.
  /// * `frames`: The user-specified max number of call stack frames.
  pub fn interpret(filepath: PathBuf, source: Vec<char>, frames: Option<usize>) -> Result<(), InterpretError> {
    let frames = frames.unwrap_or(FRAMES_MAX); // Max stack call frames.

    #[cfg(feature = "PLV")]
    let lexer_start = get_time_millis();

    let lexer = Lexer::lex(&source);
    let tokens_list = TokenList::new(&filepath, &source, &lexer);

    #[cfg(feature = "PLV")]
    let lexer_end = get_time_millis();

    #[cfg(not(feature = "PLV"))]
    let compiler_result = compiler::Compiler::compile(&tokens_list);

    #[cfg(feature = "PLV")]
    let compiler_result = plv::export(&tokens_list, lexer_end - lexer_start);

    // Report any errors generated from the compilation pipeline.
    let (gc, consts, main_fn) = compiler_result.map_err(|err| {
      report_errors_list(&tokens_list, err, true);
      InterpretError::CompileError
    })?;

    // Create a new virtual machine and execute the code.
    let mut vm = VM::new(gc, consts, main_fn, frames);
    vm.run().map_err(|e| vm.report_runtime_error(e, &tokens_list))
  }

  /// Get an immutable reference to the current (last) frame in the call stack.
  fn current_frame(&self) -> &CallFrame {
    unsafe { self.frames.last().unwrap_unchecked() }
  }

  /// Get a mutable reference to the current (last) frame in the call stack.
  fn current_frame_mut(&mut self) -> &mut CallFrame {
    unsafe { self.frames.last_mut().unwrap_unchecked() }
  }

  /// The current function object being executed.
  fn current_fn(&self) -> &FuncObj {
    unsafe { self.gc.get(&self.current_frame().func_ptr).as_func_obj().unwrap_unchecked() }
  }

  /// Increments the instruction pointer of the current stack frame by the given offset.
  fn increment_instr_prt(&mut self, offset: usize) -> usize {
    let func = self.current_frame_mut();
    func.ip += offset;
    func.ip - offset
  }

  /// Gets the next raw byte in the chunk.
  fn next_byte(&mut self) -> u8 {
    let prev_pos = self.increment_instr_prt(1);
    self.current_fn().chunk.instructions[prev_pos]
  }

  /// Gets the next raw two bytes in the chunk.
  fn next_short(&mut self) -> u16 {
    let prev_pos = self.increment_instr_prt(2);
    self.current_fn().chunk.get_short(prev_pos)
  }

  /// Gets an immutable reference to the given stack position,
  /// with the offset starting at the last element.
  pub fn peek_stack(&self, offset: usize) -> &Object {
    &self.stack[self.stack.len() - offset - 1]
  }

  /// Pops the last object in the objects stack.
  fn pop_stack(&mut self) -> Object {
    match self.stack.pop() {
      Some(val) => val,
      None => panic!("Stack is empty!"),
    }
  }

  /// Tries to call the object at the `args_count` stack offset.
  fn call_obj(&mut self, args_count: usize) -> OpRes {
    let fn_stack_pos = self.stack.len() - args_count - 1;
    let callee = &self.stack[fn_stack_pos];

    let func_ptr = match callee {
      Object::Func(f) => f,
      Object::NativeFunc(n) => return self.call_native(n.0, fn_stack_pos),
      _ => {
        let err_msg = format!("Cannot call object of type '{}'.", callee.type_name());
        return ControlFlow::Break(Err(RuntimeErrMsg::Type(err_msg)));
      }
    };

    let (min, max) = match &self.gc.get(func_ptr).obj {
      GcObject::Func(f) => (f.min_arity, f.max_arity),
      _ => unreachable!("Can only check arity of function-like objects."),
    };

    // Perform arity check on the function or method call
    match self.arity_check(args_count, min, max) {
      Ok(_) => self.call_function(*func_ptr, fn_stack_pos),
      Err(e) => ControlFlow::Break(Err(e)),
    }
  }

  /// Generates a new stack frame for the given function.
  fn call_function(&mut self, func_ptr: GcId, return_idx: usize) -> OpRes {
    if self.frames.len() > self.max_frames {
      return ControlFlow::Break(Err(RuntimeErrMsg::Recursion("Stack overflow.".into())));
    }

    self.frames.push(CallFrame { func_ptr, return_idx, ip: 0 });
    ControlFlow::Continue(())
  }

  fn arity_check(&self, args_count: usize, min: u16, max: Option<u16>) -> Result<(), RuntimeErrMsg> {
    let arg_max = max.unwrap_or(u16::MAX);

    if args_count < min as usize || args_count > arg_max as usize {
      let msg = if min == arg_max {
        // all required params
        format!("Expected {} args, but got {} instead.", min, args_count)
      } else if max.is_none() {
        // 0+ required params, 0+ optional, 1 rest.
        format!("Expected at least {} args, but got {} instead.", min, args_count)
      } else {
        // 0+ required args and 1+ optional (not rest)
        format!("Expected {} to {} args, but got {} instead.", min, arg_max, args_count)
      };

      return Err(RuntimeErrMsg::Argument(msg));
    }

    Ok(())
  }

  /// Calls a native function.
  fn call_native(&mut self, idx: usize, return_idx: usize) -> OpRes {
    let native = &NATIVES[idx];
    let stack_len = self.stack.len();

    // Do arity check on the native function's call
    match self.arity_check(stack_len - (return_idx + 1), native.min_arity, native.max_arity) {
      Err(e) => ControlFlow::Break(Err(e)),
      Ok(_) => {
        // Take the top n elements of the stack as arguments to the function
        let args = &mut self.stack[(return_idx + 1)..stack_len];

        // Call the function
        match (native.body)(&mut self.gc, args) {
          Ok(o) => self.stack[return_idx] = o,
          Err(e) => return ControlFlow::Break(Err(e)),
        }

        // Pop the top n elements of the stack
        self.stack.drain((return_idx + 1)..self.stack.len());
        ControlFlow::Continue(())
      }
    }
  }

  /// Reports a runtime error and stack trace to the console.
  ///
  /// # Arguments
  ///
  /// * `error`: The runtime error message.
  /// * `token_list`: The tokens list.
  ///
  /// # Returns:
  /// ```InterpretError```
  fn report_runtime_error(&self, error: RuntimeErrMsg, token_list: &TokenList) -> InterpretError {
    let frame = self.current_frame();
    let token = self.current_fn().chunk.get_tok(frame.ip - 1);

    // Report the main error
    let err_msg = ErrMsg::Runtime(error);
    let main_err = ErrorReport { token, err_msg, hint: None };
    report_errors_list(token_list, vec![main_err], false);

    // Print stack trace
    println!("Traceback (most recent call last):");

    let mut prev_err = String::new();
    let mut repeated_line_count = 0;
    let frames_list = self.frames.iter();
    let frames_list_len = frames_list.len();

    for (i, frame) in frames_list.enumerate() {
      let func = match &self.gc.get(&frame.func_ptr).obj {
        GcObject::Func(f) => f,
        _ => unreachable!("Field 'func_ptr' should point to a function-like object."),
      };

      let tok_idx = func.chunk.get_tok(frame.ip);
      let tok = token_list[tok_idx];
      let fn_name = self.gc.get(&func.name).as_str_obj().unwrap().0.to_owned();

      let new_err = if fn_name.starts_with("<MainFunc") {
        format!("{:2}at origin {}", "", fn_name)
      } else {
        let loc = (tok.loc.line_num, tok.loc.col_start());
        format!("{:2}at [{}:{}] in '{}()'", "", loc.0, loc.1, fn_name,)
      };

      if prev_err == new_err {
        repeated_line_count += 1;

        if repeated_line_count < 3 {
          eprintln!("{}", new_err);
        } else {
          if i == frames_list_len - 1 {
            let c = repeated_line_count - 2;
            eprintln!("{:5}\x1b[1mPrevious line repeated {} more times.\x1b[0m", "", c);
          }
          continue;
        }
      } else {
        if repeated_line_count > 0 {
          let c = repeated_line_count - 2;
          eprintln!("{:5}\x1b[1mPrevious line repeated {} more times.\x1b[0m", "", c);
          repeated_line_count = 0;
        }
        eprintln!("{}", new_err);
        prev_err = new_err;
      }
    }

    eprintln!("\n\x1b[31;1mERROR:\x1b[0m Aborted execution due to 1 previous error.");
    InterpretError::RuntimeError
  }
}
