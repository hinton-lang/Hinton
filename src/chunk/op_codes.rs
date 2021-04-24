/// The set of instructions supported by the virtual machine.
///
/// **NOTE:** Changing the order in which members are declared creates
/// incompatibilities between different versions of the interpreter.
#[allow(non_camel_case_types)]
#[derive(Debug, PartialEq, Clone)]
#[repr(u8)]
pub enum OpCode {
    // Interpreter V1.0.0
    OP_ADD,
    OP_BITWISE_AND,
    OP_BITWISE_L_SHIFT,
    OP_BITWISE_NOT,
    OP_BITWISE_OR,
    OP_BITWISE_R_SHIFT,
    OP_BITWISE_XOR,
    OP_VALUE,
    OP_DIVIDE,
    OP_EQUALS,
    OP_EXPO,
    OP_FALSE,
    OP_GET_VAR,
    OP_GREATER_THAN,
    OP_GREATER_THAN_EQ,
    OP_JUMP,
    OP_JUMP_IF_FALSE,
    OP_LESS_THAN,
    OP_LESS_THAN_EQ,
    OP_LOGIC_NOT,
    OP_LOOP,
    OP_MODULUS,
    OP_MULTIPLY,
    OP_NEGATE,
    OP_NOT_EQUALS,
    OP_NULL,
    OP_POP_STACK,
    OP_RETURN,
    OP_SET_VAR,
    OP_SUBTRACT,
    OP_TRUE,
    OP_GENERATE_RANGE,
    OP_TERNARY,
    OP_NULLISH_COALESCING,
    OP_PRE_INCREMENT,
    OP_POST_INCREMENT,
    OP_PRE_DECREMENT,
    OP_POST_DECREMENT,
    // Temporaries - These should always stay at
    // the bottom of the enum.
    OP_PRINT,
}

impl OpCode {
    /// Gets an OpCode by index.
    ///
    /// ## Arguments
    /// * `idx` – The index of the OpCode
    ///
    /// ## Returns
    /// `Option<OpCode>` – The OpCode at the given index.
    pub fn get(idx: u8) -> Option<OpCode> {
        return match idx {
            // Interpreter V1.0.0
            0 => Some(OpCode::OP_ADD),
            1 => Some(OpCode::OP_BITWISE_AND),
            2 => Some(OpCode::OP_BITWISE_L_SHIFT),
            3 => Some(OpCode::OP_BITWISE_NOT),
            4 => Some(OpCode::OP_BITWISE_OR),
            5 => Some(OpCode::OP_BITWISE_R_SHIFT),
            6 => Some(OpCode::OP_BITWISE_XOR),
            7 => Some(OpCode::OP_VALUE),
            8 => Some(OpCode::OP_DIVIDE),
            9 => Some(OpCode::OP_EQUALS),
            10 => Some(OpCode::OP_EXPO),
            11 => Some(OpCode::OP_FALSE),
            12 => Some(OpCode::OP_GET_VAR),
            13 => Some(OpCode::OP_GREATER_THAN),
            14 => Some(OpCode::OP_GREATER_THAN_EQ),
            15 => Some(OpCode::OP_JUMP),
            16 => Some(OpCode::OP_JUMP_IF_FALSE),
            17 => Some(OpCode::OP_LESS_THAN),
            18 => Some(OpCode::OP_LESS_THAN_EQ),
            19 => Some(OpCode::OP_LOGIC_NOT),
            20 => Some(OpCode::OP_LOOP),
            21 => Some(OpCode::OP_MODULUS),
            22 => Some(OpCode::OP_MULTIPLY),
            23 => Some(OpCode::OP_NEGATE),
            24 => Some(OpCode::OP_NOT_EQUALS),
            25 => Some(OpCode::OP_NULL),
            26 => Some(OpCode::OP_POP_STACK),
            27 => Some(OpCode::OP_RETURN),
            28 => Some(OpCode::OP_SET_VAR),
            29 => Some(OpCode::OP_SUBTRACT),
            30 => Some(OpCode::OP_TRUE),
            31 => Some(OpCode::OP_GENERATE_RANGE),
            32 => Some(OpCode::OP_TERNARY),
            33 => Some(OpCode::OP_NULLISH_COALESCING),
            34 => Some(OpCode::OP_PRE_INCREMENT),
            35 => Some(OpCode::OP_POST_INCREMENT),
            36 => Some(OpCode::OP_PRE_DECREMENT),
            37 => Some(OpCode::OP_POST_DECREMENT),
            38 => Some(OpCode::OP_PRINT),
            _ => None,
        };
    }
}
