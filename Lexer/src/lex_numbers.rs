use core::tokens::ErrorTokenKind::*;
use core::tokens::Token;
use core::tokens::TokenKind::*;

use crate::{char_is_ident_start, Lexer};

/// The types of number we can expect to lex.
enum ExNumType {
  /// Expect hexadecimal number.
  Hex,
  /// Expect octal number.
  Oct,
  /// Expect binary number.
  Bin,
  /// Expect floating-point number with no leading digits (e.g., .99, .017)
  Flt,
  /// Expect integer or regular floating-point number.
  IntOrFlt,
}

macro_rules! expect_num_kind {
  ($self:ident, $radix:expr, $num_type:expr) => {{
    $self.advance();
    ($radix, $num_type)
  }};
}

impl<'a> Lexer<'a> {
  /// Makes a numeric literal. This includes Binary, Octal, Decimal,
  /// Floating-Point, and Hexadecimal numbers.
  ///
  /// # Returns
  /// - `Token`: A numeric token (integer, float, binary, octal, or hex).
  pub(super) fn make_numeric_token(&mut self) -> Token {
    let mut has_period = false;
    let mut is_scientific = false;

    // Check which type of numeric literal we are lexing
    let (radix, num_kind): (u8, ExNumType) = match (self.get_previous(), self.get_current()) {
      ('0', 'x') | ('0', 'X') => expect_num_kind![self, 16, ExNumType::Hex],
      ('0', 'o') | ('0', 'O') => expect_num_kind![self, 8, ExNumType::Oct],
      ('0', 'b') | ('0', 'B') => expect_num_kind![self, 2, ExNumType::Bin],
      ('0', b) if b.is_ascii_digit() => return self.make_error_token(NoLeadZerosInNum, true),
      ('.', _) => (10, ExNumType::Flt),
      _ => (10, ExNumType::IntOrFlt),
    };

    while !self.is_at_end() && self.is_digit_char(radix) {
      if self.get_current() == '.' {
        // If the Lexer encounters double periods, then the current
        // expression is actually a Range, therefore, only consider
        // the first part of the range operand as a numeric literal.
        if self.get_next() == '.' {
          break;
        }

        // If the Lexer encounters a possible identifier after a period, then
        // the current expression is actually a member access, therefore, only
        // consider the first part of the member access as a numeric literal.
        if char_is_ident_start![self.get_next()] {
          break;
        }

        if is_scientific {
          return self.make_error_token(ExpectedIntExpo, true);
        }

        match num_kind {
          ExNumType::Hex => return self.make_error_token(DotInHex, true),
          ExNumType::Oct => return self.make_error_token(DotInOct, true),
          ExNumType::Bin => return self.make_error_token(DotInBin, true),
          ExNumType::Flt => return self.make_error_token(ExtraDotInFloat, true),
          _ => {}
        };

        if has_period {
          return self.make_error_token(ExtraDotInFloat, true);
        } else {
          has_period = true;
        }
      }

      if self.get_current() == '_' {
        if self.get_next() == '_' {
          self.advance();
          continue;
        }

        match self.get_previous() {
          '_' => return self.make_error_token(ExtraSepInNum, true),
          '.' => return self.make_error_token(SepAfterFloat, true),
          _ => {}
        }

        if self.get_next() == '.' {
          return self.make_error_token(SepBeforeFloat, true);
        }
      }

      if self.get_current() == 'e' || self.get_current() == 'E' {
        match num_kind {
          ExNumType::Oct => return self.make_error_token(NonOctChar, true),
          ExNumType::Bin => return self.make_error_token(NonBinChar, true),
          ExNumType::Flt | ExNumType::IntOrFlt => {
            if is_scientific {
              return self.make_error_token(ExtraEInScientific, true);
            } else {
              is_scientific = true
            }
          }
          _ => {}
        };
      }

      self.advance();
    }

    if self.get_previous() == '_' {
      return self.make_error_token_at_prev(SepAtEndOfNum);
    }

    if self.get_previous() == 'e' || self.get_previous() == 'E' {
      return self.make_error_token_at_prev(ExpectedIntExpo);
    }

    match num_kind {
      ExNumType::Hex => self.make_token(HEX_LIT),
      ExNumType::Oct => self.make_token(OCTAL_LIT),
      ExNumType::Bin => self.make_token(BINARY_LIT),
      ExNumType::Flt => self.make_token(if is_scientific { SCIENTIFIC_LIT } else { FLOAT_LIT }),
      ExNumType::IntOrFlt => self.make_token(if is_scientific {
        SCIENTIFIC_LIT
      } else if has_period {
        FLOAT_LIT
      } else {
        INT_LIT
      }),
    }
  }

  /// Checks whether the current character is a digit in the given radix, or an '_', or a '.'
  ///
  /// # Arguments
  /// * `radix`: The expected base of the digit.
  ///
  /// returns: bool
  fn is_digit_char(&self, radix: u8) -> bool {
    self.get_current().is_digit(radix as u32)
      || self.get_current() == '_'
      || self.get_current() == '.'
      || self.get_current() == 'e'
      || self.get_current() == 'E'
      || (self.get_current() == '-'
        && (self.get_previous() == 'e' || self.get_previous() == 'E')
        && self.get_next().is_ascii_digit())
  }
}
