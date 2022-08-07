use crate::tokens::{TokenIdx, TokenKind};

/// Represents the index of an AST Node in the ASTArena.
#[derive(PartialEq)]
pub struct ASTNodeIdx(pub usize);

impl From<usize> for ASTNodeIdx {
  fn from(x: usize) -> Self {
    ASTNodeIdx(x)
  }
}

impl Default for ASTNodeIdx {
  fn default() -> Self {
    usize::MAX.into()
  }
}

/// Represents the index of an Class Declaration Node
/// in the `classes` section of the ASTArena.
#[derive(PartialEq)]
pub struct ASTClassIdx(pub usize);

impl From<usize> for ASTClassIdx {
  fn from(x: usize) -> Self {
    ASTClassIdx(x)
  }
}

impl Default for ASTClassIdx {
  fn default() -> Self {
    usize::MAX.into()
  }
}

/// Abstract syntax tree in the form
/// of an Arena data structure.
pub struct ASTArena {
  arena: Vec<ASTNodeKind>,
  /// Because class nodes are very heavy and used rarely,
  /// we store them in a separate section so that the
  ///`ASTNodeKind` enum is not penalized by their weight.
  classes: Vec<ASTClassDeclNode>,
}

impl Default for ASTArena {
  /// The default AST Arena, which comes with an empty module node as root.
  fn default() -> Self {
    let root = ASTNodeKind::Module(ASTModuleNode {
      children: vec![],
      public_members: vec![],
    });

    Self {
      arena: vec![root],
      classes: vec![],
    }
  }
}

impl ASTArena {
  /// Pushes a new node to the arena.
  ///
  /// # Arguments
  ///
  /// * `val`: The node to insert into the arena.
  ///
  /// # Returns:
  /// ```ASTNodeIdx```
  pub fn push(&mut self, val: ASTNodeKind) -> ASTNodeIdx {
    self.arena.push(val);
    (self.arena.len() - 1).into()
  }

  /// Pushes a new class node to the `class` section of the arena.
  ///
  /// # Arguments
  ///
  /// * `val`: The node to insert into the arena.
  ///
  /// # Returns:
  /// ```ASTNodeIdx```
  pub fn push_class(&mut self, val: ASTClassDeclNode) -> ASTClassIdx {
    self.classes.push(val);
    (self.classes.len() - 1).into()
  }

  /// Gets an ASTNode in the arena from its ASTNodeIdx. Can also
  /// use a `usize` and convert it to an `ASTNodeIdx` with `.into()`.
  ///
  /// # Arguments
  ///
  /// * `idx`: The ASTNodeIdx of the node.
  ///
  /// # Returns:
  /// ```&ASTArenaNode```
  pub fn get(&self, idx: &ASTNodeIdx) -> &ASTNodeKind {
    &self.arena[idx.0]
  }

  /// Gets an Class declaration node from the class section of the arena
  /// from its ASTClassIdx. Can also use a `usize` and convert it to
  /// an `ASTClassIdx` with `.into()`.
  ///
  /// # Arguments
  ///
  /// * `idx`: The ASTClassIdx of the node.
  ///
  /// # Returns:
  /// ```&ASTClassDeclNode```
  pub fn get_class(&self, idx: &ASTClassIdx) -> &ASTClassDeclNode {
    &self.classes[idx.0]
  }

  /// Attaches a node to the root module node.
  ///
  /// # Arguments
  ///
  /// * `child`: The node to be attached to the root module node.
  ///
  /// # Returns:
  /// ```()```
  pub fn attach_to_root(&mut self, child: ASTNodeIdx) {
    match &mut self.arena[0] {
      ASTNodeKind::Module(m) => m.children.push(child),
      _ => unreachable!("Root node should be a module node."),
    }
  }

