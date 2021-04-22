#![allow(dead_code)]

// Using other modules
use std::fs;

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
