use crate::lexer::tokens::{TokenIdx, TokenKind};
use crate::objects::Object;

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

pub struct ASTArena {
  arena: Vec<ASTArenaNode>,
}

impl Default for ASTArena {
  fn default() -> Self {
    Self {
      arena: vec![ASTArenaNode::new(0.into(), ASTNodeKind::Module(vec![]))],
    }
  }
}

impl ASTArena {
  pub fn append(&mut self, val: ASTNodeKind) -> ASTNodeIdx {
    self.arena.push(ASTArenaNode::new(self.arena.len().into(), val));
    (self.arena.len() - 1).into()
  }

  pub fn get(&self, idx: &ASTNodeIdx) -> &ASTArenaNode {
    &self.arena[idx.0]
  }

  pub fn attach_to_root(&mut self, child: ASTNodeIdx) {
    match &mut self.arena[0].kind {
      ASTNodeKind::Module(m) => m.push(child),
      _ => unreachable!("Root node should be a module node."),
    }
  }
}

pub struct ASTArenaNode {
  pub idx: ASTNodeIdx,
  pub kind: ASTNodeKind,
}

impl ASTArenaNode {
  fn new(idx: ASTNodeIdx, kind: ASTNodeKind) -> Self {
    Self { idx, kind }
  }
}

// TODO: Prevent node duplication.
// When inserting new nodes, check that the arena doesn't already contain a node
// with the exact same signature as the one to be inserted. If a such node already
// exist, return its ASTNodeIdx and do not insert a new one.
pub enum ASTNodeKind {
  Module(Vec<ASTNodeIdx>),

  // Terminal Nodes
  Literal(ASTLiteralNode),
  StringLiteral(TokenIdx),
  SelfLiteral(TokenIdx),
  SuperLiteral(TokenIdx),
  Identifier(TokenIdx),

  // Expression Nodes
  Reassignment(ASTReassignmentNode),
  TernaryConditional(ASTTernaryConditionalNode),
  BinaryExpr(ASTBinaryExprNode),
  UnaryExpr(ASTUnaryExprNode),
  Indexing(ASTIndexingNode),
  ArraySlice(ASTArraySliceNode),
  MemberAccess(ASTMemberAccessNode),
  ArrayLiteral(Vec<ASTNodeIdx>),
  TupleLiteral(Vec<ASTNodeIdx>),
  RepeatLiteral(ASTRepeatLiteralNode),
  SpreadExpr(ASTNodeIdx),
  DictLiteral(Vec<ASTNodeIdx>),
  DictKeyValPair((ASTNodeIdx, ASTNodeIdx)),
  EvaluatedDictKey(ASTNodeIdx),
  CallExpr(ASTCallExprNode),
  Lambda(ASTLambdaNode),
  LoopExpr(ASTNodeIdx), // This will always be a block stmt

  // Statement Nodes
  ExprStmt(ASTNodeIdx),
  BlockStmt(Vec<ASTNodeIdx>),
  BreakStmt(Option<ASTNodeIdx>),
  ContinueStmt,
  ReturnStmt(ASTNodeIdx),
  YieldStmt(ASTNodeIdx),
  ThrowStmt(ASTNodeIdx),
  DelStmt(ASTNodeIdx),
  WhileLoop(ASTWhileLoopNode),
  ForLoop(ASTForLoopNode),
  CompactArrOrTpl(ASTCompactArrOrTplNode),
  CompactDict(ASTCompactDictNode),
  IfStmt(ASTIfStmtNode),
  VarConstDecl(ASTVarConsDeclNode),
  DestructingPattern(ASTDestructingPatternNode),
  DestructingWildCard(Option<ASTNodeIdx>),
  WithStmt(ASTWithStmtNode),
  FuncDecl(ASTFuncDeclNode),
  TryCatchFinally(ASTTryCatchFinallyNode),
  ImportDecl(ASTImportExportNode),
  ExportDecl(ASTImportExportNode),
}

pub struct ASTReassignmentNode {
  pub target: ASTNodeIdx,
  pub kind: ASTReassignmentKind,
  pub value: ASTNodeIdx,
}

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