  /// Adds the name of a public member declaration to the root node.
  ///
  /// # Arguments
  ///
  /// * `child`: The ASTNodeIdx of the public member declaration.
  ///
  ///  # Returns:
  /// ```()```
  pub fn add_pub_to_root(&mut self, child: ASTNodeIdx) {
    match &mut self.arena[0] {
      ASTNodeKind::Module(m) => m.public_members.push(child),
      _ => unreachable!("Root node should be a module node."),
    }
  }
}

pub enum ASTNodeKind {
  Module(ASTModuleNode),

  ArrayLiteral(Vec<ASTNodeIdx>),
  ArraySlice(ASTArraySliceNode),
  BinaryExpr(ASTBinaryExprNode),
  BlockStmt(Vec<ASTNodeIdx>),
  BreakStmt(Option<ASTNodeIdx>),
  CallExpr(ASTCallExprNode),
  ClassDecl(ASTClassIdx),
  CompactArrOrTpl(ASTCompactArrOrTplNode),
  CompactDict(ASTCompactDictNode),
  ContinueStmt,
  DelStmt(ASTNodeIdx),
  DestructingPattern(ASTDestructingPatternNode),
  DestructingWildCard(Option<ASTNodeIdx>),
  DictKeyValPair((ASTNodeIdx, ASTNodeIdx)),
  DictLiteral(Vec<ASTNodeIdx>),
  EvaluatedDictKey(ASTNodeIdx),
  ExportDecl(ASTImportExportNode),
  ExprStmt(ASTNodeIdx),
  FalseLiteral(TokenIdx),
  ForLoop(ASTForLoopNode),
  FuncDecl(ASTFuncDeclNode),
  Identifier(TokenIdx),
  IfStmt(ASTIfStmtNode),
  ImportDecl(ASTImportExportNode),
  Indexing(ASTIndexingNode),
  Lambda(ASTLambdaNode),
  LoopExpr(ASTNodeIdx), // This will always be a block stmt
  MemberAccess(ASTMemberAccessNode),
  NoneLiteral(TokenIdx),
  NumLiteral(TokenIdx),
  Reassignment(ASTReassignmentNode),
  RepeatLiteral(ASTRepeatLiteralNode),
  ReturnStmt(ASTNodeIdx),
  SelfLiteral(TokenIdx),
  SpreadExpr(ASTNodeIdx),
  StringLiteral(TokenIdx),
  SuperLiteral(TokenIdx),
  TernaryConditional(ASTTernaryConditionalNode),
  ThrowStmt(ASTNodeIdx),
  TrueLiteral(TokenIdx),
  TryCatchFinally(ASTTryCatchFinallyNode),
  TupleLiteral(Vec<ASTNodeIdx>),
  UnaryExpr(ASTUnaryExprNode),
  VarConstDecl(ASTVarConsDeclNode),
  WhileLoop(ASTWhileLoopNode),
  WithStmt(ASTWithStmtNode),
  YieldStmt(ASTNodeIdx),
}

/// An AST Module node.
pub struct ASTModuleNode {
  pub children: Vec<ASTNodeIdx>,
  // We store the ASTNodeIdx of public declarations so
  // we can obtain their names during compile time.
  pub public_members: Vec<ASTNodeIdx>,
}

/// An AST Reassignment Node
pub struct ASTReassignmentNode {
  pub target: ASTNodeIdx,
  pub kind: ASTReassignmentKind,
  pub value: ASTNodeIdx,
}

#[derive(Debug)]
#[repr(u8)]
pub enum ASTReassignmentKind {
  Assign,   // a = b
  BitAnd,   // a &= b
  BitOr,    // a |= b
  Div,      // a /= b
  Expo,     // a **= b
  LogicAnd, // a &&= b
  LogicOr,  // a ||= b
  MatMul,   // a @= b
  Minus,    // a -= b
  Mod,      // a %= b
  Mul,      // a *= b
  Nonish,   // a ??= b
  Plus,     // a += b
  ShiftL,   // a <<= b
  ShiftR,   // a >>= b
  Xor,      // a ^= b
}

