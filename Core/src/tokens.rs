use std::ops::Index;
use std::path::PathBuf;

/// Represents the index of a Token in the TokenList.
pub type TokenIdx = usize;

/// List of Tokens found in the source code.
pub struct TokenList<'a> {
  pub tokens: &'a [Token],
  pub src: &'a [char],
  pub filepath: &'a PathBuf,
}

impl<'a> Index<usize> for TokenList<'a> {
  type Output = Token;
  fn index(&self, index: usize) -> &Self::Output {
    &self.tokens[index]
  }
}

impl<'a> TokenList<'a> {
  /// Generates a new Tokens List.
  ///
  /// # Arguments
  ///
  /// * `src`: A reference to the source list of characters.
  /// * `tokens`: A reference to the source list of lexed tokens.
  ///
  /// # Returns:
  /// ```TokenList```
  pub fn new(filepath: &'a PathBuf, src: &'a [char], tokens: &'a [Token]) -> Self {
    Self { src, tokens, filepath }
  }

  /// Gets the lexeme of a token based on it's location information.
  ///
  /// # Arguments
  ///
  /// * `idx`: The index of the token in the list of tokens.
  ///
  /// # Returns:
  /// ```String```
  pub fn lexeme(&self, idx: TokenIdx) -> String {
    let tok = &self[idx];

    match &tok.kind {
      TokenKind::ERROR(e) => e.to_str().to_string(),
      TokenKind::EOF => "\0".to_string(),
      _ => self.src[tok.span.0..tok.span.1].iter().collect(),
    }
  }

  /// Gets the source-code location information of a token.
  ///
  /// # Arguments
  ///
  /// * `idx`: The index of the token in the list of tokens.
  ///
  /// # Returns:
  /// ```TokenLoc```
  pub fn location(&self, idx: TokenIdx) -> TokenLoc {
    self[idx].get_location()
  }
}

/// The source-code location information for a Token.
pub struct TokenLoc {
  pub line_num: usize,
  pub col_start: usize,
  pub col_end: usize,
  pub span: (usize, usize),
  pub line_start: usize,
}

// A token that represents a single unit of Hinton code.
#[derive(Clone)]
pub struct Token {
  /// The token's line number
  pub line_num: usize,
  /// The beginning of this token's line in the source.
  pub line_start: usize,
  /// The token's lexeme span (column start, column end)
  pub span: (usize, usize),
  /// The token's type
  pub kind: TokenKind,
}

impl Token {
  /// Gets the source-code location information of a token.
  pub fn get_location(&self) -> TokenLoc {
    TokenLoc {
      line_num: self.line_num,
      col_start: self.span.0 - self.line_start,
      col_end: self.span.0 - self.line_start,
      span: self.span,
      line_start: self.line_start,
    }
  }
}

/// The types of tokens in a Hinton program.
#[allow(non_camel_case_types, clippy::upper_case_acronyms)]
#[derive(Debug, Clone)]
#[repr(u8)]
pub enum TokenKind {
  // Symbol-based tokens
  AMPERSAND,        // &
  AT,               // @
  AT_EQ,            // @=
  BANG,             // !
  BIT_AND_EQ,       // &=
  BIT_L_SHIFT,      // <<
  BIT_L_SHIFT_EQ,   // <<=
  BIT_NOT,          // ~
  BIT_OR_EQ,        // |=
  BIT_R_SHIFT,      // >>
  BIT_R_SHIFT_EQ,   // >>=
  BIT_XOR,          // ^
  BIT_XOR_EQ,       // ^=
  COLON,            // :
  COLON_EQUALS,     // :=
  COMMA,            // ,
  DASH,             // -
  DOT,              // .
  DOUBLE_AMPERSAND, // &&
  DOUBLE_DOT,       // ..
  DOUBLE_VERT_BAR,  // ||
  EQUALS,           // =
  GREATER_THAN,     // >
  GREATER_THAN_EQ,  // >=
  HASHTAG,          // #
  LESS_THAN,        // <
  LESS_THAN_EQ,     // <=
  LOGIC_AND_EQ,     // &&=
  LOGIC_EQ,         // ==
  LOGIC_NOT_EQ,     // !=
  LOGIC_OR_EQ,      // ||=
  L_BRACKET,        // [
  L_CURLY,          // {
  L_PAREN,          // (
  MINUS_EQ,         // -=
  MOD_EQ,           // %=
  NONISH,           // ??
  NONISH_EQ,        // ??=
  PERCENT,          // %
  PIPE,             // |>
  PLUS,             // +
  PLUS_EQ,          // +=
  POW,              // **
  POW_EQUALS,       // **=
  QUESTION,         // ?
  RANGE_EQ,         // ..= (NOTE: This is not an assignment operator)
  R_BRACKET,        // ]
  R_CURLY,          // }
  R_PAREN,          // )
  SAFE_ACCESS,      // ?.
  SEMICOLON,        // ;
  SLASH,            // /
  SLASH_EQ,         // /=
  STAR,             // *
  STAR_EQ,          // *=
  THICK_ARROW,      // =>
  THIN_ARROW,       // ->
  TRIPLE_DOT,       // ...
  VERT_BAR,         // |

