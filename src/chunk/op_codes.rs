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
    OP_CONSTANT,
    OP_DEFINE_GLOBAL_VAR,
    OP_DIVIDE,
    OP_EQUALS,
    OP_EXPO,
    OP_FALSE,
    OP_GET_GLOBAL_VAR,
    OP_GET_LOCAL_VAR,
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
    OP_SET_GLOBAL_VAR,
    OP_SET_LOCAL_VAR,
    OP_SUBTRACT,
    OP_TRUE,
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
            7 => Some(OpCode::OP_CONSTANT),
            8 => Some(OpCode::OP_DEFINE_GLOBAL_VAR),
            9 => Some(OpCode::OP_DIVIDE),
            10 => Some(OpCode::OP_EQUALS),
            11 => Some(OpCode::OP_EXPO),
            12 => Some(OpCode::OP_FALSE),
            13 => Some(OpCode::OP_GET_GLOBAL_VAR),
            14 => Some(OpCode::OP_GET_LOCAL_VAR),
            15 => Some(OpCode::OP_GREATER_THAN),
            16 => Some(OpCode::OP_GREATER_THAN_EQ),
            17 => Some(OpCode::OP_JUMP),
            18 => Some(OpCode::OP_JUMP_IF_FALSE),
            19 => Some(OpCode::OP_LESS_THAN),
            20 => Some(OpCode::OP_LESS_THAN_EQ),
            21 => Some(OpCode::OP_LOGIC_NOT),
            22 => Some(OpCode::OP_LOOP),
            23 => Some(OpCode::OP_MODULUS),
            24 => Some(OpCode::OP_MULTIPLY),
            25 => Some(OpCode::OP_NEGATE),
            26 => Some(OpCode::OP_NOT_EQUALS),
            27 => Some(OpCode::OP_NULL),
            28 => Some(OpCode::OP_POP_STACK),
            29 => Some(OpCode::OP_RETURN),
            30 => Some(OpCode::OP_SET_GLOBAL_VAR),
            31 => Some(OpCode::OP_SET_LOCAL_VAR),
            32 => Some(OpCode::OP_SUBTRACT),
            33 => Some(OpCode::OP_TRUE),
            34 => Some(OpCode::OP_PRINT),
            _ => None,
        };
    }
}