impl ASTReassignmentKind {
  /// Tries to create a ReassignmentKind from a TokenKind.
  ///
  /// # Arguments
  ///
  /// * `tk`: The TokenKind to be converted.
  ///
  /// # Returns:
  /// ```Option<ASTReassignmentKind>```
  pub fn try_from_token(tk: &TokenKind) -> Option<Self> {
    match tk {
      TokenKind::AT_EQ => Some(ASTReassignmentKind::MatMul),
      TokenKind::BIT_AND_EQ => Some(ASTReassignmentKind::BitAnd),
      TokenKind::BIT_L_SHIFT_EQ => Some(ASTReassignmentKind::ShiftL),
      TokenKind::BIT_OR_EQ => Some(ASTReassignmentKind::BitOr),
      TokenKind::BIT_R_SHIFT_EQ => Some(ASTReassignmentKind::ShiftR),
      TokenKind::BIT_XOR_EQ => Some(ASTReassignmentKind::Xor),
      TokenKind::EQUALS => Some(ASTReassignmentKind::Assign),
      TokenKind::POW_EQUALS => Some(ASTReassignmentKind::Expo),
      TokenKind::LOGIC_AND_EQ => Some(ASTReassignmentKind::LogicAnd),
      TokenKind::LOGIC_OR_EQ => Some(ASTReassignmentKind::LogicOr),
      TokenKind::MINUS_EQ => Some(ASTReassignmentKind::Minus),
      TokenKind::MOD_EQ => Some(ASTReassignmentKind::Mod),
      TokenKind::NONISH_EQ => Some(ASTReassignmentKind::Nonish),
      TokenKind::PLUS_EQ => Some(ASTReassignmentKind::Plus),
      TokenKind::SLASH_EQ => Some(ASTReassignmentKind::Div),
      TokenKind::STAR_EQ => Some(ASTReassignmentKind::Mul),
      _ => None,
    }
  }
}

/// An AST Ternary Conditional Node
pub struct ASTTernaryConditionalNode {
  pub condition: ASTNodeIdx,
  pub branch_true: ASTNodeIdx,
  pub branch_false: ASTNodeIdx,
}

/// An AST Binary Expression Node
pub struct ASTBinaryExprNode {
  pub left: ASTNodeIdx,
  pub right: ASTNodeIdx,
  pub kind: BinaryExprKind,
}

#[derive(Clone, Debug)]
#[repr(u8)]
pub enum BinaryExprKind {
  Add,           // +
  BitAND,        // &
  BitOR,         // |
  BitShiftLeft,  // <<
  BitShiftRight, // >>
  BitXOR,        // ^
  Div,           // /
  Equals,        // =
  Pow,           // **
  GreaterThan,   // >
  GreaterThanEQ, // >=
  In,            // in
  InstOf,        // instof
  LessThan,      // <
  LessThanEQ,    // <=
  LogicAND,      // &&, and
  LogicOR,       // ||, or
  MatMult,       // @
  Mod,           // %, mod
  Mult,          // *
  Nonish,        // ??
  NotEquals,     // !=
  Pipe,          // |>
  Range,         // ..
  RangeEQ,       // ..=
  Subtract,      // -
}

impl BinaryExprKind {
  /// Tries to create a Binary Equality Operator from a TokenKind.
  ///
  /// # Arguments
  ///
  /// * `tk`: The TokenKind to be converted.
  ///
  /// # Returns:
  /// ```Option<BinaryExprKind>```
  pub fn try_equality(tk: &TokenKind) -> Option<BinaryExprKind> {
    match tk {
      TokenKind::LOGIC_EQ => Some(BinaryExprKind::Equals),
      TokenKind::LOGIC_NOT_EQ => Some(BinaryExprKind::NotEquals),
      _ => None,
    }
  }

