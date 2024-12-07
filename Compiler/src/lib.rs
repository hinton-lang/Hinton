use analyzers::symbols::{SymbolTable, SymbolTableArena};
use core::ast::*;
use core::bytecode::OpCode;
use core::chunk::Chunk;
use core::errors::{ErrMsg, ErrorReport};
use core::tokens::{TokenIdx, TokenList};
use objects::gc::{GarbageCollector, GcId, GcObject, GcObjectKind};
use objects::str_obj::StrObj;
use objects::{func_obj, Object};
use parser::Parser;

mod visitor;

/// If the compiler is currently inside of a loop, this represents
/// where in the chunk of code the loop started.
#[derive(Copy, Clone)]
pub struct LoopScope {
  /// The chunk position of the loop.
  loc: usize,
  /// The number of declarations currently made inside the loop. This
  /// is required by break and continue statements since they must pop
  /// local declarations off the stack before the loop's block ends.
  decls_count: u16,
  /// Whether or not the loop's `decls_count` field can be updated.
  /// This is used to lock the current loop when compiling a function
  /// inside the loop, since the declarations made inside the function
  /// do not belong on the loop's lexical scope.
  can_update: bool,
}

impl LoopScope {
  /// Generates a new encoding for a loop's lexical scope.
  ///
  /// # Arguments
  ///
  /// * `loc`: The position of the loop's start, including the loop's
  /// condition, on the chunk of bytecode.
  pub fn new(loc: usize) -> Self {
    LoopScope { loc, decls_count: 0, can_update: true }
  }
}

/// Represents the location of a break declaration in the chunk.
/// Used to patch the jump at the end of it's parent loop's block.
#[derive(Copy, Clone)]
pub struct BreakScope {
  parent_loop: usize,
  chunk_pos: usize,
}

/// The compiler. Each field holds the information needed to emit the
/// the correct bytecode associated with each node in the AST.
pub struct Compiler<'a> {
  tokens: &'a TokenList<'a>,
  ast: &'a ASTArena,
  symbol_tables: &'a [SymbolTable],
  current_table: usize,
  constants: Vec<Object>,
  gc_objs: GarbageCollector,
  current_fn: GcId,
  local_decl_count: u16,
  loop_scopes: Vec<LoopScope>,
  break_scopes: Vec<BreakScope>,
  errors: Vec<ErrorReport>,
}

impl<'a> Compiler<'a> {
  /// Generates a new compiler.
  ///
  /// # Arguments
  ///
  /// * `tokens`: The list of tokens.
  /// * `ast`: The abstract syntax tree.
  /// * `symbols`: The symbol table.
  pub fn new(tokens: &'a TokenList, ast: &'a ASTArena, symbols: &'a [SymbolTable]) -> Self {
    let mut gc = GarbageCollector::default();
    let file_path = tokens.filepath.to_string_lossy().into_owned();

    let main_fn = func_obj::FuncObj {
      defaults: vec![],
      min_arity: 0,
      max_arity: Some(0),
      chunk: Default::default(),
      name: gc.push(GcObject::Str(StrObj(file_path))),
    };

    let main_fn = gc.push(main_fn.into());

    Compiler {
      tokens,
      ast,
      symbol_tables: symbols,
      current_table: 0,
      constants: vec![Object::Func(main_fn)],
      gc_objs: gc,
      current_fn: main_fn,
      local_decl_count: 0,
      loop_scopes: vec![],
      break_scopes: vec![],
      errors: vec![],
    }
  }

  /// Compiles the given token list to a series of function objects
  /// that represent the program.
  ///
  /// # Arguments
  ///
  /// * `tokens`: The list of tokens.
  ///
  /// # Returns:
  /// ```Result<(GarbageCollector, Vec<Object>, GcId), Vec<ErrorReport>>```
  pub fn compile(tokens: &TokenList) -> Result<(GarbageCollector, Vec<Object>, GcId), Vec<ErrorReport>> {
    let ast = Parser::parse(tokens)?;
    let symbols = SymbolTableArena::tables_from(tokens, &ast)?;
    let compiler = Compiler::new(tokens, &ast, &symbols);
    Compiler::compile_from(compiler)
  }

  /// Executes the compiler.
  ///
  /// # Arguments
  ///
  /// * `compiler`: The compiler instance to execute.
  ///
  /// # Returns:
  /// ```Result<(GarbageCollector, Vec<Object>, GcId), Vec<ErrorReport>>```
  pub fn compile_from(mut compiler: Compiler) -> Result<(GarbageCollector, Vec<Object>, GcId), Vec<ErrorReport>> {
    // Traverse the tree and compile the source.
    compiler.ast_visit_node(0, ());

    // Emit the "EndVM" Instruction at EOF.
    compiler.emit_op(OpCode::EndVirtualMachine, compiler.tokens.tokens.len() - 1);

    if compiler.errors.is_empty() {
      Ok((compiler.gc_objs, compiler.constants, compiler.current_fn))
    } else {
      Err(compiler.errors)
    }
  }

  /// Gets an immutable reference to the symbol table
  /// associated with the function currently being compiled.
  fn get_current_table(&self) -> &SymbolTable {
    &self.symbol_tables[self.current_table]
  }

  /// Gets a mutable reference to the function object currently being compiled.
  fn get_current_fn_mut(&mut self) -> &mut func_obj::FuncObj {
    self.gc_objs.get_mut(&self.current_fn).as_func_obj_mut().unwrap()
  }

  /// Gets a mutable reference ot the chunk currently being compiled to.
  fn current_chunk_mut(&mut self) -> &mut Chunk {
    &mut self.get_current_fn_mut().chunk
  }

