use super::objects::Object;
use num_traits::FromPrimitive;

/// The set of instructions supported by the virtual machine.
///
/// **NOTE:** Changing the order in which members are declared creates
/// incompatibilities between different versions of the interpreter.
#[derive(Debug, Clone)]
#[repr(u8)]
#[derive(FromPrimitive)]
pub enum OpCode {
    // Instructions with zero chunk operands.
    // While these instructions do not have any
    // bytecode operands, some of them do have
    // object operands from the stack.
    Add,
    BitwiseAnd,
    BitwiseNot,
    BitwiseOr,
    BitwiseShiftLeft,
    BitwiseShiftRight,
    BitwiseXor,
    Divide,
    Equals,
    Expo,
    GreaterThan,
    GreaterThanEq,
    Indexing,
    LessThan,
    LessThanEq,
    LoadImm0F,
    LoadImm0I,
    LoadImm1F,
    LoadImm1I,
    LoadImmFalse,
    LoadImmNull,
    LoadImmTrue,
    LogicNot,
    MakeRange,
    Modulus,
    Multiply,
    Negate,
    NotEq,
    NullishCoalescing,
    PopStack,
    Subtract,
    LoadNative,

    // Instructions with one chunk operands.
    // These instructions use the next byte
    // from the chunk as its operand.
    BindDefaults,
    FuncCall,
    GetVar,
    LoadConstant,
    LoadImm,
    LoopJump,
    MakeArray,
    Return,
    SetVar,

    // Instructions with two chunk operands.
    // These instructions use the next two
    // bytes (a short) as their operands.
    GetVarLong,
    Jump,
    JumpIfFalse,
    LoadConstantLong,
    LoadImmLong,
    LoopJumpLong,
    MakeArrayLong,
    SetVarLong,
}

/// Contains all the necessary information about
/// the instructions to be executed.
#[derive(Clone)]
pub struct Chunk {
    instructions: Vec<u8>,
    locations: Vec<(usize, usize)>,
    constants: Vec<Object>,
}

