use std::error::Error;
use std::io::ErrorKind;
use std::path::PathBuf;
use std::{env, fs, io};

fn main() {
  // structure: hinton <flags?> <filename> <program args?>
  let args = env::args().collect::<Vec<String>>();

  match args.as_slice() {
    [_] => todo!("REPL"),
    // TODO: the user can specify a stack frame size.
    [_, file] => run_file(file, None),
    _ => todo!("Program and Script Flags"),
  }
}

fn run_file(filename: &str, frames: Option<usize>) {
  let source = read_file(filename);
  virtual_machine::VM::execute_file(source.0, source.1, frames)
}

fn read_file(filename: &str) -> (PathBuf, Vec<char>) {
  match read_file_chars(filename) {
    Ok(src) => src,
    Err(error) => match error.downcast_ref::<io::Error>() {
      Some(e) => {
        match e.kind() {
          ErrorKind::NotFound => eprintln!("File '{}' not found.", filename),
          ErrorKind::PermissionDenied => eprintln!("Need permission to open '{}'.", filename),
          ErrorKind::UnexpectedEof => eprintln!("Unexpected end-of-file '{}'.", filename),
          _ => eprintln!("Unexpected error when opening file '{}'.", filename),
        };

        match e.raw_os_error() {
          Some(code) => std::process::exit(code),
          None => std::process::exit(70),
        }
      }
      None => {
        eprintln!("Unexpected error when opening file '{}'.", filename);
        std::process::exit(70);
      }
    },
  }
}

fn read_file_chars(filename: &str) -> Result<(PathBuf, Vec<char>), Box<dyn Error>> {
  let path = fs::canonicalize(filename)?;
  let contents = fs::read_to_string(filename)?;
  Ok((path, contents.chars().collect()))
}
