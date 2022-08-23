use analyzers::symbols::{SymbolTable, SymbolTableArena};
use core::ast::*;
use core::bytecode::OpCode;
use core::chunk::Chunk;
use core::errors::{ErrMsg, ErrorReport};
use core::objects::{func_obj, Object};
use core::tokens::{TokenIdx, TokenList};
use core::values::Value;
use parser::Parser;

mod visitor;

pub struct Compiler<'a> {
  tokens: &'a TokenList<'a>,
  ast: &'a ASTArena,
  symbol_tables: &'a [SymbolTable],
  current_table: usize,
  constants: Vec<Value>,
  current_fn: usize,
  errors: Vec<ErrorReport>,
}

impl<'a> Compiler<'a> {
  pub fn new(tokens: &'a TokenList, ast: &'a ASTArena, symbols: &'a [SymbolTable]) -> Self {
    let main_fn = func_obj::FuncObj {
      defaults: vec![],
      min_arity: 0,
      max_arity: 0,
      chunk: Default::default(),
      name: 0, // See TokenKind::THIS_FILE for more info.
      up_val_count: 0,
    };

    Compiler {
      tokens,
      ast,
      symbol_tables: symbols,
      current_table: 0,
      constants: vec![main_fn.into()],
      current_fn: 0,
      errors: vec![],
    }
  }

  pub fn compile(tokens: &TokenList) -> Result<Vec<Value>, Vec<ErrorReport>> {
    let ast = Parser::parse(tokens)?;
    let symbols = SymbolTableArena::tables_from(tokens, &ast)?;
    let compiler = Compiler::new(tokens, &ast, &symbols);
    Compiler::compile_from(compiler)
  }

  pub fn compile_from(mut compiler: Compiler) -> Result<Vec<Value>, Vec<ErrorReport>> {
    // Traverse the tree and compile the source.
    compiler.ast_visit_node(0, ());

    if compiler.errors.is_empty() {
      Ok(compiler.constants)
    } else {
      Err(compiler.errors)
    }
  }

  fn get_current_table(&self) -> &SymbolTable {
    &self.symbol_tables[self.current_table]
  }

  fn get_current_fn_mut(&mut self) -> &mut func_obj::FuncObj {
    match &mut self.constants[self.current_fn] {
      Value::Obj(Object::Func(f)) => f,
      _ => unreachable!("`self.current_fn` should point to function object."),
    }
  }

  fn current_chunk_mut(&mut self) -> &mut Chunk {
    &mut self.get_current_fn_mut().chunk
  }

  /// Emits a byte instruction from an OpCode into the chunk's instruction list.
  ///
  /// # Parameters
  /// - `instr`: The OpCode instruction to be added to the chunk.
  /// - `tok`: The token associated with the instruction.
  fn emit_op_code(&mut self, instr: OpCode, tok: TokenIdx) {
    self.current_chunk_mut().push_op_code(instr);
    self.current_chunk_mut().push_tok(tok);
  }

  /// Emits a raw byte instruction into the chunk's instruction list.
  ///
  /// # Parameters
  /// - `byte`: The byte instruction to be added to the chunk.
  /// - `tok`: The token associated with the instruction.
  fn emit_raw_byte(&mut self, byte: u8, tok: TokenIdx) {
    self.current_chunk_mut().push_byte(byte);
    self.current_chunk_mut().push_tok(tok);
  }

  /// Emits a raw short instruction from a 16-bit integer into the chunk's instruction list.
  ///
  /// # Parameters
  /// - `instr`: The 16-bit short instruction to add to the chunk.
  /// - `tok`: The token associated with the instruction.
  fn emit_raw_short(&mut self, short: u16, tok: TokenIdx) {
    self.current_chunk_mut().push_short(short);
    self.current_chunk_mut().push_tok(tok);
    self.current_chunk_mut().push_tok(tok);
  }

  /// Emits a byte instruction from an OpCode into the chunk's instruction list, followed by a raw byte.
  ///
  /// # Parameters
  /// - `instr`: The OpCode instruction to be added to the chunk.
  /// - `byte`: The raw byte to follow the emitted instruction.
  /// - `tok`: The token associated with the instruction.
  fn emit_op_code_with_byte(&mut self, instr: OpCode, byte: u8, tok: TokenIdx) {
    self.emit_op_code(instr, tok);
    self.emit_raw_byte(byte, tok);
  }

  /// Emits a byte instruction from an OpCode into the chunk's instruction list, followed by two more
  /// raw bytes from a 16-bit integer.
  ///
  /// # Parameters
  /// - `instr`: The OpCode instruction to be added to the chunk.
  /// - `short`: The 16-bit short instruction to add to the chunk.
  /// - `tok`: The token associated with the instruction.
  fn emit_op_code_with_short(&mut self, instr: OpCode, short: u16, tok: TokenIdx) {
    self.emit_op_code(instr, tok);
    self.emit_raw_short(short, tok);
  }

  /// Emits a byte instruction from an OpCode into the chunk's instruction list, followed by one or two bytes from
  /// a usize integer.
  /// NOTE: This will not check that the operand is less than or equal to u16::MAX.
  ///
  /// # Parameters
  /// - `instr1`: The OpCode short instruction to be added to the chunk if `operand < 256`.
  /// - `instr2`: The OpCode short instruction to be added to the chunk if `operand >= 256`.
  /// - `operand`: The usize instruction to be converted to a u8 or u16 before it is added to the chunk.
  /// - `tok`: The token associated with the instruction.
  fn emit_op_code_with_usize(&mut self, instr1: OpCode, instr2: OpCode, operand: usize, tok: TokenIdx) {
    if operand < 256 {
      self.emit_op_code_with_byte(instr1, operand as u8, tok);
    } else {
      self.emit_op_code_with_short(instr2, operand as u16, tok);
    }
  }

  pub fn emit_const(&mut self, val: Value, token: TokenIdx, load: bool) {
    // Try not to create unnecessary constant duplication.
    // TODO: Does dereferencing `x` duplicate its vale?
    let constant_pos = match self.constants.iter().position(|x| *x == val) {
      Some(pos) => pos,
      None => {
        self.constants.push(val);
        self.constants.len() - 1
      }
    };

    if constant_pos <= u16::MAX as usize {
      if load {
        self.emit_op_code_with_usize(OpCode::LoadConstant, OpCode::LoadConstantLong, constant_pos, token);
      }
    } else {
      let err_msg = ErrMsg::MaxCapacity("Too many constants in one chunk.".to_string());
      self.emit_error(token, err_msg, None);
    }
  }

  fn emit_error(&mut self, token: TokenIdx, err_msg: ErrMsg, hint: Option<String>) {
    self.errors.push(ErrorReport { token, err_msg, hint });
  }
}
