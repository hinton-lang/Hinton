use crate::lexer::tokens::TokenType;

/// Represents the precedence of different expressions in ascending order.
/// For example, `PREC_EQUALITY` has lower precedence than `PREC_UNARY` because
/// `PREC_EQUALITY` appears earlier in the enum, and `PREC_UNARY` appears after.
#[allow(non_camel_case_types)]
#[derive(Debug, PartialEq, Clone)]
#[allow(dead_code)]
#[repr(u8)]
pub enum Precedence {
    PREC_NONE,
    PREC_ASSIGNMENT,    // =, ++, --, +=, -=,
    PREC_TERNARY,       // ?:
    PREC_NULLISH,       // ??
    PREC_LOGIC_OR,      // or ||
    PREC_LOGIC_AND,     // and &&
    PREC_BITWISE_OR,    // |
    PREC_BITWISE_XOR,   // ^
    PREC_BITWISE_AND,   // &
    PREC_EQUALITY,      // == !=
    PREC_COMPARISON,    // < > <= >=
    PREC_RANGE,         // ..
    PREC_BITWISE_SHIFT, // << >>
    PREC_TERM,          // + -
    PREC_FACTOR,        // * / %
    PREC_EXPO,          // **
    PREC_UNARY,         // ~ ! -
    PREC_CALL,          // . ()
    PREC_PRIMARY,
}

impl Precedence {
    /// Gets the Precedence variant associated with a given numeric value.
    ///
    /// ## Arguments
    /// * `val` – The numeric value of the expected variant.
    ///
    /// ## Returns
    /// `Precedence` – the Precedence variant associated with the
    /// provided numeric value
    pub fn get_by_val(val: u8) -> Precedence {
        match val {
            0 => Precedence::PREC_NONE,
            1 => Precedence::PREC_ASSIGNMENT,
            2 => Precedence::PREC_TERNARY,
            3 => Precedence::PREC_NULLISH,
            4 => Precedence::PREC_LOGIC_OR,
            5 => Precedence::PREC_LOGIC_AND,
            6 => Precedence::PREC_BITWISE_OR,
            7 => Precedence::PREC_BITWISE_XOR,
            8 => Precedence::PREC_BITWISE_AND,
            9 => Precedence::PREC_EQUALITY,
            10 => Precedence::PREC_COMPARISON,
            11 => Precedence::PREC_RANGE,
            12 => Precedence::PREC_BITWISE_SHIFT,
            13 => Precedence::PREC_TERM,
            14 => Precedence::PREC_FACTOR,
            15 => Precedence::PREC_EXPO,
            16 => Precedence::PREC_UNARY,
            17 => Precedence::PREC_CALL,
            18 => Precedence::PREC_PRIMARY,
            _ => Precedence::PREC_NONE, // Should never be reached
        }
    }
}

/// The set compiling function that can be associated
/// with a given token
pub enum ParseFn {
    CompileBinaryNum,
    CompileBinaryExpr,
    CompileUnary,
    CompileLiteral,
    CompileHexNum,
    CompileVariable,
    CompileLogicAnd,
    CompileLogicOr,
    CompileGrouping,
    CompileString,
    CompileTernary,
    CompileOctalNum,
    CompileNumeric,
    CompilePreIncrement,
    CompilePostIncrement,
    CompilePreDecrement,
    CompilePostDecrement,
    NONE, // Do not call a parsing function
}

/// Wraps the parsing properties of a token so that they
// can be easily accessed throughout the compiler.
pub struct ParserRule {
    pub prefix: ParseFn,
    pub infix: ParseFn,
    pub postfix: ParseFn,
    pub precedence: Precedence,
}

