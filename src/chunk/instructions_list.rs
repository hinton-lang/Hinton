use super::op_codes::OpCode;

/// A list that holds the ByteCode instructions in a chunk.
pub struct InstructionsList {
    instructions: Vec<u8>,
}

impl InstructionsList {
    /// Creates a new instructions list.
    ///
    /// ## Returns
    /// `InstructionsList` – a new instructions list.
    pub fn new() -> InstructionsList {
        Self { instructions: Vec::new() }
    }

    /// Gets the length of the instructions list.
    ///
    /// ## Returns
    /// * `usize` – The length of the instructions list.
    pub fn len(&self) -> usize {
        return self.instructions.len();
    }

    /// Returns the OpCode associated with a byte instruction in the
    /// instructions list.
    ///
    /// ## Arguments
    /// * `idx` – The index of the instruction in the instructions list.
    ///
    /// ## Returns
    /// `Option<OpCode>` – The OpCode instruction at the given index.
    pub fn get_op_code(&self, idx: usize) -> Option<OpCode> {
        let byte = self.instructions.get(idx);

        match byte {
            Some(x) => OpCode::get(*x),
            None => None,
        }
    }

    /// Adds a ByteCode instruction from a given OpCode into the
    /// instructions list.
    ///
    /// ## Arguments
    /// * `val` – The OpCode instruction to add to the list.
    pub fn push_op_code(&mut self, val: OpCode) {
        self.instructions.push(val as u8);
    }

    /// Retrieves a raw byte from the instructions list
    ///
    /// ## Arguments
    /// * `idx` – The index of the instruction in the instructions list.
    ///
    /// ## Returns
    /// `Option<u8>` – The byte instruction at the given index.
    pub fn get_byte(&mut self, idx: usize) -> Option<u8> {
        let byte = self.instructions.get(idx);

        match byte {
            Some(x) => Some(*x),
            None => None,
        }
    }

    /// Adds a raw byte into the instructions list
    ///
    /// ## Arguments
    /// * `val` – The byte to add to the instructions list.
    pub fn push_byte(&mut self, val: u8) {
        self.instructions.push(val);
    }

    /// Retrieves the current byte and the next byte in the instructions list
    /// to form a 16-bit short.
    ///
    /// ## Arguments
    /// * `idx` – The index of the first byte in the instructions list.
    ///
    /// ## Returns
    /// `u16` – The short generated from the two bytes.
    pub fn get_short(&self, idx: usize) -> u16 {
        let b1 = self.instructions.get(idx).unwrap();
        let b2 = self.instructions.get(idx + 1).unwrap();

        return u16::from_be_bytes([*b1, *b2]);
    }

    /// Splits a 16-bit integer into two bytes, and adds each individual
    /// byte in sequence into the instructions list.
    ///
    /// ## Arguments
    /// * `val` – The short to add to the instruction list.
    pub fn push_short(&mut self, val: u16) {
        let short = val.to_be_bytes();

        self.instructions.push(short[0]);
        self.instructions.push(short[1]);
    }
}