  /// Tries to create a Relation Operator from a TokenKind.
  ///
  /// # Arguments
  ///
  /// * `tk`: The TokenKind to be converted.
  ///
  /// # Returns:
  /// ```Option<BinaryExprKind>```
  pub fn try_relation(tk: &TokenKind) -> Option<BinaryExprKind> {
    match tk {
      TokenKind::GREATER_THAN => Some(BinaryExprKind::GreaterThan),
      TokenKind::GREATER_THAN_EQ => Some(BinaryExprKind::GreaterThanEQ),
      TokenKind::LESS_THAN => Some(BinaryExprKind::LessThan),
      TokenKind::LESS_THAN_EQ => Some(BinaryExprKind::LessThanEQ),
      TokenKind::IN_KW => Some(BinaryExprKind::In),
      TokenKind::INSTOF_KW => Some(BinaryExprKind::InstOf),
      _ => None,
    }
  }

  /// Tries to create a Bit Shift Operator from a TokenKind.
  ///
  /// # Arguments
  ///
  /// * `tk`: The TokenKind to be converted.
  ///
  /// # Returns:
  /// ```Option<BinaryExprKind>```
  pub fn try_bit_shift(tk: &TokenKind) -> Option<BinaryExprKind> {
    match tk {
      TokenKind::BIT_L_SHIFT => Some(BinaryExprKind::BitShiftLeft),
      TokenKind::BIT_R_SHIFT => Some(BinaryExprKind::BitShiftRight),
      _ => None,
    }
  }

  /// Tries to create a Range Operator from a TokenKind.
  ///
  /// # Arguments
  ///
  /// * `tk`: The TokenKind to be converted.
  ///
  /// # Returns:
  /// ```Option<BinaryExprKind>```
  pub fn try_range(tk: &TokenKind) -> Option<BinaryExprKind> {
    match tk {
      TokenKind::DOUBLE_DOT => Some(BinaryExprKind::Range),
      TokenKind::RANGE_EQ => Some(BinaryExprKind::RangeEQ),
      _ => None,
    }
  }

  /// Tries to create a Term Operator from a TokenKind.
  ///
  /// # Arguments
  ///
  /// * `tk`: The TokenKind to be converted.
  ///
  /// # Returns:
  /// ```Option<BinaryExprKind>```
  pub fn try_term(tk: &TokenKind) -> Option<BinaryExprKind> {
    match tk {
      TokenKind::PLUS => Some(BinaryExprKind::Add),
      TokenKind::DASH => Some(BinaryExprKind::Subtract),
      _ => None,
    }
  }

  /// Tries to create a Factor Operator from a TokenKind.
  ///
  /// # Arguments
  ///
  /// * `tk`: The TokenKind to be converted.
  ///
  /// # Returns:
  /// ```Option<BinaryExprKind>```
  pub fn try_factor(tk: &TokenKind) -> Option<BinaryExprKind> {
    match tk {
      TokenKind::AT => Some(BinaryExprKind::MatMult),
      TokenKind::MOD_KW => Some(BinaryExprKind::Mod),
      TokenKind::PERCENT => Some(BinaryExprKind::Mod),
      TokenKind::SLASH => Some(BinaryExprKind::Div),
      TokenKind::STAR => Some(BinaryExprKind::Mult),
      _ => None,
    }
  }
}

/// An AST Unary Expression Node
pub struct ASTUnaryExprNode {
  pub kind: UnaryExprKind,
  pub operand: ASTNodeIdx,
}

#[repr(u8)]
pub enum UnaryExprKind {
  LogicNot,
  Negate,
  BitNot,
  New,
  Typeof,
  Await,
}

impl UnaryExprKind {
  /// Tries to create a Unary Operator from a TokenKind.
  ///
  /// # Arguments
  ///
  /// * `tk`: The TokenKind to be converted.
  ///
  /// # Returns:
  /// ```Option<UnaryExprKind>```
  pub fn try_from_token(tk: &TokenKind) -> Option<UnaryExprKind> {
    match tk {
      TokenKind::BANG => Some(UnaryExprKind::LogicNot),
      TokenKind::DASH => Some(UnaryExprKind::Negate),
      TokenKind::BIT_NOT => Some(UnaryExprKind::BitNot),
      TokenKind::NEW_KW => Some(UnaryExprKind::New),
      TokenKind::TYPEOF_KW => Some(UnaryExprKind::Typeof),
      TokenKind::AWAIT_KW => Some(UnaryExprKind::Await),
      _ => None,
    }
  }
}