  // Special Token for Str interpolation
  START_INTERPOL_STR,
  END_INTERPOL_STR,
  START_INTERPOL_EXPR,
  END_INTERPOL_EXPR,

  // Value Literals
  BINARY_LIT,
  FALSE_LIT,
  FLOAT_LIT,
  HEX_LIT,
  IDENTIFIER,
  INT_LIT,
  NONE_LIT,
  OCTAL_LIT,
  SCIENTIFIC_LIT,
  STR_LIT,
  TRUE_LIT,

  // Keywords
  ABSTRACT_KW,
  AND_KW,
  ASYNC_KW,
  AS_KW,
  AWAIT_KW,
  BREAK_KW,
  CATCH_KW,
  CLASS_KW,
  CONST_KW,
  CONTINUE_KW,
  DEFAULT_KW,
  DEL_KW,
  ELSE_KW,
  ENUM_KW,
  EXPORT_KW,
  FINALLY_KW,
  FOR_KW,
  FROM_KW,
  FUNC_KW,
  IF_KW,
  IMPL_KW,
  IMPORT_KW,
  INIT_KW,
  INSTOF_KW,
  IN_KW,
  LET_KW,
  LOOP_KW,
  MATCH_KW,
  MOD_KW,
  NEW_KW,
  OR_KW,
  OVERRIDE_KW,
  PUB_KW,
  RETURN_KW,
  SELF_KW,
  STATIC_KW,
  SUPER_KW,
  THROW_KW,
  TRY_KW,
  TYPEOF_KW,
  WHILE_KW,
  WITH_KW,
  YIELD_KW,

  /// Other Tokens
  EOF,
  ERROR(ErrorTokenKind),
  // **** For Future Implementation
  // INTERFACE_KW
  // ANY_TYPE,
  // ARRAY_TYPE,
  // BOOL_TYPE,
  // DICT_TYPE,
  // FLOAT_TYPE,
  // FUNC_TYPE,
  // INT_TYPE,
  // NULL_TYPE,
  // STR_TYPE,
  // VOID_TYPE,
}

impl TokenKind {
  /// Checks that this token is of a given type.
  ///
  /// # Parameters
  /// - `token_type`: The token type to be matched against this token.
  pub fn type_match(&self, token_type: &TokenKind) -> bool {
    std::mem::discriminant(self) == std::mem::discriminant(token_type)
  }
}

