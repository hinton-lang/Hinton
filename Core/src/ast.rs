use crate::tokens::{TokenIdx, TokenKind};

/// Represents the index of an AST Node in the ASTArena.
pub type ASTNodeIdx = usize;

/// Represents the index of an Class Declaration Node
/// in the `classes` section of the ASTArena.
pub type ASTClassIdx = usize;

/// Abstract syntax tree in the form
/// of an Arena data structure.
pub struct ASTArena {
  arena: Vec<ASTNodeKind>,
  /// Because class nodes are very heavy and used rarely,
  /// we store them in a separate section so that the
  /// `ASTNodeKind` enum is not penalized by their weight.
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
    self.arena.len() - 1
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
    self.classes.len() - 1
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
  pub fn get(&self, idx: ASTNodeIdx) -> &ASTNodeKind {
    &self.arena[idx]
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
  pub fn get_class(&self, idx: ASTClassIdx) -> &ASTClassDeclNode {
    &self.classes[idx]
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

  ArrayLiteral(ASTArrayLiteralNode),
  ArraySlice(ASTArraySliceNode),
  BinaryExpr(ASTBinaryExprNode),
  BlockStmt(BlockNode),
  BreakStmt(ASTBreakStmtNode),
  CallExpr(ASTCallExprNode),
  ClassDecl(ASTClassIdx),
  CompactArrOrTpl(ASTCompactArrOrTplNode),
  CompactDict(ASTCompactDictNode),
  ContinueStmt(TokenIdx),
  DelStmt(ASTNodeIdx),
  DictKeyValPair((ASTNodeIdx, ASTNodeIdx)),
  DictLiteral(Vec<ASTNodeIdx>),
  EvaluatedDictKey(ASTNodeIdx),
  ExportDecl(ASTImportExportNode),
  ExprStmt(ASTExprStmt),
  FalseLiteral(TokenIdx),
  ForLoop(ASTForLoopNode),
  FuncDecl(ASTFuncDeclNode),
  IdLiteral(TokenIdx),
  IfStmt(ASTIfStmtNode),
  ImportDecl(ASTImportExportNode),
  Indexing(ASTIndexingNode),
  Lambda(ASTLambdaNode),
  LoopExpr(ASTLoopExprNode),
  MemberAccess(ASTMemberAccessNode),
  NoneLiteral(TokenIdx),
  NumLiteral(TokenIdx),
  Reassignment(ASTReassignmentNode),
  RepeatLiteral(ASTRepeatLiteralNode),
  ReturnStmt(ASTReturnStmtNode),
  SelfLiteral(TokenIdx),
  SpreadExpr(ASTNodeIdx),
  StringInterpol(Vec<ASTNodeIdx>),
  StringLiteral(TokenIdx),
  SuperLiteral(TokenIdx),
  TernaryConditional(ASTTernaryConditionalNode),
  ThrowStmt(ASTNodeIdx),
  TrueLiteral(TokenIdx),
  TryCatchFinally(ASTTryCatchFinallyNode),
  TupleLiteral(ASTTupleLiteralNode),
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

/// An AST Block Node.
pub struct BlockNode {
  pub close_token: TokenIdx,
  pub children: Vec<ASTNodeIdx>,
}

pub struct ASTExprStmt {
  pub token: TokenIdx,
  pub expr: ASTNodeIdx,
}

/// An AST Reassignment Node
pub struct ASTReassignmentNode {
  pub target: ASTNodeIdx,
  pub kind: ReassignmentKind,
  pub value: ASTNodeIdx,
}

#[derive(Debug)]
#[repr(u8)]
pub enum ReassignmentKind {
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

impl ReassignmentKind {
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
      TokenKind::AT_EQ => Some(ReassignmentKind::MatMul),
      TokenKind::BIT_AND_EQ => Some(ReassignmentKind::BitAnd),
      TokenKind::BIT_L_SHIFT_EQ => Some(ReassignmentKind::ShiftL),
      TokenKind::BIT_OR_EQ => Some(ReassignmentKind::BitOr),
      TokenKind::BIT_R_SHIFT_EQ => Some(ReassignmentKind::ShiftR),
      TokenKind::BIT_XOR_EQ => Some(ReassignmentKind::Xor),
      TokenKind::EQUALS => Some(ReassignmentKind::Assign),
      TokenKind::POW_EQUALS => Some(ReassignmentKind::Expo),
      TokenKind::LOGIC_AND_EQ => Some(ReassignmentKind::LogicAnd),
      TokenKind::LOGIC_OR_EQ => Some(ReassignmentKind::LogicOr),
      TokenKind::MINUS_EQ => Some(ReassignmentKind::Minus),
      TokenKind::MOD_EQ => Some(ReassignmentKind::Mod),
      TokenKind::NONISH_EQ => Some(ReassignmentKind::Nonish),
      TokenKind::PLUS_EQ => Some(ReassignmentKind::Plus),
      TokenKind::SLASH_EQ => Some(ReassignmentKind::Div),
      TokenKind::STAR_EQ => Some(ReassignmentKind::Mul),
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
  pub token: TokenIdx,
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
  Pow,           // **
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
  pub token: TokenIdx,
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
  pub step: Option<ASTNodeIdx>,
}

/// An AST Member Access Node
pub struct ASTMemberAccessNode {
  pub is_safe: bool,
  pub target: ASTNodeIdx,
  pub member: TokenIdx,
}

/// An AST Call Expression Node
#[derive(Default)]
pub struct ASTCallExprNode {
  pub token: ASTNodeIdx,
  pub target: ASTNodeIdx,
  pub args: Vec<CallArg>,
}

pub enum CallArg {
  Val(ASTNodeIdx),
  Rest(ASTNodeIdx),
  Named { name: TokenIdx, value: ASTNodeIdx },
}

/// An AST Lambda Expression Node
pub struct ASTLambdaNode {
  pub is_async: bool,
  pub params: Vec<SingleParam>,
  pub min_arity: u8,
  pub max_arity: u8,
  pub body: LambdaBody,
}

/// The body of a lambda literal node
pub enum LambdaBody {
  Expr(ASTNodeIdx),
  Block(BlockNode),
}

/// An AST Array Literal Node
pub struct ASTArrayLiteralNode {
  pub token: TokenIdx,
  pub values: Vec<ASTNodeIdx>,
}

/// An AST Tuple Literal Node
pub struct ASTTupleLiteralNode {
  pub token: TokenIdx,
  pub values: Vec<ASTNodeIdx>,
}

/// An AST Repeat Literal Node
pub struct ASTRepeatLiteralNode {
  pub kind: RepeatLiteralKind,
  pub value: ASTNodeIdx,
  pub count: ASTNodeIdx,
  pub token: TokenIdx,
}

#[derive(Debug)]
pub enum RepeatLiteralKind {
  Array,
  Tuple,
}

/// An AST Loop Expression Node
pub struct ASTLoopExprNode {
  pub body: BlockNode,
  pub count: Option<TokenIdx>,
}

/// An AST While Loop Node
pub struct ASTWhileLoopNode {
  pub token: TokenIdx,
  pub let_id: Option<TokenIdx>,
  pub cond: ASTNodeIdx,
  pub body: BlockNode,
}

/// An AST For Loop Node
pub struct ASTForLoopNode {
  pub head: ForLoopHead,
  pub body: BlockNode,
}

/// The Head of a For-loop (both compact and statement)
pub struct ForLoopHead {
  pub id: CompoundIdDecl,
  pub target: ASTNodeIdx,
}

/// A compound identifier declaration, which can be either
/// a single identifier or a destructing pattern.
pub enum CompoundIdDecl {
  Single(TokenIdx),
  Unpack(UnpackPattern),
}

/// A collection of declarations in a destructing statement.
pub struct UnpackPattern {
  pub token: TokenIdx,
  pub decls: Vec<UnpackPatternMember>,
  pub wildcard: UnpackWildcard,
}

/// Encodes the kind of wildcard inside a destructing pattern.
pub enum UnpackWildcard {
  None(usize),
  Ignore(usize, usize),
  Named(usize, usize),
}

/// A single member of a destructing patter.
pub enum UnpackPatternMember {
  Id(TokenIdx),
  NamedWildcard(TokenIdx),
  EmptyWildcard,
}

/// An AST Break Statement Node
pub struct ASTBreakStmtNode {
  pub token: TokenIdx,
  pub val: Option<ASTNodeIdx>,
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

/// An AST If-Else-If Statement Node
pub struct ASTIfStmtNode {
  pub token: TokenIdx,
  pub cond: ASTNodeIdx,
  pub true_branch: BlockNode,
  pub else_branch: ElseBranch,
}

/// The 'else' branch of an 'if' statement.
pub enum ElseBranch {
  None,
  Block(BlockNode),
  IfStmt(ASTNodeIdx),
}

/// An AST Variable/Constant declaration Node
pub struct ASTVarConsDeclNode {
  pub is_const: bool,
  pub id: CompoundIdDecl,
  pub val: ASTNodeIdx,
}

/// An AST With-As Statement Node
pub struct ASTWithStmtNode {
  pub heads: Vec<WithStmtHead>,
  pub body: BlockNode,
}

/// The head of a With-As Statement
pub struct WithStmtHead {
  pub expr: ASTNodeIdx,
  pub id: TokenIdx,
}

/// An AST Try-Catch-Finally Statement Node
pub struct ASTTryCatchFinallyNode {
  pub body: BlockNode,
  pub catchers: Vec<CatchPart>,
  pub finally: Option<BlockNode>,
}

/// The "catch" Part of a Try-Catch-Finally Statement.
pub struct CatchPart {
  pub target: Option<CatchTarget>,
  pub body: BlockNode,
}

/// The Target For the `catch` Part of a Try-Catch-Finally Statement.
pub struct CatchTarget {
  pub error_class: TokenIdx,
  pub error_result: Option<TokenIdx>,
}

/// An AST Import/Export Declaration Node
pub struct ASTImportExportNode {
  pub members: Vec<ImportExportMember>,
  pub wildcard: Option<TokenIdx>,
  pub path: ASTNodeIdx, // This will always be a string literal node,
}

/// A single Import/Export Member.
pub struct ImportExportMember {
  pub member: TokenIdx,
  pub alias: Option<TokenIdx>,
}

/// A Trait to Generalize Over Parameter Nodes
pub trait SingleParamLike {
  fn get_kind(&self) -> &SingleParamKind;
}

/// A Single Parameter (For Classes or Functions)
pub enum SingleParamKind {
  Required,
  Named(ASTNodeIdx), // The default value
  Optional,
  Rest,
}

/// An AST Function Declaration Node
pub struct ASTFuncDeclNode {
  pub decor: Vec<Decorator>,
  pub is_async: bool,
  pub name: TokenIdx,
  pub is_gen: bool,
  pub params: Vec<SingleParam>,
  pub min_arity: u8,
  pub max_arity: u8,
  pub body: BlockNode,
  pub table_pos: usize,
}

/// An AST Return Statement Node
pub struct ASTReturnStmtNode {
  pub token: TokenIdx,
  pub val: ASTNodeIdx,
}

/// A Function Parameter
pub struct SingleParam {
  pub name: TokenIdx,
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
  pub name: TokenIdx,
  pub params: Vec<ClassParam>,
  pub min_arity: u8,
  pub max_arity: u8,
  pub extends: Vec<TokenIdx>,
  pub impls: Vec<TokenIdx>,
  pub init: Option<BlockNode>,
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

pub trait ASTVisitor<'a> {
  type Res;
  type Data: Copy + Clone;

  fn get_ast(&self) -> &'a ASTArena;

  fn ast_visit_all(&mut self, nodes: &[ASTNodeIdx], data: Self::Data) {
    nodes.iter().for_each(|node| {
      self.ast_visit_node(*node, data);
    })
  }

  fn ast_visit_node(&mut self, node: ASTNodeIdx, data: Self::Data) -> Self::Res {
    match self.get_ast().get(node) {
      ASTNodeKind::Module(node) => self.ast_visit_module(node, data),
      ASTNodeKind::ArrayLiteral(node) => self.ast_visit_array_literal(node, data),
      ASTNodeKind::ArraySlice(node) => self.ast_visit_array_slice(node, data),
      ASTNodeKind::BinaryExpr(node) => self.ast_visit_binary_expr(node, data),
      ASTNodeKind::BlockStmt(node) => self.ast_visit_block_stmt(node, data),
      ASTNodeKind::BreakStmt(node) => self.ast_visit_break_stmt(node, data),
      ASTNodeKind::CallExpr(node) => self.ast_visit_call_expr(node, data),
      ASTNodeKind::ClassDecl(node) => self.ast_visit_class_decl(node, data),
      ASTNodeKind::CompactArrOrTpl(node) => self.ast_visit_compact_arr_or_tpl(node, data),
      ASTNodeKind::CompactDict(node) => self.ast_visit_compact_dict(node, data),
      ASTNodeKind::ContinueStmt(node) => self.ast_visit_continue_stmt(node, data),
      ASTNodeKind::DelStmt(node) => self.ast_visit_del_stmt(node, data),
      ASTNodeKind::DictKeyValPair(node) => self.ast_visit_dict_key_val_pair(node, data),
      ASTNodeKind::DictLiteral(node) => self.ast_visit_dict_literal(node, data),
      ASTNodeKind::EvaluatedDictKey(node) => self.ast_visit_evaluated_dict_key(node, data),
      ASTNodeKind::ExportDecl(node) => self.ast_visit_export_decl(node, data),
      ASTNodeKind::ExprStmt(node) => self.ast_visit_expr_stmt(node, data),
      ASTNodeKind::FalseLiteral(node) => self.ast_visit_false_literal(node, data),
      ASTNodeKind::ForLoop(node) => self.ast_visit_for_loop(node, data),
      ASTNodeKind::FuncDecl(node) => self.ast_visit_func_decl(node, data),
      ASTNodeKind::IdLiteral(node) => self.ast_visit_id_literal(node, data),
      ASTNodeKind::IfStmt(node) => self.ast_visit_if_stmt(node, data),
      ASTNodeKind::ImportDecl(node) => self.ast_visit_import_decl(node, data),
      ASTNodeKind::Indexing(node) => self.ast_visit_indexing(node, data),
      ASTNodeKind::Lambda(node) => self.ast_visit_lambda(node, data),
      ASTNodeKind::LoopExpr(node) => self.ast_visit_loop_expr(node, data),
      ASTNodeKind::MemberAccess(node) => self.ast_visit_member_access(node, data),
      ASTNodeKind::NoneLiteral(node) => self.ast_visit_none_literal(node, data),
      ASTNodeKind::NumLiteral(node) => self.ast_visit_num_literal(node, data),
      ASTNodeKind::Reassignment(node) => self.ast_visit_reassignment(node, data),
      ASTNodeKind::RepeatLiteral(node) => self.ast_visit_repeat_literal(node, data),
      ASTNodeKind::ReturnStmt(node) => self.ast_visit_return_stmt(node, data),
      ASTNodeKind::SelfLiteral(node) => self.ast_visit_self_literal(node, data),
      ASTNodeKind::SpreadExpr(node) => self.ast_visit_spread_expr(node, data),
      ASTNodeKind::StringInterpol(node) => self.ast_visit_string_interpol(node, data),
      ASTNodeKind::StringLiteral(node) => self.ast_visit_string_literal(node, data),
      ASTNodeKind::SuperLiteral(node) => self.ast_visit_super_literal(node, data),
      ASTNodeKind::TernaryConditional(node) => self.ast_visit_ternary_conditional(node, data),
      ASTNodeKind::ThrowStmt(node) => self.ast_visit_throw_stmt(node, data),
      ASTNodeKind::TrueLiteral(node) => self.ast_visit_true_literal(node, data),
      ASTNodeKind::TryCatchFinally(node) => self.ast_visit_try_catch_finally(node, data),
      ASTNodeKind::TupleLiteral(node) => self.ast_visit_tuple_literal(node, data),
      ASTNodeKind::UnaryExpr(node) => self.ast_visit_unary_expr(node, data),
      ASTNodeKind::VarConstDecl(node) => self.ast_visit_var_const_decl(node, data),
      ASTNodeKind::WhileLoop(node) => self.ast_visit_while_loop(node, data),
      ASTNodeKind::WithStmt(node) => self.ast_visit_with_stmt(node, data),
      ASTNodeKind::YieldStmt(node) => self.ast_visit_yield_stmt(node, data),
    }
  }

  // Root node.
  fn ast_visit_module(&mut self, node: &ASTModuleNode, data: Self::Data) -> Self::Res;

  // >>>>>>>>>> General statements and declarations
  fn ast_visit_block_stmt(&mut self, node: &BlockNode, data: Self::Data) -> Self::Res;
  fn ast_visit_del_stmt(&mut self, node: &ASTNodeIdx, data: Self::Data) -> Self::Res;
  fn ast_visit_export_decl(&mut self, node: &ASTImportExportNode, data: Self::Data) -> Self::Res;
  fn ast_visit_expr_stmt(&mut self, node: &ASTExprStmt, data: Self::Data) -> Self::Res;
  fn ast_visit_if_stmt(&mut self, node: &ASTIfStmtNode, data: Self::Data) -> Self::Res;
  fn ast_visit_import_decl(&mut self, node: &ASTImportExportNode, data: Self::Data) -> Self::Res;
  fn ast_visit_throw_stmt(&mut self, node: &ASTNodeIdx, data: Self::Data) -> Self::Res;
  fn ast_visit_try_catch_finally(&mut self, node: &ASTTryCatchFinallyNode, data: Self::Data) -> Self::Res;
  fn ast_visit_var_const_decl(&mut self, node: &ASTVarConsDeclNode, data: Self::Data) -> Self::Res;
  fn ast_visit_with_stmt(&mut self, node: &ASTWithStmtNode, data: Self::Data) -> Self::Res;

  // >>>>>>>>>> Loops and loop-related nodes
  fn ast_visit_break_stmt(&mut self, node: &ASTBreakStmtNode, data: Self::Data) -> Self::Res;
  fn ast_visit_continue_stmt(&mut self, node: &TokenIdx, data: Self::Data) -> Self::Res;
  fn ast_visit_for_loop(&mut self, node: &ASTForLoopNode, data: Self::Data) -> Self::Res;
  fn ast_visit_loop_expr(&mut self, node: &ASTLoopExprNode, data: Self::Data) -> Self::Res;
  fn ast_visit_while_loop(&mut self, node: &ASTWhileLoopNode, data: Self::Data) -> Self::Res;

  // >>>>>>>>>> Classes, functions, and function-related nodes
  fn ast_visit_class_decl(&mut self, node: &ASTClassIdx, data: Self::Data) -> Self::Res;
  fn ast_visit_func_decl(&mut self, node: &ASTFuncDeclNode, data: Self::Data) -> Self::Res;
  fn ast_visit_lambda(&mut self, node: &ASTLambdaNode, data: Self::Data) -> Self::Res;
  fn ast_visit_return_stmt(&mut self, node: &ASTReturnStmtNode, data: Self::Data) -> Self::Res;
  fn ast_visit_yield_stmt(&mut self, node: &ASTNodeIdx, data: Self::Data) -> Self::Res;

  // >>>>>>>>>> General expressions
  fn ast_visit_binary_expr(&mut self, node: &ASTBinaryExprNode, data: Self::Data) -> Self::Res;
  fn ast_visit_call_expr(&mut self, node: &ASTCallExprNode, data: Self::Data) -> Self::Res;
  fn ast_visit_member_access(&mut self, node: &ASTMemberAccessNode, data: Self::Data) -> Self::Res;
  fn ast_visit_reassignment(&mut self, node: &ASTReassignmentNode, data: Self::Data) -> Self::Res;
  fn ast_visit_spread_expr(&mut self, node: &ASTNodeIdx, data: Self::Data) -> Self::Res;
  fn ast_visit_string_interpol(&mut self, nodes: &[ASTNodeIdx], data: Self::Data) -> Self::Res;
  fn ast_visit_ternary_conditional(&mut self, node: &ASTTernaryConditionalNode, data: Self::Data) -> Self::Res;
  fn ast_visit_unary_expr(&mut self, node: &ASTUnaryExprNode, data: Self::Data) -> Self::Res;

  // >>>>>>>>>> Collection expressions
  fn ast_visit_array_literal(&mut self, node: &ASTArrayLiteralNode, data: Self::Data) -> Self::Res;
  fn ast_visit_array_slice(&mut self, node: &ASTArraySliceNode, data: Self::Data) -> Self::Res;
  fn ast_visit_compact_arr_or_tpl(&mut self, node: &ASTCompactArrOrTplNode, data: Self::Data) -> Self::Res;
  fn ast_visit_compact_dict(&mut self, node: &ASTCompactDictNode, data: Self::Data) -> Self::Res;
  fn ast_visit_dict_key_val_pair(&mut self, node: &(ASTNodeIdx, ASTNodeIdx), data: Self::Data) -> Self::Res;
  fn ast_visit_dict_literal(&mut self, nodes: &[ASTNodeIdx], data: Self::Data) -> Self::Res;
  fn ast_visit_evaluated_dict_key(&mut self, node: &ASTNodeIdx, data: Self::Data) -> Self::Res;
  fn ast_visit_indexing(&mut self, node: &ASTIndexingNode, data: Self::Data) -> Self::Res;
  fn ast_visit_repeat_literal(&mut self, node: &ASTRepeatLiteralNode, data: Self::Data) -> Self::Res;
  fn ast_visit_tuple_literal(&mut self, node: &ASTTupleLiteralNode, data: Self::Data) -> Self::Res;

  // >>>>>>>>>> Identifier literal nodes
  fn ast_visit_id_literal(&mut self, node: &TokenIdx, data: Self::Data) -> Self::Res;
  fn ast_visit_self_literal(&mut self, node: &TokenIdx, data: Self::Data) -> Self::Res;
  fn ast_visit_super_literal(&mut self, node: &TokenIdx, data: Self::Data) -> Self::Res;

  // >>>>>>>>>> Value literal nodes
  fn ast_visit_false_literal(&mut self, node: &TokenIdx, data: Self::Data) -> Self::Res;
  fn ast_visit_none_literal(&mut self, node: &TokenIdx, data: Self::Data) -> Self::Res;
  fn ast_visit_num_literal(&mut self, node: &TokenIdx, data: Self::Data) -> Self::Res;
  fn ast_visit_string_literal(&mut self, node: &TokenIdx, data: Self::Data) -> Self::Res;
  fn ast_visit_true_literal(&mut self, node: &TokenIdx, data: Self::Data) -> Self::Res;
}