pub struct ASTTernaryConditionalNode {
  pub condition: ASTNodeIdx,
  pub branch_true: ASTNodeIdx,
  pub branch_false: ASTNodeIdx,
}

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
  pub fn try_equality(tk: &TokenKind) -> Option<Self> {
    match tk {
      TokenKind::LOGIC_EQ => Some(BinaryExprKind::Equals),
      TokenKind::LOGIC_NOT_EQ => Some(BinaryExprKind::NotEquals),
      _ => None,
    }
  }

  pub fn try_relation(tk: &TokenKind) -> Option<Self> {
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

  pub fn try_bit_shift(tk: &TokenKind) -> Option<Self> {
    match tk {
      TokenKind::BIT_L_SHIFT => Some(BinaryExprKind::BitShiftLeft),
      TokenKind::BIT_R_SHIFT => Some(BinaryExprKind::BitShiftRight),
      _ => None,
    }
  }

  pub fn try_range(tk: &TokenKind) -> Option<Self> {
    match tk {
      TokenKind::DOUBLE_DOT => Some(BinaryExprKind::Range),
      TokenKind::RANGE_EQ => Some(BinaryExprKind::RangeEQ),
      _ => None,
    }
  }

  pub fn try_term(tk: &TokenKind) -> Option<Self> {
    match tk {
      TokenKind::PLUS => Some(BinaryExprKind::Add),
      TokenKind::DASH => Some(BinaryExprKind::Subtract),
      _ => None,
    }
  }

  pub fn try_factor(tk: &TokenKind) -> Option<Self> {
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
  pub fn try_from_token(tk: &TokenKind) -> Option<Self> {
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

pub struct ASTIndexingNode {
  pub target: ASTNodeIdx,
  pub indexers: Vec<ASTNodeIdx>,
}

pub struct ASTArraySliceNode {
  pub upper: Option<ASTNodeIdx>,
  pub lower: Option<ASTNodeIdx>,
}

pub struct ASTMemberAccessNode {
  pub is_safe: bool,
  pub target: ASTNodeIdx,
  pub member: ASTNodeIdx, // id node
}

#[derive(Default)]
pub struct ASTCallExprNode {
  pub target: ASTNodeIdx,
  pub val_args: Vec<ASTNodeIdx>,
  pub rest_args: Vec<ASTNodeIdx>,
  pub named_args: Vec<(ASTNodeIdx, ASTNodeIdx)>, // (id node, value node)
}

pub struct ASTLambdaNode {
  pub is_async: bool,
  pub params: Vec<FuncParam>,
  pub min_arity: u8,
  pub max_arity: u8,
  pub body: ASTNodeIdx, // This will always be a block stmt or single expression
}

pub struct ASTLiteralNode {
  pub value: Object,
  pub token_idx: TokenIdx,
}

pub struct ASTRepeatLiteralNode {
  pub kind: RepeatLiteralKind,
  pub value: ASTNodeIdx,
  pub count: ASTNodeIdx,
}

#[repr(u8)]
pub enum RepeatLiteralKind {
  Array,
  Tuple,
}

pub struct ASTWhileLoopNode {
  pub let_id: Option<ASTNodeIdx>,
  pub cond: ASTNodeIdx,
  pub body: ASTNodeIdx, // This will always be a block stmt
}

pub struct ASTForLoopNode {
  pub head: ForLoopHead,
  pub body: ASTNodeIdx, // This will always be a block stmt
}

pub struct ASTCompactArrOrTplNode {
  pub heads: Vec<CompactForLoop>,
  pub body: ASTNodeIdx,
  pub is_tpl: bool,
}

pub struct ASTCompactDictNode {
  pub heads: Vec<CompactForLoop>,
  pub body: ASTNodeIdx,
}

pub struct CompactForLoop {
  pub head: ForLoopHead,
  pub cond: Option<ASTNodeIdx>, // The if-part, it it exists
}

pub struct ForLoopHead {
  pub id: ASTNodeIdx, // Can either be a single id node or destructuring pattern
  pub target: ASTNodeIdx,
}

pub struct ASTIfStmtNode {
  pub cond: ASTNodeIdx,
  pub true_branch: ASTNodeIdx,
  pub else_branch: Option<ASTNodeIdx>,
}

pub struct ASTVarConsDeclNode {
  pub is_const: bool,
  pub id: ASTNodeIdx, // Can either be a single id node or destructuring pattern
  pub val: ASTNodeIdx,
}

pub struct ASTDestructingPatternNode {
  // List of identifier nodes, and optionally, at most one wild-card node.
  pub patterns: Vec<ASTNodeIdx>,
}

pub struct ASTWithStmtNode {
  pub heads: Vec<WithStmtHead>,
  pub body: ASTNodeIdx,
}

pub struct WithStmtHead {
  pub expr: ASTNodeIdx,
  pub id: ASTNodeIdx,
}

pub struct ASTFuncDeclNode {
  pub is_async: bool,
  pub name: ASTNodeIdx,
  pub is_gen: bool,
  pub params: Vec<FuncParam>,
  pub min_arity: u8,
  pub max_arity: u8,
  pub body: ASTNodeIdx, // This will always be a block stmt
}

pub struct FuncParam {
  pub name: ASTNodeIdx,
  pub kind: FuncParamKind,
}

pub enum FuncParamKind {
  Required,
  Named(ASTNodeIdx),
  Optional,
  Rest,
}

pub struct ASTTryCatchFinallyNode {
  pub body: ASTNodeIdx, // This will always be a block stmt
  pub catchers: Vec<CatchPart>,
  pub finally: Option<ASTNodeIdx>, // This will always be a block stmt
}

pub struct CatchPart {
  pub body: ASTNodeIdx, // This will always be a block stmt,
  pub target: Option<CatchTarget>,
}

pub struct CatchTarget {
  pub error_class: ASTNodeIdx,          // This will always be an identifier node
  pub error_result: Option<ASTNodeIdx>, // This will always be an identifier node
}

pub struct ASTImportExportNode {
  pub members: Vec<ImportExportMember>,
  pub wildcard: Option<ASTNodeIdx>, // This will always be an identifier node,
  pub path: ASTNodeIdx,             // This will always be a string literal node,
}

pub struct ImportExportMember {
  pub member: ASTNodeIdx,        // This will always be an identifier node
  pub alias: Option<ASTNodeIdx>, // This will always be an identifier node
}
