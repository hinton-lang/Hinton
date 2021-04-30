#![allow(dead_code)]

#[cfg(feature = "bench_time")]
use std::time::Instant;

// Using other modules
use std::{fs, time::Duration};

// Declaring crate-level Modules
mod analyzer;
mod chunk;
mod intermediate;
mod lexer;
mod objects;
mod virtual_machine;

// Using create-level sub-modules
use virtual_machine::InterpretResult;
use virtual_machine::VirtualMachine;

/// The main function
fn main() {
    let filename = "./test.ht";
    run_file(filename);
}

fn run_file(filename: &str) {
    let contents = fs::read_to_string(filename).expect("Something went wrong reading the file.");

    // Creates a virtual machine with the given source contents
    let mut vm = VirtualMachine::new();
    // Interprets the source contents in the VM
    let result = vm.interpret(&contents);

    // Exit the interpreter with the appropriate code
    match result {
        InterpretResult::INTERPRET_PARSE_ERROR => std::process::exit(65),
        InterpretResult::INTERPRET_COMPILE_ERROR => std::process::exit(65),
        InterpretResult::INTERPRET_RUNTIME_ERROR => std::process::exit(70),
        InterpretResult::INTERPRET_OK => std::process::exit(0),
    }
}

/// Executes the provided function while also calculating it's execution time.
///
/// ## Arguments
/// `executor` – The function (with no parameters) to be executed.
///
/// ## Returns
/// `(T, Duration)` – A tuple with the result of the executed function as its
/// first parameter, and the execution time of the function as its second parameter.
pub fn exec_time<T, F: Fn() -> T>(executor: F) -> (T, Duration) {
    #[cfg(feature = "bench_time")]
    {
        let start = Instant::now();
        let exec = executor();
        let time = start.elapsed();

        return (exec, time);
    }

    #[cfg(not(feature = "bench_time"))]
    (executor(), Duration::new(0, 0))
}