/// An AST Indexing Node
pub struct ASTIndexingNode {
  pub target: ASTNodeIdx,
  pub indexers: Vec<ASTNodeIdx>,
}

/// An AST Slice Node
pub struct ASTArraySliceNode {
  pub upper: Option<ASTNodeIdx>,
  pub lower: Option<ASTNodeIdx>,
}

/// An AST Member Access Node
pub struct ASTMemberAccessNode {
  pub is_safe: bool,
  pub target: ASTNodeIdx,
  pub member: ASTNodeIdx, // id node
}

/// An AST Call Expression Node
#[derive(Default)]
pub struct ASTCallExprNode {
  pub target: ASTNodeIdx,
  pub val_args: Vec<ASTNodeIdx>,
  pub rest_args: Vec<ASTNodeIdx>,
  pub named_args: Vec<(ASTNodeIdx, ASTNodeIdx)>, // (id node, value node)
}

/// An AST Lambda Expression Node
pub struct ASTLambdaNode {
  pub is_async: bool,
  pub params: Vec<SingleParam>,
  pub min_arity: u8,
  pub max_arity: u8,
  pub body: ASTNodeIdx, // This will always be a block stmt or single expression
}

/// An AST Repeat Literal Node
pub struct ASTRepeatLiteralNode {
  pub kind: RepeatLiteralKind,
  pub value: ASTNodeIdx,
  pub count: ASTNodeIdx,
}

#[derive(Debug)]
#[repr(u8)]
pub enum RepeatLiteralKind {
  Array,
  Tuple,
}

/// An AST While Loop Node
pub struct ASTWhileLoopNode {
  pub let_id: Option<ASTNodeIdx>,
  pub cond: ASTNodeIdx,
  pub body: ASTNodeIdx, // This will always be a block stmt
}

/// An AST For Loop Node
pub struct ASTForLoopNode {
  pub head: ForLoopHead,
  pub body: ASTNodeIdx, // This will always be a block stmt
}

/// An AST Compact Array/Tuple Node
pub struct ASTCompactArrOrTplNode {
  pub heads: Vec<CompactForLoop>,
  pub body: ASTNodeIdx,
  pub is_tpl: bool,
}

/// An AST Compact Dict Node
pub struct ASTCompactDictNode {
  pub heads: Vec<CompactForLoop>,
  pub body: ASTNodeIdx,
}

/// An AST Compact For Loop Node
pub struct CompactForLoop {
  pub head: ForLoopHead,
  pub cond: Option<ASTNodeIdx>, // The if-part, it it exists
}

/// The Head of a For-loop (both compact and statement)
pub struct ForLoopHead {
  pub id: ASTNodeIdx, // Can either be a single id node or destructuring pattern
  pub target: ASTNodeIdx,
}

/// An AST If-Else-If Statement Node
pub struct ASTIfStmtNode {
  pub cond: ASTNodeIdx,
  pub true_branch: ASTNodeIdx,
  pub else_branch: Option<ASTNodeIdx>,
}

/// An AST Variable/Constant declaration Node
pub struct ASTVarConsDeclNode {
  pub is_const: bool,
  pub id: ASTNodeIdx, // Can either be a single id node or destructuring pattern
  pub val: ASTNodeIdx,
}

/// An AST Destructing Pattern Node
pub struct ASTDestructingPatternNode {
  // List of identifier nodes, and optionally, at most one wild-card node.
  pub patterns: Vec<ASTNodeIdx>,
}

/// An AST With-As Statement Node
pub struct ASTWithStmtNode {
  pub heads: Vec<WithStmtHead>,
  pub body: ASTNodeIdx,
}

