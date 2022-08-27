use std::num::{ParseFloatError, ParseIntError};
use std::time::{SystemTime, UNIX_EPOCH};

/// Get the current unix epoch time in milliseconds.
pub fn get_time_millis() -> u64 {
  let start = SystemTime::now();
  let time_since_epoch = start.duration_since(UNIX_EPOCH).expect("Time went backwards");
  time_since_epoch.as_secs() * 1000 + time_since_epoch.subsec_nanos() as u64 / 1_000_000
}

/// Parses an integer literal lexeme into a Rust int.
///
/// ```bnf
/// INTEGER_LITERAL ::= DIGIT_NOT_ZERO ("_" DIGIT+)*
/// ```
pub fn parse_int_lexeme(lexeme: String) -> Result<i64, ParseIntError> {
  // Removes any underscores and parses the lexeme into an int
  // that can then be converted to a Hinton Int Object
  lexeme.replace('_', "").parse::<i64>()
}

/// Parses a float literal lexeme into a Rust float.
///
/// ```bnf
/// FLOAT_LITERAL ::= (DIGIT+ "." DIGIT*) | (DIGIT* "." DIGIT+)
/// ```
pub fn parse_float_lexeme(lexeme: String) -> Result<f64, ParseFloatError> {
  // Removes any underscores and parses the lexeme into a float
  // that can then be converted to a Hinton Float Object
  lexeme.replace('_', "").parse::<f64>()
}

/// Parses a hex, octal, and binary literal lexeme into a Rust Int.
///
/// ```bnf
/// HEX_LITERAL      ::= ("0x" | "0X") HEX_DIGIT+ ("_" HEX_DIGIT+)*
/// HEX_DIGIT        ::= DIGIT
///                  | ("a" | "b" | "c" | "d" | "e" | "f")
///                  | ("A" | "B" | "C" | "D" | "E" | "F")
/// OCT_LITERAL      ::= ("0o" | "0O") OCT_DIGIT+ ("_" OCT_DIGIT+)*
/// OCT_DIGIT        ::= "0" | "1" | "2" | "3" | "4" | "5" | "6" | "7"
/// BINARY_LITERAL   ::= ("0b" | "0B") BINARY_DIGIT+ ("_" BINARY_DIGIT+)*
/// BINARY_DIGIT     ::= "0" | "1"
/// ```
pub fn parse_int_from_lexeme_base(lexeme: String, radix: u32) -> Result<i64, ParseIntError> {
  // Removes any underscores and parses the lexeme into an int
  // that can then be converted to a Hinton Int Object
  i64::from_str_radix(&lexeme.replace('_', "")[2..], radix)
}

/// Parses a scientific-notation literal into a Rust float.
///
/// ```bnf
/// SCIENTIFIC_LITERAL ::= (FLOAT_LITERAL | INTEGER_LITERAL) ("e" | "E") "-"? INTEGER_LITERAL
/// ```
pub fn parse_scientific_literal_lexeme(lexeme: String) -> Result<f64, ParseFloatError> {
  // Removes the underscores from the lexeme and Split into base and exponent
  let lexeme = lexeme.replace('_', "");
  let lexemes: Vec<&str> = lexeme.split(&['e', 'E']).collect();

  // Parses the base into a float
  let base = lexemes[0].parse::<f64>()?;

  // Parses the exponent into a float
  let exponent = lexemes[1].parse::<f64>()?;

  Ok(base * 10f64.powf(exponent))
}
