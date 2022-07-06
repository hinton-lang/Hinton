use crate::core::tokens::Token;
use crate::core::tokens::TokenType::*;
use crate::lexer::Lexer;

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
   /// Expect integer of regular floating-point number.
   IntOrFlt,
}

macro_rules! expect_num_type {
   ( $self:ident, $radix:expr, $num_type:expr ) => {{
      $self.advance();
      ($radix, $num_type)
   }};
}

impl Lexer {
   /// Makes a numeric literal. This includes Binary, Octal, Decimal,
   /// Floating-Point, and Hexadecimal numbers.
   ///
   /// # Returns
   /// - `Token`: A numeric token (integer, float, binary, octal, or hex).
   pub(super) fn make_numeric_token(&mut self) -> Token {
      let mut has_period = false;

      // Check which type of numeric literal we are lexing
      let (radix, num_type): (u8, ExNumType) = match (self.get_previous(), self.get_current()) {
         ('0', 'x') | ('0', 'X') => expect_num_type![self, 16, ExNumType::Hex],
         ('0', 'o') | ('0', 'O') => expect_num_type![self, 8, ExNumType::Oct],
         ('0', 'b') | ('0', 'B') => expect_num_type![self, 2, ExNumType::Bin],
         ('.', _) => (10, ExNumType::Flt),
         _ => (10, ExNumType::IntOrFlt),
      };

      while !self.is_at_end() && self.is_digit_char(radix) {
         if self.get_current() == '.' {
            // If the lexer encounters double periods, then the current
            // expression is actually a Range, therefore, only consider
            // the first part of the range operand as numeric literal.
            if self.next() == '.' {
               break;
            }

            match num_type {
               ExNumType::Hex => return self.make_error_token("Unexpected '.' in hexadecimal literal."),
               ExNumType::Oct => return self.make_error_token("Unexpected '.' in octal literal."),
               ExNumType::Bin => return self.make_error_token("Unexpected '.' in binary literal."),
               ExNumType::Flt => return self.make_error_token("Unexpected extra '.' in float literal."),
               _ => {}
            };

            if has_period {
               return self.make_error_token("Unexpected extra '.' in float literal.");
            } else {
               has_period = true;
            }
         }

         if self.get_current() == '_' {
            match self.get_previous() {
               '_' => return self.make_error_token("Too many underscores in numeric literal separator."),
               '.' => return self.make_error_token("Separator not allowed after floating point."),
               _ => {}
            }

            if self.next() == '.' {
               return self.make_error_token("Separator not allowed before floating point.");
            }
         }

         self.advance();
      }

      if self.get_previous() == '_' {
         return self.make_error_token_at_prev("Separator not allowed at the end of numeric literal.");
      }

      match num_type {
         ExNumType::Hex => self.make_token(HEXADECIMAL),
         ExNumType::Oct => self.make_token(OCTAL),
         ExNumType::Bin => self.make_token(BINARY),
         ExNumType::Flt => self.make_token(FLOAT),
         ExNumType::IntOrFlt => self.make_token(if has_period { FLOAT } else { INTEGER }),
      }
   }

   /// Checks whether the current character is a digit in the given radix, or an '_', or a '.'
   ///
   /// # Arguments
   /// * `radix`: The expected base of the digit.
   ///
   /// returns: bool
   fn is_digit_char(&self, radix: u8) -> bool {
      self.get_current().is_digit(radix as u32) || self.get_current() == '_' || self.get_current() == '.'
   }
}