pub fn get_rule(tok_type: TokenType) -> ParserRule {
    match tok_type {
        TokenType::BINARY_LITERAL => ParserRule {
            prefix: ParseFn::CompileBinaryNum,
            infix: ParseFn::NONE,
            postfix: ParseFn::NONE,
            precedence: Precedence::PREC_ASSIGNMENT, // TODO: Shouldn't this be PREC_PRIMARY?
        },

        TokenType::BITWISE_AND => ParserRule {
            prefix: ParseFn::NONE,
            infix: ParseFn::CompileBinaryExpr,
            postfix: ParseFn::NONE,
            precedence: Precedence::PREC_BITWISE_AND,
        },

        TokenType::BITWISE_LEFT_SHIFT => ParserRule {
            prefix: ParseFn::NONE,
            infix: ParseFn::CompileBinaryExpr,
            postfix: ParseFn::NONE,
            precedence: Precedence::PREC_BITWISE_SHIFT,
        },

        TokenType::BITWISE_NOT => ParserRule {
            prefix: ParseFn::CompileUnary,
            infix: ParseFn::NONE,
            postfix: ParseFn::NONE,
            precedence: Precedence::PREC_UNARY,
        },

        TokenType::BITWISE_OR => ParserRule {
            prefix: ParseFn::NONE,
            infix: ParseFn::CompileBinaryExpr,
            postfix: ParseFn::NONE,
            precedence: Precedence::PREC_BITWISE_OR,
        },

        TokenType::BITWISE_RIGHT_SHIFT => ParserRule {
            prefix: ParseFn::NONE,
            infix: ParseFn::CompileBinaryExpr,
            postfix: ParseFn::NONE,
            precedence: Precedence::PREC_BITWISE_SHIFT,
        },

        TokenType::BITWISE_XOR => ParserRule {
            prefix: ParseFn::NONE,
            infix: ParseFn::CompileBinaryExpr,
            postfix: ParseFn::NONE,
            precedence: Precedence::PREC_BITWISE_XOR,
        },

        TokenType::DECREMENT => ParserRule {
            prefix: ParseFn::CompilePreIncrement,
            infix: ParseFn::NONE,
            postfix: ParseFn::CompilePostDecrement,
            precedence: Precedence::PREC_ASSIGNMENT,
        },

        TokenType::EXPO => ParserRule {
            prefix: ParseFn::NONE,
            infix: ParseFn::CompileBinaryExpr,
            postfix: ParseFn::NONE,
            precedence: Precedence::PREC_EXPO,
        },

        TokenType::FALSE_LITERAL => ParserRule {
            prefix: ParseFn::CompileLiteral,
            infix: ParseFn::NONE,
            postfix: ParseFn::NONE,
            precedence: Precedence::PREC_NONE,
        },

        TokenType::GREATER_THAN => ParserRule {
            prefix: ParseFn::NONE,
            infix: ParseFn::CompileBinaryExpr,
            postfix: ParseFn::NONE,
            precedence: Precedence::PREC_COMPARISON,
        },

        TokenType::GREATER_THAN_EQ => ParserRule {
            prefix: ParseFn::NONE,
            infix: ParseFn::CompileBinaryExpr,
            postfix: ParseFn::NONE,
            precedence: Precedence::PREC_COMPARISON,
        },

        TokenType::HEXADECIMAL_LITERAL => ParserRule {
            prefix: ParseFn::CompileHexNum,
            infix: ParseFn::NONE,
            postfix: ParseFn::NONE,
            precedence: Precedence::PREC_NONE, // TODO: Shouldn't this be PREC_PRIMARY?
        },

        TokenType::IDENTIFIER => ParserRule {
            prefix: ParseFn::CompileVariable,
            infix: ParseFn::NONE,
            postfix: ParseFn::NONE,
            precedence: Precedence::PREC_NONE, // TODO: Shouldn't this be PREC_PRIMARY?
        },

        TokenType::INCREMENT => ParserRule {
            prefix: ParseFn::CompilePreIncrement,
            infix: ParseFn::NONE,
            postfix: ParseFn::CompilePostIncrement,
            precedence: Precedence::PREC_ASSIGNMENT,
        },

        TokenType::LESS_THAN => ParserRule {
            prefix: ParseFn::NONE,
            infix: ParseFn::CompileBinaryExpr,
            postfix: ParseFn::NONE,
            precedence: Precedence::PREC_COMPARISON,
        },

        TokenType::LESS_THAN_EQ => ParserRule {
            prefix: ParseFn::NONE,
            infix: ParseFn::CompileBinaryExpr,
            postfix: ParseFn::NONE,
            precedence: Precedence::PREC_COMPARISON,
        },

        TokenType::LOGICAL_AND => ParserRule {
            prefix: ParseFn::NONE,
            infix: ParseFn::CompileLogicAnd,
            postfix: ParseFn::NONE,
            precedence: Precedence::PREC_LOGIC_AND,
        },

        TokenType::LOGICAL_EQ => ParserRule {
            prefix: ParseFn::NONE,
            infix: ParseFn::CompileBinaryExpr,
            postfix: ParseFn::NONE,
            precedence: Precedence::PREC_EQUALITY,
        },

        TokenType::LOGICAL_NOT => ParserRule {
            prefix: ParseFn::CompileUnary,
            infix: ParseFn::NONE,
            postfix: ParseFn::NONE,
            precedence: Precedence::PREC_NONE,
        },

        TokenType::LOGICAL_NOT_EQ => ParserRule {
            prefix: ParseFn::NONE,
            infix: ParseFn::CompileBinaryExpr,
            postfix: ParseFn::NONE,
            precedence: Precedence::PREC_EQUALITY,
        },

        TokenType::LOGICAL_OR => ParserRule {
            prefix: ParseFn::NONE,
            infix: ParseFn::CompileLogicOr,
            postfix: ParseFn::NONE,
            precedence: Precedence::PREC_LOGIC_OR,
        },

        TokenType::LEFT_PARENTHESIS => ParserRule {
            prefix: ParseFn::CompileGrouping,
            infix: ParseFn::NONE,
            postfix: ParseFn::NONE,
            precedence: Precedence::PREC_NONE,
        },

        TokenType::MINUS => ParserRule {
            prefix: ParseFn::CompileUnary,
            infix: ParseFn::CompileBinaryExpr,
            postfix: ParseFn::NONE,
            precedence: Precedence::PREC_TERM,
        },

        TokenType::MODULUS => ParserRule {
            prefix: ParseFn::NONE,
            infix: ParseFn::CompileBinaryExpr,
            postfix: ParseFn::NONE,
            precedence: Precedence::PREC_FACTOR,
        },

        TokenType::NULLISH_COALESCING => ParserRule {
            prefix: ParseFn::NONE,
            infix: ParseFn::CompileBinaryExpr,
            postfix: ParseFn::NONE,
            precedence: Precedence::PREC_NULLISH,
        },

        TokenType::NULL_LITERAL => ParserRule {
            prefix: ParseFn::CompileLiteral,
            infix: ParseFn::NONE,
            postfix: ParseFn::NONE,
            precedence: Precedence::PREC_NONE, // TODO: Shouldn't this be PREC_PRIMARY?
        },

        TokenType::NUMERIC_LITERAL => ParserRule {
            prefix: ParseFn::CompileNumeric,
            infix: ParseFn::NONE,
            postfix: ParseFn::NONE,
            precedence: Precedence::PREC_NONE, // TODO: Shouldn't this be PREC_PRIMARY?
        },

        TokenType::OCTAL_LITERAL => ParserRule {
            prefix: ParseFn::CompileOctalNum,
            infix: ParseFn::NONE,
            postfix: ParseFn::NONE,
            precedence: Precedence::PREC_NONE, // TODO: Shouldn't this be PREC_PRIMARY?
        },

        TokenType::PLUS => ParserRule {
            prefix: ParseFn::NONE,
            infix: ParseFn::CompileBinaryExpr,
            postfix: ParseFn::NONE,
            precedence: Precedence::PREC_TERM,
        },

        TokenType::QUESTION_MARK => ParserRule {
            prefix: ParseFn::NONE,
            infix: ParseFn::CompileTernary,
            postfix: ParseFn::NONE,
            precedence: Precedence::PREC_TERNARY,
        },

        TokenType::RANGE_OPERATOR => ParserRule {
            prefix: ParseFn::NONE,
            infix: ParseFn::CompileBinaryExpr,
            postfix: ParseFn::NONE,
            precedence: Precedence::PREC_RANGE,
        },

        TokenType::SLASH => ParserRule {
            prefix: ParseFn::NONE,
            infix: ParseFn::CompileBinaryExpr,
            postfix: ParseFn::NONE,
            precedence: Precedence::PREC_FACTOR,
        },

        TokenType::STAR => ParserRule {
            prefix: ParseFn::NONE,
            infix: ParseFn::CompileBinaryExpr,
            postfix: ParseFn::NONE,
            precedence: Precedence::PREC_FACTOR,
        },

        TokenType::STRING_LITERAL => ParserRule {
            prefix: ParseFn::CompileString,
            infix: ParseFn::NONE,
            postfix: ParseFn::NONE,
            precedence: Precedence::PREC_NONE, // TODO: Shouldn't this be PREC_PRIMARY?
        },

        TokenType::TRUE_LITERAL => ParserRule {
            prefix: ParseFn::CompileLiteral,
            infix: ParseFn::NONE,
            postfix: ParseFn::NONE,
            precedence: Precedence::PREC_NONE, // TODO: Shouldn't this be PREC_PRIMARY?
        },

        // The rest of the tokens do not have a parse rule
        _ => ParserRule {
            prefix: ParseFn::NONE,
            infix: ParseFn::NONE,
            postfix: ParseFn::NONE,
            precedence: Precedence::PREC_NONE,
        },
    }
}
