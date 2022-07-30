use crate::lexer::tokens::{TokenIdx, TokenKind};
use crate::objects::Object;

pub type ASTNodeIdx = usize;

pub struct ASTArena {
  arena: Vec<ASTArenaNode>,
}

impl Default for ASTArena {
  fn default() -> Self {
    Self {
      arena: vec![ASTArenaNode::new(0, ASTNodeKind::Module(vec![]))],
    }
  }
}

impl ASTArena {
  pub fn append(&mut self, val: ASTNodeKind) -> ASTNodeIdx {
    self.arena.push(ASTArenaNode::new(self.arena.len(), val));
    self.arena.len() - 1
  }

  pub fn get(&self, idx: ASTNodeIdx) -> &ASTArenaNode {
    &self.arena[idx]
  }

  pub fn attach_to_root(&mut self, child: ASTNodeIdx) {
    match &mut self.arena[0].kind {
      ASTNodeKind::Module(m) => m.push(child),
      _ => unreachable!("Root node should be a module node."),
    }
  }

  pub fn len(&self) -> usize {
    self.arena.len()
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

  VarReassignment(ASTVarReassignmentNode),
  Literal(ASTLiteralNode),
  StringLiteral(TokenIdx),
  SelfLiteral(TokenIdx),
  SuperLiteral(TokenIdx),
  Identifier(TokenIdx),
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
  CallExpr(ASTCallExprNode),

  ExprStmt(ASTNodeIdx),
  BlockStmt(Vec<ASTNodeIdx>),
  LoopExprStmt(ASTLoopExprStmtNode),
  BreakStmt(Option<ASTNodeIdx>),
  ContinueStmt,
  ReturnStmt(ASTNodeIdx),
  YieldStmt(ASTNodeIdx),
  ThrowStmt(ASTNodeIdx),
  DelStmt(ASTNodeIdx),
  WhileLoop(ASTWhileLoopNode),
  ForLoop(ASTForLoopNode),
  ForLoopHead(ASTForLoopHeadNode),
  CompactArrOrTpl(ASTCompactArrOrTplNode),
  CompactDict(ASTCompactDictNode),
  CompactForLoop(ASTCompactForLoopNode),
}

pub struct ASTVarReassignmentNode {
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
      TokenKind::RANGE => Some(BinaryExprKind::Range),
      TokenKind::RANGE_EQ => Some(BinaryExprKind::RangeEQ),
      _ => None,
    }
  }

  pub fn try_term(tk: &TokenKind) -> Option<Self> {
    match tk {
      TokenKind::PLUS => Some(BinaryExprKind::Add),
      TokenKind::MINUS => Some(BinaryExprKind::Subtract),
      _ => None,
    }
  }

  pub fn try_factor(tk: &TokenKind) -> Option<Self> {
    match tk {
      TokenKind::STAR => Some(BinaryExprKind::Mult),
      TokenKind::SLASH => Some(BinaryExprKind::Div),
      TokenKind::AT => Some(BinaryExprKind::MatMult),
      TokenKind::MODULUS => Some(BinaryExprKind::Mod),
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
      TokenKind::LOGIC_NOT => Some(UnaryExprKind::LogicNot),
      TokenKind::MINUS => Some(UnaryExprKind::Negate),
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
  pub member: TokenIdx,
}

#[derive(Default)]
pub struct ASTCallExprNode {
  pub target: ASTNodeIdx,
  pub val_args: Vec<ASTNodeIdx>,
  pub rest_args: Vec<ASTNodeIdx>,
  pub named_args: Vec<(TokenIdx, ASTNodeIdx)>,
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

pub struct ASTLoopExprStmtNode {
  pub body: ASTNodeIdx, // This will always be a block stmt
  pub is_expr: bool,
}

pub struct ASTWhileLoopNode {
  pub let_id: Option<TokenIdx>,
  pub cond: ASTNodeIdx,
  pub body: ASTNodeIdx, // This will always be a block stmt
}

pub struct ASTForLoopNode {
  pub head: ASTNodeIdx, // This will always be a for-loop head node
  pub body: ASTNodeIdx, // This will always be a block stmt
}

pub struct ASTCompactArrOrTplNode {
  // This will always be a list of compact
  // for-loop nodes with at least one element
  pub heads: Vec<ASTNodeIdx>,
  pub body: ASTNodeIdx,
  pub is_tpl: bool,
}

pub struct ASTCompactDictNode {
  // This will always be a list of compact
  // for-loop nodes with at least one element
  pub heads: Vec<ASTNodeIdx>,
  pub body: ASTNodeIdx,
}

pub struct ASTCompactForLoopNode {
  pub head: ASTNodeIdx,         // This will always be a for-loop head node
  pub cond: Option<ASTNodeIdx>, // The if-part, it it exists
}

pub struct ASTForLoopHeadNode {
  pub ids: Vec<TokenIdx>,
  pub target: ASTNodeIdx,
}
