use core::ast::*;
use core::errors::ErrorReport;
use core::tokens::{TokenIdx, TokenList};

pub mod resolver;
pub mod visitor;

/// The kind of declaration this symbol represents
#[derive(Copy, Clone)]
pub enum SymbolKind {
  Var,
  Const,
  Func,
  Class,
  Method,
  Param,
}

/// The location of a declaration, whether in the globals or the stack.
#[derive(Copy, Clone)]
pub enum SymbolLoc {
  Global(usize),
  Stack(usize),
}

/// The location of a symbol resolution.
#[derive(Copy, Clone)]
pub enum SymRes {
  None,
  Stack(usize),
  UpVal(usize),
  Global(usize),
  Native(usize),
  Primitive(usize),
}

/// The symbol's scope id and depth.
#[derive(Copy, Clone, Default)]
pub struct SymbolScopeData {
  pub id: usize,
  pub depth: u16,
}

/// Create a new SymbolScopeData
/// [`scope_id`, `scope_depth`]
#[macro_export]
macro_rules! ssd {
  ($id:expr,$d:expr) => {{
    SymbolScopeData { id: $id, depth: $d }
  }};
}

/// A symbol found in a Hinton program.
/// Symbols in Hinton normally have an associated identifier.
#[derive(Copy, Clone)]
pub struct Symbol {
  pub token_idx: TokenIdx,
  pub kind: SymbolKind,
  pub scope: SymbolScopeData,
  pub has_reference: bool,
  pub loc: SymbolLoc,
  pub is_out_of_scope: bool,
}

/// Represents the index of a Symbol Table in the SymbolTableArena.
pub type SymbolTableIdx = usize;

/// A collection SymbolTables.
/// Since SymbolTables can nest inside one another, we represent
/// their structure with an Arena data structure.
pub struct SymbolTableArena<'a> {
  ast: &'a ASTArena,
  tokens: &'a TokenList<'a>,
  arena: Vec<SymbolTable>,
  current_table: SymbolTableIdx,
  globals_len: usize,
  errors: Vec<ErrorReport>,
  up_values_len: usize,
}

impl<'a> SymbolTableArena<'a> {
  /// Generates a collection of SymbolTables from the given token list and ast.
  ///
  /// # Arguments
  ///
  /// * `tokens`: The lexed tokens list.
  /// * `ast`: The parsed abstract syntax tree.
  ///
  /// # Returns:
  /// ```Result<Vec<SymbolTable, Global>, Vec<ErrorReport, Global>>```
  pub fn tables_from(tokens: &'a TokenList, ast: &'a ASTArena) -> Result<Vec<SymbolTable>, Vec<ErrorReport>> {
    let mut tables = SymbolTableArena {
      arena: vec![SymbolTable::new(None, false, false)],
      ast,
      tokens,
      errors: vec![],
      current_table: 0,
      globals_len: 0,
      up_values_len: 0,
    };

    // Recursively visit all nodes in the AST arena.
    tables.ast_visit_node(0, SymbolScopeData::default());

    if tables.errors.is_empty() {
      Ok(tables.arena)
    } else {
      Err(tables.errors)
    }
  }

  /// Gets an immutable reference to the current symbol table being generated.
  fn get_current_table(&self) -> &SymbolTable {
    &self.arena[self.current_table]
  }

  /// Gets mutable reference to the current symbol table being generated.
  fn get_current_table_mut(&mut self) -> &mut SymbolTable {
    &mut self.arena[self.current_table]
  }
}

/// Represents the index of a Symbol in a SymbolTable.
pub type SymbolIdx = usize;

/// Encodes the kind of loop, if any, currently being analyzed.
#[derive(Copy, Clone)]
pub enum TableLoopState {
  None,
  Loop,
  For,
  While,
}

/// A collection of symbols found inside a function.
/// Encodes the lexical scoping of symbols as they are found throughout a program.
pub struct SymbolTable {
  symbols: Vec<Symbol>,
  resolved: Vec<(TokenIdx, SymRes)>,
  parent_table: Option<SymbolTableIdx>,
  max_scope_id: usize,
  stack_len: usize,
  is_func_ctx: bool,
  is_class_ctx: bool,
  loop_ctx: TableLoopState,
}

impl SymbolTable {
  /// Generates a new symbol table given the specified arguments.
  ///
  /// # Arguments
  ///
  /// * `parent_table`: The position, if any, of the parent symbol table.
  /// * `parent_scope`: The scope_id where the function was declared.
  /// * `is_func_ctx`: Whether or not the current symbol table represents a function context.
  /// * `is_class_ctx`: Whether or not the function is declared in the context of a class.
  ///
  /// # Returns:
  /// ```SymbolTable```
  pub fn new(parent_table: Option<SymbolTableIdx>, is_func_ctx: bool, is_class_ctx: bool) -> SymbolTable {
    SymbolTable {
      symbols: vec![],
      resolved: vec![],
      parent_table,
      max_scope_id: 0,
      stack_len: 1,
      is_func_ctx,
      is_class_ctx,
      loop_ctx: TableLoopState::None,
    }
  }

  /// Pushes a new symbol to the symbol table.
  ///
  /// # Arguments
  ///
  /// * `symbol`: The symbol to be pushed into the symbol table.
  ///
  /// # Returns:
  /// ```usize```
  pub fn push(&mut self, symbol: Symbol) -> SymbolIdx {
    self.symbols.push(symbol);
    self.symbols.len() - 1
  }

  /// Gets an immutable reference to the symbol at the given SymbolIdx.
  ///
  /// # Arguments
  ///
  /// * `idx`: The SymbolIdx of the Symbol.
  ///
  /// # Returns:
  /// ```&Symbol```
  pub fn get(&self, idx: SymbolIdx) -> &Symbol {
    &self.symbols[idx]
  }
}