  /// Emits a byte instruction from an OpCode into the chunk's instruction list.
  ///
  /// # Parameters
  /// - `instr`: The OpCode instruction to be added to the chunk.
  /// - `tok`: The token associated with the instruction.
  fn emit_op(&mut self, instr: OpCode, tok: TokenIdx) {
    self.current_chunk_mut().push_byte(instr as u8);
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
  fn emit_op_with_byte(&mut self, instr: OpCode, byte: u8, tok: TokenIdx) {
    self.emit_op(instr, tok);
    self.emit_raw_byte(byte, tok);
  }

  /// Emits a byte instruction from an OpCode into the chunk's instruction list, followed by two more
  /// raw bytes from a 16-bit integer.
  ///
  /// # Parameters
  /// - `instr`: The OpCode instruction to be added to the chunk.
  /// - `short`: The 16-bit short instruction to add to the chunk.
  /// - `tok`: The token associated with the instruction.
  fn emit_op_with_short(&mut self, instr: OpCode, short: u16, tok: TokenIdx) {
    self.emit_op(instr, tok);
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
  fn emit_op_with_usize(&mut self, instr1: OpCode, instr2: OpCode, operand: usize, tok: TokenIdx) {
    if operand < 256 {
      self.emit_op_with_byte(instr1, operand as u8, tok);
    } else {
      self.emit_op_with_short(instr2, operand as u16, tok);
    }
  }

  /// Emits a garbage collected object into the garbage collector's memory manager, as
  /// well as a reference ot eh GcObject into the constants table, and returns the position
  /// of the object in the constants table.
  ///
  /// # Arguments
  ///
  /// * `gc_obj`: The object to emit.
  /// * `token`: The token associated with the object.
  /// * `load`: Whether or not to emit an instruction to load the object onto the stack.
  ///
  /// # Returns:
  /// ```usize```
  pub fn emit_const_gc_obj(&mut self, gc_obj: GcObject, token: TokenIdx, load: bool) -> usize {
    let obj_kind = gc_obj.kind();
    let id = self.gc_objs.push(gc_obj);

    let obj = match obj_kind {
      GcObjectKind::Str => Object::Str(id),
      GcObjectKind::Array => Object::Array(id),
      GcObjectKind::Tuple => Object::Tuple(id),
      GcObjectKind::Range => Object::Range(id),
      GcObjectKind::Func => Object::Func(id),
    };

    self.emit_const(obj, token, load)
  }

  /// Emits an object into the constants table, and returns its position.
  ///
  /// # Arguments
  ///
  /// * `gc_obj`: The object to emit.
  /// * `token`: The token associated with the object.
  /// * `load`: Whether or not to emit an instruction to load the object onto the stack.
  ///
  /// # Returns:
  /// ```usize```
  pub fn emit_const(&mut self, val: Object, token: TokenIdx, load: bool) -> usize {
    // Try not to create unnecessary constant duplication.
    let constant_pos = match self.constants.iter().position(|x| x.equals(&val, &self.gc_objs)) {
      Some(pos) => pos,
      None => {
        self.constants.push(val);
        self.constants.len() - 1
      }
    };

    if constant_pos <= u16::MAX as usize {
      if load {
        self.emit_op_with_usize(OpCode::LoadConstant, OpCode::LoadConstantLong, constant_pos, token);
      }
    } else {
      let err_msg = ErrMsg::MaxCapacity("Too many constants in one chunk.".to_string());
      self.emit_error(token, err_msg, None);
    }

    constant_pos
  }

  /// Emits a jump instructions with a dummy jump offset. This offset should be
  /// later replaced by calling the `patch_jump(...)` function.
  ///
  /// # Parameters
  /// - `instruction`: The jump instruction to emit to the chunk.
  /// - `token`: The TokenIdx associated with this jump.
  ///
  /// # Returns
  /// `usize`: The position of the currently emitted jump instruction in the chunk. This value
  /// should be used by the call to the `patch_jump(...)` function to patch the correct jump
  /// instruction's offset.
  fn emit_jump(&mut self, instruction: OpCode, token: TokenIdx) -> usize {
    self.emit_op_with_short(instruction, 0xffff, token);
    return self.current_chunk_mut().len() - 2;
  }

  /// Patches the offset of a jump instruction.
  ///
  /// # Parameters
  /// - `offset`: The position in the chunk of the jump instruction to be patched.
  /// - `token`: The token associated with this jump patch.
  fn patch_jump(&mut self, offset: usize, token: TokenIdx) {
    // -2 to adjust for the bytecode of the jump offset itself.
    match u16::try_from(self.current_chunk_mut().len() - offset - 2) {
      Ok(forward) => {
        let jump = forward.to_be_bytes();
        self.current_chunk_mut().patch(offset, jump[0]);
        self.current_chunk_mut().patch(offset + 1, jump[1]);
      }
      Err(_) => self.emit_error(token, ErrMsg::MaxCapacity("Too much code to jump over.".into()), None),
    }
  }

  /// Emits a `LoopJump` instruction.
  ///
  /// # Parameters
  /// - `loop_start`: The position in the chunk of the loop's first instruction.
  /// - `token`: The token associated with this loop instruction.
  fn emit_loop(&mut self, loop_start: usize, token: TokenIdx) {
    // +3 to adjust for the bytecode of the loop jump instruction and offset.
    match u16::try_from(self.current_chunk_mut().len() - loop_start + 3) {
      Ok(backward) => {
        self.emit_op(OpCode::LoopJump, token);
        self.emit_raw_short(backward, token);
      }
      Err(_) => self.emit_error(token, ErrMsg::MaxCapacity("Loop body too large.".into()), None),
    }
  }

  /// Emits an error to the errors list.
  fn emit_error(&mut self, token: TokenIdx, err_msg: ErrMsg, hint: Option<String>) {
    self.errors.push(ErrorReport { token, err_msg, hint });
  }
}