impl<'a> Chunk {
    /// Creates a new chunk.
    ///
    /// ## Returns
    /// `Chunk` – a new chunk.
    pub fn new() -> Self {
        Self {
            instructions: vec![],
            locations: Vec::new(),
            constants: Vec::new(),
        }
    }

    /// Adds a constant to this chunk's constants pool
    ///
    /// ## Arguments
    /// * `obj` – The object to be added to the pool
    ///
    /// ## Returns
    /// * `Result<u16, ()>` – If the object was successfully added to the pool,
    /// returns the position of the object in the pool. If the item could not be
    // added because the pool is full, returns error.
    pub fn add_constant(&mut self, obj: Object) -> Result<u16, ()> {
        return if self.constants.len() < (u16::MAX as usize) {
            // Having to create an iterator, then enumerating that iterator, to finally
            // look for the object seems very expensive. Can we do better? Do the benefits
            // of storing a single object in the heap outweigh the cost of these operations?
            // TODO: Find a better way of doing this, or do research to see if the benefits outweigh the cost.
            match self.constants.iter().enumerate().find(|x| x.1.equals(&obj)) {
                Some(x) => Ok(x.0 as u16),
                None => {
                    self.constants.push(obj);
                    Ok((self.constants.len() as u16) - 1)
                }
            }
        } else {
            Err(())
        };
    }

    /// Retrieves a constant from this chunk's constants pool.
    ///
    /// ## Arguments
    /// * `idx` – The index of the object constant.
    ///
    /// ## Returns
    /// `Option<&Object>` – The object at the given index in the constant pool
    pub fn get_constant(&self, idx: usize) -> Option<&Object> {
        self.constants.get(idx)
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
            Some(x) => FromPrimitive::from_u8(*x),
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
    pub fn get_byte(&self, idx: usize) -> Option<u8> {
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
    /// `Option<u16>` – The short generated from the two bytes.
    pub fn get_short(&self, idx: usize) -> Option<u16> {
        let b1 = match self.instructions.get(idx) {
            Some(byte) => byte,
            None => return None,
        };

        let b2 = match self.instructions.get(idx + 1) {
            Some(byte) => byte,
            None => return None,
        };

        return Some(u16::from_be_bytes([*b1, *b2]));
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

    /// Modifies the byte value at the specified chunk index.
    ///
    /// ## Arguments
    /// * `idx` – The index in the chunk of the byte to be modified.
    /// * `new_val` – The new value of the byte.
    pub fn modify_byte(&mut self, idx: usize, new_val: u8) {
        self.instructions[idx] = new_val;
    }

    pub fn get_line_info(&self, idx: usize) -> Option<&(usize, usize)> {
        return self.locations.get(idx);
    }

    pub fn push_line_info(&mut self, line_info: (usize, usize)) {
        self.locations.push(line_info);
    }

    /// Gets the length of the instructions list.
    ///
    /// ## Returns
    /// * `usize` – The length of the instructions list.
    pub fn len(&self) -> usize {
        return self.instructions.len();
    }
}

/// Disassembles the chunk, printing the each instruction and
/// their related information.
///
/// ## Arguments
/// * `name` – the name to print for the current chunk
pub fn disassemble_chunk(chunk: &Chunk, name: &str) {
    // prints this chunk's name
    println!("==== {} ====", name);

    let mut current_line = 0;

    let mut idx = 0;
    while idx < chunk.len() {
        let code = chunk.get_op_code(idx);
        let line_info = chunk.get_line_info(idx);

        // Prints a line number or a vertical bar indicating that the
        // current instruction is in the same line as the previous one.
        match line_info {
            Some(location) => {
                if location.0 != current_line {
                    print!("{:>03}\t", location.0);
                    current_line = location.0;
                } else {
                    print!(" |\t")
                }
            }
            _ => print!("??? "),
        }

        // Prints the index of the current instruction

        print!("{:>04} ", idx);
        // Prints the instruction name
        match code {
            Some(instr) => {
                // Prints the instruction with a teal color
                print!(
                    "\x1b[32m{:#04X}\x1b[0m – \x1b[36m{:?}\x1b[0m ",
                    instr.clone() as u8,
                    instr
                );

                // Reads two bytes as the index of a constant
                let mut const_val = |is_long: bool| -> &Object {
                    idx += 1;

                    let pos;
                    if is_long {
                        pos = match chunk.get_short(idx) {
                            Some(short) => short as usize,
                            None => unreachable!("Could not get short."),
                        };
                        idx += 1; // increment `i` again for the second byte in the short
                    } else {
                        pos = match chunk.get_byte(idx) {
                            Some(byte) => byte as usize,
                            None => unreachable!("Could not get byte."),
                        };
                    }

                    chunk.get_constant(pos).unwrap()
                };

                match instr {
                    // Prints the value associated with an OP_CONSTANT instruction
                    OpCode::LoadConstant => println!("\t---> {}", const_val(false)),
                    OpCode::LoadConstantLong => println!("\t---> {}", const_val(true)),
                    OpCode::GetVar
                    | OpCode::SetVar
                    | OpCode::MakeArray
                    | OpCode::FuncCall
                    | OpCode::BindDefaults
                    | OpCode::Return
                    | OpCode::LoadImm => {
                        idx += 1;
                        println!("\t{}", chunk.get_byte(idx).unwrap());
                    }

                    OpCode::LoopJump => {
                        idx += 1;
                        println!("\t{}", (idx + 1) - (chunk.get_byte(idx).unwrap() as usize));
                    }

                    OpCode::GetVarLong
                    | OpCode::SetVarLong
                    | OpCode::LoopJumpLong
                    | OpCode::MakeArrayLong
                    | OpCode::LoadImmLong => {
                        idx += 2;
                        println!("\t{}", chunk.get_short(idx - 1).unwrap());
                    }

                    OpCode::Jump | OpCode::JumpIfFalse => {
                        idx += 2;
                        println!(
                            "\t{}",
                            (chunk.get_short(idx - 1).unwrap() as usize) + idx + 2
                        );
                    }

                    // If the instruction does not use the next to bytes, then print nothing
                    _ => println!(),
                }
            }
            None => println!("No Instruction Found..."),
        }

        idx += 1;
    }

    println!();
}

/// Disassembles the chunk into its raw bytes, and prints the each instruction byte.
/// This is useful when comparing the chunk generated by a program vs another program.
///
/// ## Arguments
///
/// * `name` – the name to print for the current chunk
pub fn print_raw(chunk: &Chunk, name: &str) {
    let mut i = 0;

    // prints this chunk's name
    println!("==== {} ====", name);

    while i < chunk.instructions.len() {
        match chunk.get_byte(i) {
            Some(instr) => print!("{:#04X} ", instr as u8),
            None => print!("\x1b[36mNONE\x1b[0m "),
        }

        if (i + 1) % 8 == 0 {
            println!();
        }

        i += 1;
    }

    println!("\n\nChunk Size: {}", i);
    println!("================\n");
}