/// The head of a With-As Statement
pub struct WithStmtHead {
  pub expr: ASTNodeIdx,
  pub id: ASTNodeIdx,
}

/// An AST Try-Catch-Finally Statement Node
pub struct ASTTryCatchFinallyNode {
  pub body: ASTNodeIdx, // This will always be a block stmt
  pub catchers: Vec<CatchPart>,
  pub finally: Option<ASTNodeIdx>, // This will always be a block stmt
}

/// The "catch" Part of a Try-Catch-Finally Statement.
pub struct CatchPart {
  pub body: ASTNodeIdx, // This will always be a block stmt,
  pub target: Option<CatchTarget>,
}

/// The Target For the `catch` Part of a Try-Catch-Finally Statement.
pub struct CatchTarget {
  pub error_class: ASTNodeIdx,          // This will always be an identifier node
  pub error_result: Option<ASTNodeIdx>, // This will always be an identifier node
}

/// An AST Import/Export Declaration Node
pub struct ASTImportExportNode {
  pub members: Vec<ImportExportMember>,
  pub wildcard: Option<ASTNodeIdx>, // This will always be an identifier node,
  pub path: ASTNodeIdx,             // This will always be a string literal node,
}

/// A single Import/Export Member.
pub struct ImportExportMember {
  pub member: ASTNodeIdx,        // This will always be an identifier node
  pub alias: Option<ASTNodeIdx>, // This will always be an identifier node
}

pub trait SingleParamLike {
  fn get_kind(&self) -> &SingleParamKind;
}

// A Single Parameter (For Classes or Functions)
pub enum SingleParamKind {
  Required,
  Named(ASTNodeIdx),
  Optional,
  Rest,
}

/// An AST Function Declaration Node
pub struct ASTFuncDeclNode {
  pub decor: Vec<Decorator>,
  pub is_async: bool,
  pub name: ASTNodeIdx,
  pub is_gen: bool,
  pub params: Vec<SingleParam>,
  pub min_arity: u8,
  pub max_arity: u8,
  pub body: ASTNodeIdx, // This will always be a block stmt
}

/// A Function Parameter
pub struct SingleParam {
  pub name: ASTNodeIdx,
  pub kind: SingleParamKind,
}

impl SingleParamLike for SingleParam {
  fn get_kind(&self) -> &SingleParamKind {
    &self.kind
  }
}

/// A Class or Function Decorator (note that this is not an AST node).
// This will always be an identifier or function call expression.
pub struct Decorator(pub ASTNodeIdx);

// An AST Class Declaration Node
pub struct ASTClassDeclNode {
  pub decor: Vec<Decorator>,
  pub is_abstract: bool,
  pub name: ASTNodeIdx,
  pub params: Vec<ClassParam>,
  pub min_arity: u8,
  pub max_arity: u8,
  pub extends: Vec<ASTNodeIdx>, // This will always be a list of identifier nodes
  pub impls: Vec<ASTNodeIdx>,   // This will always be a list of identifier nodes
  pub init: Option<ASTNodeIdx>, // This will always be a block statement
  pub members: Vec<ClassMember>,
}

// A Class Parameter
pub struct ClassParam {
  pub decor: Vec<Decorator>,
  pub is_pub: bool,
  pub is_const: bool,
  pub param: SingleParam,
}

impl SingleParamLike for ClassParam {
  fn get_kind(&self) -> &SingleParamKind {
    &self.param.kind
  }
}

// A Class Member
pub struct ClassMember {
  pub mode: ClassMemberMode,
  pub member: ClassMemberKind,
}

// The Mode of a Class Member
pub struct ClassMemberMode {
  pub is_public: bool,
  pub is_static: bool,
  pub is_override: bool,
}

// The Kinds of Class Members
pub enum ClassMemberKind {
  Var(Vec<Decorator>, ASTNodeIdx),
  Const(Vec<Decorator>, ASTNodeIdx),
  Func(ASTNodeIdx),
}
