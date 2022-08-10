// A token that represents a single unit of Hinton code.
#[derive(Clone)]
pub struct Token {
  /// The token's line number
  pub line_num: usize,
  /// The token's column start
  pub column_start: usize,
  /// The token's column end
  pub column_end: usize,
  /// The token's lexeme
  pub lexeme: String,
}
