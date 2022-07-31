pub struct TokenIdx(pub usize);

impl Into<usize> for TokenIdx {
  fn into(self) -> usize {
    self.0
  }
}

impl From<usize> for TokenIdx {
  fn from(x: usize) -> Self {
    TokenIdx(x)
  }
}

impl Default for TokenIdx {
  fn default() -> Self {
    usize::MAX.into()
  }
}

// A token that represents a single unit of Hinton code.
#[derive(Clone)]
pub struct Token {
  /// The token's line number
  pub line_num: usize,
  /// The token's column start
  pub column_start: usize,
  /// The token's column end
  pub column_end: usize,
  /// The token's type
  pub kind: TokenKind,
  /// The token's lexeme
  pub lexeme: String,
}

/// The types of tokens in a Hinton program.
#[allow(non_camel_case_types)]
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
  DOUBLE_DOT,       // ..
  EQUALS,           // =
  GREATER_THAN,     // >
  GREATER_THAN_EQ,  // >=
  LESS_THAN,        // <
  LESS_THAN_EQ,     // <=
  DOUBLE_AMPERSAND, // &&
  LOGIC_AND_EQ,     // &&=
  LOGIC_EQ,         // ==
  LOGIC_NOT_EQ,     // !=
  DOUBLE_VERT_BAR,  // ||
  LOGIC_OR_EQ,      // ||=
  L_BRACKET,        // [
  L_CURLY,          // {
  L_PAREN,          // (
  MINUS_EQ,         // -=
  PERCENT,          // %
  MOD_EQ,           // %=
  NONISH,           // ??
  NONISH_EQ,        // ??=
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
  FINALLY_KW,
  FOR_KW,
  FROM_KW,
  FUNC_KW,
  IF_KW,
  IMPORT_KW,
  INSTOF_KW,
  IN_KW,
  LET_KW,
  LOOP_KW,
  MATCH_KW,
  MOD_KW,
  NEW_KW,
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
  OR_KW,
  AND_KW,

  /// Other Tokens
  EOF,
  ERROR,
  // **** For Future Implementation
  // IMPL_KW,
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
    "finally" => TokenKind::FINALLY_KW,
    "for" => TokenKind::FOR_KW,
    "from" => TokenKind::FROM_KW,
    "func" => TokenKind::FUNC_KW,
    "if" => TokenKind::IF_KW,
    "import" => TokenKind::IMPORT_KW,
    "in" => TokenKind::IN_KW,
    "instof" => TokenKind::INSTOF_KW,
    "let" => TokenKind::LET_KW,
    "loop" => TokenKind::LOOP_KW,
    "match" => TokenKind::MATCH_KW,
    "mod" => TokenKind::MOD_KW,
    "new" => TokenKind::NEW_KW,
    "or" => TokenKind::OR_KW,
    "override" => TokenKind::OVERRIDE_KW,
    "pub" => TokenKind::PUB_KW,
    "return" => TokenKind::RETURN_KW,
    "self" => TokenKind::SELF_KW,
    "static" => TokenKind::STATIC_KW,
    "super" => TokenKind::SUPER_KW,
    "throw" => TokenKind::THROW_KW,
    "try" => TokenKind::TRY_KW,
    "typeof" => TokenKind::TYPEOF_KW,
    "while" => TokenKind::WHILE_KW,
    "with" => TokenKind::WITH_KW,
    "yield" => TokenKind::YIELD_KW,

    // **** For Future Implementation
    // "interface" => TokenType::INTERFACE_KEYWORD,
    // "impl"      => TokenType::IMPL_KW,
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
