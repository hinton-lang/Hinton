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
    OP_ARRAY,
    OP_ARRAY_LONG,
    OP_BITWISE_AND,
    OP_BITWISE_L_SHIFT,
    OP_BITWISE_NOT,
    OP_BITWISE_OR,
    OP_BITWISE_R_SHIFT,
    OP_BITWISE_XOR,
    OP_LOAD_IMM_0,
    OP_LOAD_IMM_1,
    OP_DIVIDE,
    OP_EQUALS,
    OP_EXPO,
    OP_FALSE,
    OP_GENERATE_RANGE,
    OP_GET_VAR,
    OP_GET_VAR_LONG,
    OP_GREATER_THAN,
    OP_GREATER_THAN_EQ,
    OP_JUMP,
    OP_JUMP_IF_FALSE,
    OP_LESS_THAN,
    OP_LESS_THAN_EQ,
    OP_LOAD_CONST,
    OP_LOAD_CONST_LONG,
    OP_LOGIC_NOT,
    OP_LOOP_JUMP,
    OP_LOOP_JUMP_LONG,
    OP_MODULUS,
    OP_MULTIPLY,
    OP_NEGATE,
    OP_NOT_EQUALS,
    OP_NULL,
    OP_NULLISH_COALESCING,
    OP_POP_STACK,
    OP_POST_DECREMENT,
    OP_POST_DECREMENT_LONG,
    OP_POST_INCREMENT,
    OP_POST_INCREMENT_LONG,
    OP_PRE_DECREMENT,
    OP_PRE_DECREMENT_LONG,
    OP_PRE_INCREMENT,
    OP_PRE_INCREMENT_LONG,
    OP_RETURN,
    OP_SET_VAR,
    OP_SET_VAR_LONG,
    OP_SUBTRACT,
    OP_TRUE,
    OP_ARRAY_INDEXING,
    OP_LOAD_IMM,
    OP_LOAD_IMM_LONG,
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
    pub fn get(idx: &u8) -> Option<OpCode> {
        return match idx {
            0 => Some(OpCode::OP_ADD),
            1 => Some(OpCode::OP_ARRAY),
            2 => Some(OpCode::OP_ARRAY_LONG),
            3 => Some(OpCode::OP_BITWISE_AND),
            4 => Some(OpCode::OP_BITWISE_L_SHIFT),
            5 => Some(OpCode::OP_BITWISE_NOT),
            6 => Some(OpCode::OP_BITWISE_OR),
            7 => Some(OpCode::OP_BITWISE_R_SHIFT),
            8 => Some(OpCode::OP_BITWISE_XOR),
            9 => Some(OpCode::OP_LOAD_IMM_0),
            10 => Some(OpCode::OP_LOAD_IMM_1),
            11 => Some(OpCode::OP_DIVIDE),
            12 => Some(OpCode::OP_EQUALS),
            13 => Some(OpCode::OP_EXPO),
            14 => Some(OpCode::OP_FALSE),
            15 => Some(OpCode::OP_GENERATE_RANGE),
            16 => Some(OpCode::OP_GET_VAR),
            17 => Some(OpCode::OP_GET_VAR_LONG),
            18 => Some(OpCode::OP_GREATER_THAN),
            19 => Some(OpCode::OP_GREATER_THAN_EQ),
            20 => Some(OpCode::OP_JUMP),
            21 => Some(OpCode::OP_JUMP_IF_FALSE),
            22 => Some(OpCode::OP_LESS_THAN),
            23 => Some(OpCode::OP_LESS_THAN_EQ),
            24 => Some(OpCode::OP_LOAD_CONST),
            25 => Some(OpCode::OP_LOAD_CONST_LONG),
            26 => Some(OpCode::OP_LOGIC_NOT),
            27 => Some(OpCode::OP_LOOP_JUMP),
            28 => Some(OpCode::OP_LOOP_JUMP_LONG),
            29 => Some(OpCode::OP_MODULUS),
            30 => Some(OpCode::OP_MULTIPLY),
            31 => Some(OpCode::OP_NEGATE),
            32 => Some(OpCode::OP_NOT_EQUALS),
            33 => Some(OpCode::OP_NULL),
            34 => Some(OpCode::OP_NULLISH_COALESCING),
            35 => Some(OpCode::OP_POP_STACK),
            36 => Some(OpCode::OP_POST_DECREMENT),
            37 => Some(OpCode::OP_POST_DECREMENT_LONG),
            38 => Some(OpCode::OP_POST_INCREMENT),
            39 => Some(OpCode::OP_POST_INCREMENT_LONG),
            40 => Some(OpCode::OP_PRE_DECREMENT),
            41 => Some(OpCode::OP_PRE_DECREMENT_LONG),
            42 => Some(OpCode::OP_PRE_INCREMENT),
            43 => Some(OpCode::OP_PRE_INCREMENT_LONG),
            44 => Some(OpCode::OP_RETURN),
            45 => Some(OpCode::OP_SET_VAR),
            46 => Some(OpCode::OP_SET_VAR_LONG),
            47 => Some(OpCode::OP_SUBTRACT),
            48 => Some(OpCode::OP_TRUE),
            49 => Some(OpCode::OP_ARRAY_INDEXING),

            50 => Some(OpCode::OP_LOAD_IMM),
            51 => Some(OpCode::OP_LOAD_IMM_LONG),

            52 => Some(OpCode::OP_PRINT),
            _ => None,
        };
    }
}