/// Maps a keyword string to a token type.
///
/// # Parameters
/// - `id`: The identifier's string name.
///
/// # Returns
/// `TokenType`: The type of token matched for given identifier name.
pub fn make_identifier_kind(id: &str) -> TokenKind {
  match id {
    "abstract" => TokenKind::ABSTRACT_KW,
    "and" => TokenKind::AND_KW,
    "as" => TokenKind::AS_KW,
    "async" => TokenKind::ASYNC_KW,
    "await" => TokenKind::AWAIT_KW,
    "break" => TokenKind::BREAK_KW,
    "catch" => TokenKind::CATCH_KW,
    "class" => TokenKind::CLASS_KW,
    "const" => TokenKind::CONST_KW,
    "continue" => TokenKind::CONTINUE_KW,
    "default" => TokenKind::DEFAULT_KW,
    "del" => TokenKind::DEL_KW,
    "else" => TokenKind::ELSE_KW,
    "enum" => TokenKind::ENUM_KW,
    "export" => TokenKind::EXPORT_KW,
    "false" => TokenKind::FALSE_LIT,
    "finally" => TokenKind::FINALLY_KW,
    "for" => TokenKind::FOR_KW,
    "from" => TokenKind::FROM_KW,
    "func" => TokenKind::FUNC_KW,
    "if" => TokenKind::IF_KW,
    "impl" => TokenKind::IMPL_KW,
    "import" => TokenKind::IMPORT_KW,
    "in" => TokenKind::IN_KW,
    "init" => TokenKind::INIT_KW,
    "instof" => TokenKind::INSTOF_KW,
    "let" => TokenKind::LET_KW,
    "loop" => TokenKind::LOOP_KW,
    "match" => TokenKind::MATCH_KW,
    "mod" => TokenKind::MOD_KW,
    "new" => TokenKind::NEW_KW,
    "none" => TokenKind::NONE_LIT,
    "or" => TokenKind::OR_KW,
    "override" => TokenKind::OVERRIDE_KW,
    "pub" => TokenKind::PUB_KW,
    "return" => TokenKind::RETURN_KW,
    "self" => TokenKind::SELF_KW,
    "static" => TokenKind::STATIC_KW,
    "super" => TokenKind::SUPER_KW,
    "throw" => TokenKind::THROW_KW,
    "true" => TokenKind::TRUE_LIT,
    "try" => TokenKind::TRY_KW,
    "typeof" => TokenKind::TYPEOF_KW,
    "while" => TokenKind::WHILE_KW,
    "with" => TokenKind::WITH_KW,
    "yield" => TokenKind::YIELD_KW,

    // **** For Future Implementation
    // "interface" => TokenType::INTERFACE_KEYWORD,
    // "Any"   => TokenType::ANY_TYPE,
    // "Array" => TokenType::ARRAY_TYPE,
    // "Bool"  => TokenType::BOOL_TYPE,
    // "Dict"  => TokenType::DICT_TYPE,
    // "Float" => TokenType::FLOAT_TYPE,
    // "Func"  => TokenType::FUNC_TYPE,
    // "Int"   => TokenType::INT_TYPE,
    // "None"  => TokenType::NULL_TYPE,
    // "Str"   => TokenType::STR_TYPE,
    // "Void"  => TokenType::VOID_TYPE,
    _ => TokenKind::IDENTIFIER,
  }
}

#[derive(Debug, Clone)]
pub enum ErrorTokenKind {
  /// Invalid Character.
  InvalidChar,
  /// Unterminated String.
  UnterminatedStr,
  /// Leading zeros not allowed in numeric literal.
  NoLeadZerosInNum,
  /// The exponent of a scientific literal must be an integer.
  ExpectedIntExpo,
  /// Unexpected '.' in hexadecimal literal.
  DotInHex,
  /// Unexpected '.' in octal literal.
  DotInOct,
  /// Unexpected '.' in binary literal.
  DotInBin,
  /// Unexpected extra '.' in float literal.
  ExtraDotInFloat,
  /// Too many underscores in numeric literal separator.
  ExtraSepInNum,
  /// Separator not allowed after floating point.
  SepAfterFloat,
  /// Separator not allowed before floating point.
  SepBeforeFloat,
  /// Unexpected character in octal literal.
  NonOctChar,
  /// Unexpected character in binary literal.
  NonBinChar,
  /// Unexpected extra 'e' in scientific literal.
  ExtraEInScientific,
  /// Separator not allowed at the end of numeric literal.
  SepAtEndOfNum,
}

impl ErrorTokenKind {
  /// Converts an error token to its string message representation.
  pub fn to_str(&self) -> &str {
    match self {
      ErrorTokenKind::InvalidChar => "Invalid Character.",
      ErrorTokenKind::UnterminatedStr => "Unterminated String",
      ErrorTokenKind::NoLeadZerosInNum => "Leading zeros not allowed in numeric literal",
      ErrorTokenKind::ExpectedIntExpo => "The exponent of a scientific literal must be an integer",
      ErrorTokenKind::DotInHex => "Unexpected '.' in hexadecimal literal",
      ErrorTokenKind::DotInOct => "Unexpected '.' in octal literal",
      ErrorTokenKind::DotInBin => "Unexpected '.' in binary literal.",
      ErrorTokenKind::ExtraDotInFloat => "Unexpected extra '.' in float literal.",
      ErrorTokenKind::ExtraSepInNum => "Too many underscores in numeric literal separator.",
      ErrorTokenKind::SepAfterFloat => "Separator not allowed after floating point.",
      ErrorTokenKind::SepBeforeFloat => "Separator not allowed before floating point.",
      ErrorTokenKind::NonOctChar => "Unexpected character in octal literal.",
      ErrorTokenKind::NonBinChar => "Unexpected character in binary literal.",
      ErrorTokenKind::ExtraEInScientific => "Unexpected extra 'e' in scientific literal.",
      ErrorTokenKind::SepAtEndOfNum => "Separator not allowed at the end of numeric literal.",
    }
  }
}
