pub mod instructions_list;
pub mod op_codes;

use op_codes::OpCode;

use super::objects::Object;

/// The result of storing a constant object
/// into the constant pool.
pub enum ConstantPos {
    Pos(u16),
    Error,
}

/// Contains all the necessary information about
/// the instructions to be executed.
#[derive(Clone)]
pub struct Chunk {
    /// The list of op_code instructions
    pub codes: instructions_list::InstructionsList,
    /// Stores the line and column of each op_code in the source
    /// code. This is useful when throwing runtime errors
    pub locations: Vec<(usize, usize)>,
    /// The literal constant values found in this chuck of code.
    constants: Vec<Object>,
}

impl<'a> Chunk {
    /// Creates a new chunk.
    ///
    /// ## Returns
    /// `Chunk` – a new chunk.
    pub fn new() -> Self {
        Self {
            codes: instructions_list::InstructionsList::new(),
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
    /// * `ConstantPos` – If the object was successfully added to the pool,
    /// returns the variant `ConstantPos::Pos(u16)` with the position of the
    /// object in the pool. If the item could not be added because the pool is
    /// full, returns the enum variant `ConstantPos::Error`.
    pub fn add_constant(&mut self, obj: Object) -> ConstantPos {
        return if self.constants.len() < (u16::MAX as usize) {
            // Having to create an iterator, then enumerating that iterator, to finally
            // look for the object seems very expensive. Can we do better? Do the benefits
            // of storing a single object in the heap outweigh the cost of these operations?
            // TODO: Find a better way of doing this, or do research to see if the benefits outweigh the cost.
            match self.constants.iter().enumerate().find(|x| x.1.equals(&obj)) {
                Some(x) => ConstantPos::Pos(x.0 as u16),
                None => {
                    self.constants.push(obj);
                    ConstantPos::Pos((self.constants.len() as u16) - 1)
                }
            }
        } else {
            ConstantPos::Error
        };
    }

    /// Retrieves a constant from this chunk's constants pool.
    ///
    /// ## Arguments
    /// * `idx` – The index of the object constant.
    ///
    /// ## Returns
    /// `Option<&Rc<Object<'a>>>` – The object at the given index in the constant pool
    pub fn get_constant(&self, idx: usize) -> Option<&Object> {
        self.constants.get(idx)
    }

    /// Gets the size of the constant pool for this chunk.
    ///
    /// ## Returns
    /// `usize` – The size of the constant pool.
    pub fn get_pool_size(&self) -> usize {
        self.constants.len()
    }

    /// Disassembles the chunk, printing the each instruction and
    /// their related information.
    ///
    /// ## Arguments
    /// * `name` – the name to print for the current chunk
    pub fn disassemble(&mut self, name: &str) {
        let mut i = 0;
        let mut current_line = 0;

        // prints this chunk's name
        println!("==== {} ====", name);

        while i < self.codes.len() {
            let code = self.codes.get_op_code(i);
            let location = self.locations.get(i);

            // Prints the index of the current instruction
            print!("{:>04} ", i);

            // Prints a line number or a vertical bar indicating that the
            // current instruction is in the same line as the previous one.
            match location {
                Some(place) => {
                    if place.0 > current_line {
                        print!("{:>03} ", current_line + 1);
                        current_line = place.0;
                    } else {
                        print!(" |  ")
                    }
                }
                _ => print!("??? "),
            }

            // Prints the instruction name
            match code {
                Some(instr) => {
                    // Prints the instruction with a teal color
                    print!("\x1b[32m{:#04X}\x1b[0m – \x1b[36m{:?}\x1b[0m ", instr.clone() as u8, instr);

                    // Reads two bytes as the index of a constant
                    let mut const_val = |is_long: bool| -> &Object {
                        i += 1;

                        let pos;
                        if is_long {
                            pos = match self.codes.get_short(i) {
                                Some(short) => short as usize,
                                None => unreachable!("Could not get short."),
                            };
                            i += 1; // increment `i` again for the second byte in the short
                        } else {
                            pos = match self.codes.get_byte(i) {
                                Some(byte) => byte as usize,
                                None => unreachable!("Could not get byte."),
                            };
                        }

                        self.get_constant(pos).unwrap()
                    };

                    match instr {
                        // Prints the value associated with an OP_CONSTANT instruction
                        OpCode::OP_LOAD_CONST => println!("\t\t---> {}", const_val(false)),
                        OpCode::OP_LOAD_CONST_LONG => println!("\t\t---> {}", const_val(true)),
                        OpCode::OP_GET_VAR
                        | OpCode::OP_SET_VAR
                        | OpCode::OP_ARRAY
                        | OpCode::OP_POST_INCREMENT
                        | OpCode::OP_POST_DECREMENT
                        | OpCode::OP_LOAD_IMM => {
                            i += 1;
                            println!("\t{}", self.codes.get_byte(i).unwrap());
                        }

                        OpCode::OP_LOOP_JUMP => {
                            i += 1;
                            println!("\t{}", (i + 1) - (self.codes.get_byte(i).unwrap() as usize));
                        }

                        OpCode::OP_GET_VAR_LONG
                        | OpCode::OP_SET_VAR_LONG
                        | OpCode::OP_LOOP_JUMP_LONG
                        | OpCode::OP_ARRAY_LONG
                        | OpCode::OP_POST_INCREMENT_LONG
                        | OpCode::OP_POST_DECREMENT_LONG
                        | OpCode::OP_LOAD_IMM_LONG => {
                            i += 1;
                            println!("\t{}", self.codes.get_short(i).unwrap());
                            i += 1;
                        }

                        OpCode::OP_JUMP | OpCode::OP_JUMP_IF_FALSE => {
                            i += 1;
                            println!("\t{}", (self.codes.get_short(i).unwrap() as usize) + i + 2);
                            i += 1;
                        }

                        // If the instruction does not use the next to bytes, then print nothing
                        _ => println!(),
                    }
                }
                None => println!("No Instruction Found..."),
            }
            i += 1;
        }
    }

    /// Disassembles the chunk into its raw bytes, and prints the each instruction byte.
    /// This is useful when comparing the chunk generated by a program vs another program.
    ///
    /// ## Arguments
    /// * `name` – the name to print for the current chunk
    pub fn print_raw(&mut self, name: &str) {
        let mut i = 0;

        // prints this chunk's name
        println!("==== {} ====", name);

        while i < self.codes.len() {
            match self.codes.get_byte(i) {
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
}
