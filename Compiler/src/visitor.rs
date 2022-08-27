use std::num::{ParseFloatError, ParseIntError};

use analyzers::symbols::{SymLoc, SymRes};
use core::ast::*;
use core::bytecode::OpCode;
use core::tokens::{TokenIdx, TokenKind};
use core::utils::*;
use objects::func_obj::FuncObj;

use crate::{BreakScope, Compiler, ErrMsg, GcId, LoopScope};

impl Compiler<'_> {
  fn emit_pop_locals(&mut self, count: u16, token: TokenIdx) {
    match count {
      0 => {}
      1 => self.emit_op(OpCode::PopStackTop, token),
      _ => self.emit_op_with_usize(OpCode::PopStackTopN, OpCode::PopStackTopNLong, count as usize, token),
    }
  }

  fn visit_new_block(&mut self, block: &BlockNode, is_loop_body: bool) {
    let prev_decl_count = self.local_decl_count;
    self.local_decl_count = 0;

    self.ast_visit_all(&block.children, ());

    if is_loop_body {
      let count = self.loop_scopes.last().unwrap().decls_count;
      self.emit_pop_locals(count, block.close_token);
    } else {
      self.emit_pop_locals(self.local_decl_count, block.close_token);
    }

    self.local_decl_count = prev_decl_count;
  }

  fn visit_compound_id_decl(&mut self, decl: &CompoundIdDecl) {
    match decl {
      CompoundIdDecl::Single(tok) => self.declare_id(*tok),
      CompoundIdDecl::Unpack(unpacker) => {
        let token = unpacker.token;

        // Emits a destructing instruction that contains a wildcard
        // NOTE: We don't check if either `l` or `u` are less than u16::MAX because
        // the Symbol table already emits an error if we have too many declarations.
        let emit_wildcard = |s: &mut Compiler, l: usize, u: usize, is_ignore: bool| {
          if l < u8::MAX as usize && u < u8::MAX as usize {
            let op = if is_ignore { OpCode::UnpackIgnore } else { OpCode::UnpackAssign };
            s.emit_op(op, token);
            s.emit_raw_byte(l as u8, token);
            s.emit_raw_byte(u as u8, token);
          } else {
            let op = if is_ignore { OpCode::UnpackIgnoreLong } else { OpCode::UnpackAssignLong };
            s.emit_op(op, token);
            s.emit_raw_short(l as u16, token);
            s.emit_raw_short(u as u16, token);
          }
        };

        // Emit the unpacking instruction
        match unpacker.wildcard {
          UnpackWildcard::None(n) => self.emit_op_with_usize(OpCode::UnpackSeq, OpCode::UnpackSeqLong, n, token),
          UnpackWildcard::Ignore(l, u) => emit_wildcard(self, l, u, true),
          UnpackWildcard::Named(l, u) => emit_wildcard(self, l, u, false),
        }

        // Emit the variable assignments
        for member in &unpacker.decls {
          if let UnpackPatternMember::Id(tok) | UnpackPatternMember::NamedWildcard(tok) = member {
            self.declare_id(*tok)
          }
        }
      }
    }
  }

  fn declare_id(&mut self, token: TokenIdx) {
    // This block is actually safe because the SymbolTable ensures declarations and symbol resolutions are valid.
    let symbol = unsafe { self.get_current_table().symbols.iter().find(|s| s.token_idx == token).unwrap_unchecked() };

    match symbol.loc {
      SymLoc::Global(_) => self.emit_op(OpCode::DefineGlobal, token),
      SymLoc::Stack(_) => {} // There ia no instruction to define a local at runtime (stack semantics).
    }

    // Update the number of declarations in the current loop, if possible.
    if let Some(current_loop) = self.loop_scopes.last_mut() {
      if current_loop.can_update {
        current_loop.decls_count += 1;
      }
    }

    self.local_decl_count += 1;
  }

  fn visit_params_list(&mut self, params: &[SingleParam], token: TokenIdx) {
    let mut count = 0;

    for param in params {
      match param.kind {
        SingleParamKind::Named(val) => {
          self.ast_visit_node(val, ());
          count += 1;
        }
        SingleParamKind::Optional => {
          self.emit_op(OpCode::LoadImmNone, param.name);
          count += 1
        }
        SingleParamKind::Required => {} // do nothing
        SingleParamKind::Rest => todo!(),
      }
    }

    if count > 0 {
      self.emit_op_with_usize(OpCode::BindDefaults, OpCode::BindDefaultsLong, count, token);
    }
  }

  fn visit_logic_or_or_and_expr(&mut self, node: &ASTBinaryExprNode, data: ()) {
    // First compile the lhs of the expression which will leave its value on the stack.
    self.ast_visit_node(node.left, ());

    let op_code = match node.kind {
      // For 'AND' expressions, if the lhs is false, then the entire expression must be false.
      // We emit an `JUMP_IF_FALSE_OR_POP` instruction to jump over the rest of this
      // expression if the lhs is falsy.
      BinaryExprKind::LogicAND => OpCode::JumpIfFalseOrPop,
      // For 'OR' expressions, if the lhs is true, then the entire expression must be true.
      // We emit an `JUMP_IF_TRUE_OR_POP` instruction to jump over to the next expression
      // if the lhs is truthy.
      BinaryExprKind::LogicOR => OpCode::JumpIfTrueOrPop,
      // Other binary expressions are not allowed here.
      _ => unreachable!("Can only compile logic 'OR' or 'AND' expressions here."),
    };

    let end_jump = self.emit_jump(op_code, node.token);
    self.ast_visit_node(node.right, data);
    self.patch_jump(end_jump, node.token);
  }

  fn emit_compound_reassignment_operator(&mut self, kind: &ReassignmentKind, token: TokenIdx) {
    match kind {
      ReassignmentKind::Plus => self.emit_op(OpCode::Add, token),
      ReassignmentKind::Minus => self.emit_op(OpCode::Subtract, token),
      ReassignmentKind::Div => self.emit_op(OpCode::Divide, token),
      ReassignmentKind::Mul => self.emit_op(OpCode::Multiply, token),
      ReassignmentKind::Expo => self.emit_op(OpCode::Pow, token),
      ReassignmentKind::Mod => self.emit_op(OpCode::Modulus, token),
      ReassignmentKind::ShiftL => self.emit_op(OpCode::BitwiseShiftLeft, token),
      ReassignmentKind::ShiftR => self.emit_op(OpCode::BitwiseShiftRight, token),
      ReassignmentKind::BitAnd => self.emit_op(OpCode::BitwiseAnd, token),
      ReassignmentKind::Xor => self.emit_op(OpCode::BitwiseXor, token),
      ReassignmentKind::BitOr => self.emit_op(OpCode::BitwiseOr, token),
      ReassignmentKind::Nonish => self.emit_op(OpCode::Nonish, token),
      ReassignmentKind::LogicAnd => todo!(),
      ReassignmentKind::LogicOr => todo!(),
      ReassignmentKind::MatMul => todo!(),
      ReassignmentKind::Assign => unreachable!("Simple reassignment not handled here."),
    };
  }

  fn visit_id_reassignment(&mut self, token: TokenIdx, kind: &ReassignmentKind, value: ASTNodeIdx) {
    let current_table = self.get_current_table();

    let (instr1, instr2, pos) = match current_table.resolved.iter().find(|x| x.0 == token) {
      Some((_, SymRes::Stack(pos))) => (OpCode::SetLocal, OpCode::SetLocalLong, *pos),
      Some((_, SymRes::UpVal(_))) => todo!("SymRes::UpVal(pos)"),
      Some((_, SymRes::Global(pos))) => (OpCode::SetGlobal, OpCode::SetGlobalLong, *pos),
      _ => unreachable!("Invalid id assignment should have been handled by the symbols analyzer.",),
    };

    if let ReassignmentKind::Assign = kind {
      // Proceed to directly reassign the variable.
      self.ast_visit_node(value, ());
    } else {
      // The expression `a /= 2` expands to `a = a / 2`, so we
      // must get the variable's value onto the stack first.
      self.ast_visit_id_literal(&token, ());

      // Then we push the other operand's value onto the stack
      self.ast_visit_node(value, ());

      // Then compute the operation of the two operands.
      self.emit_compound_reassignment_operator(kind, token);
    }

    // Sets the new value (which will be on top of the stack)
    self.emit_op_with_usize(instr1, instr2, pos as usize, token);
  }

  fn patch_breaks(&mut self, loop_start: usize, token: TokenIdx) {
    for idx in (0..self.break_scopes.len()).rev() {
      let br = self.break_scopes[idx];

      if br.parent_loop != loop_start {
        break;
      }

      self.break_scopes.pop();
      self.patch_jump(br.chunk_pos, token);
    }
  }
}

impl<'a> ASTVisitor<'a> for Compiler<'a> {
  type Res = ();
  type Data = ();

  fn get_ast(&self) -> &'a ASTArena {
    self.ast
  }

  fn ast_visit_module(&mut self, node: &ASTModuleNode, data: Self::Data) -> Self::Res {
    self.ast_visit_all(&node.children, data)
  }

  fn ast_visit_block_stmt(&mut self, node: &BlockNode, _: Self::Data) -> Self::Res {
    self.visit_new_block(node, false);
  }

  fn ast_visit_del_stmt(&mut self, _: &ASTNodeIdx, _: Self::Data) -> Self::Res {
    todo!()
  }

  fn ast_visit_export_decl(&mut self, _: &ASTImportExportNode, _: Self::Data) -> Self::Res {
    todo!()
  }

  fn ast_visit_expr_stmt(&mut self, node: &ASTExprStmt, data: Self::Data) -> Self::Res {
    self.ast_visit_node(node.expr, data);
    self.emit_op(OpCode::PopStackTop, node.token);
  }

  fn ast_visit_if_stmt(&mut self, node: &ASTIfStmtNode, data: Self::Data) -> Self::Res {
    self.ast_visit_node(node.cond, data);
    let then_jump = self.emit_jump(OpCode::PopJumpIfFalse, node.token);
    self.visit_new_block(&node.true_branch, false);

    // Emit a jump for over the "else" branch iff there exists one.
    let else_jump = if !matches![node.else_branch, ElseBranch::None] {
      self.emit_jump(OpCode::JumpForward, node.token)
    } else {
      0
    };

    self.patch_jump(then_jump, node.token);

    match &node.else_branch {
      ElseBranch::None => {}
      ElseBranch::Block(b) => self.visit_new_block(b, false),
      ElseBranch::IfStmt(s) => self.ast_visit_node(*s, data),
    }

    // Patch the `else_jump` iff there exists an "else" branch.
    if !matches![node.else_branch, ElseBranch::None] {
      self.patch_jump(else_jump, node.token);
    }
  }

  fn ast_visit_import_decl(&mut self, _: &ASTImportExportNode, _: Self::Data) -> Self::Res {
    todo!()
  }

  fn ast_visit_throw_stmt(&mut self, _: &ASTNodeIdx, _: Self::Data) -> Self::Res {
    todo!()
  }

  fn ast_visit_try_catch_finally(&mut self, _: &ASTTryCatchFinallyNode, _: Self::Data) -> Self::Res {
    todo!()
  }

  fn ast_visit_var_const_decl(&mut self, node: &ASTVarConsDeclNode, data: Self::Data) -> Self::Res {
    self.ast_visit_node(node.val, data);
    self.visit_compound_id_decl(&node.id);
  }

  fn ast_visit_with_stmt(&mut self, _: &ASTWithStmtNode, _: Self::Data) -> Self::Res {
    todo!()
  }

  fn ast_visit_break_stmt(&mut self, node: &ASTBreakStmtNode, _: Self::Data) -> Self::Res {
    let current_loop = self.loop_scopes.last().unwrap();
    let parent_loop = current_loop.loc;

    self.emit_pop_locals(current_loop.decls_count, node.token);
    let chunk_pos = self.emit_jump(OpCode::JumpForward, node.token);
    // TODO: What about breaking with expressions?
    self.break_scopes.push(BreakScope { parent_loop, chunk_pos })
  }

  fn ast_visit_continue_stmt(&mut self, token: &TokenIdx, _: Self::Data) -> Self::Res {
    let current_loop = *self.loop_scopes.last().unwrap();
    self.emit_pop_locals(current_loop.decls_count, *token);
    self.emit_loop(current_loop.loc, *token);
  }

  fn ast_visit_for_loop(&mut self, _: &ASTForLoopNode, _: Self::Data) -> Self::Res {
    todo!()
  }

  fn ast_visit_loop_expr(&mut self, _: &ASTLoopExprNode, _: Self::Data) -> Self::Res {
    todo!()
  }

  fn ast_visit_while_loop(&mut self, node: &ASTWhileLoopNode, data: Self::Data) -> Self::Res {
    let loop_start = self.current_chunk_mut().len();
    self.loop_scopes.push(LoopScope::new(loop_start));

    self.ast_visit_node(node.cond, data);

    let exit_jump = if let Some(id) = node.let_id {
      self.declare_id(id);
      self.emit_jump(OpCode::IfFalsePopJump, node.token)
    } else {
      self.emit_jump(OpCode::PopJumpIfFalse, node.token)
    };

    self.visit_new_block(&node.body, true);

    self.emit_loop(loop_start, node.token);
    self.patch_jump(exit_jump, node.token);

    self.patch_breaks(loop_start, node.token);
    self.loop_scopes.pop();
  }

  fn ast_visit_class_decl(&mut self, _: &ASTClassIdx, _: Self::Data) -> Self::Res {
    todo!()
  }

  fn ast_visit_func_decl(&mut self, node: &ASTFuncDeclNode, _: Self::Data) -> Self::Res {
    let name = node.name;

    // Lock the current loop, if there exists one.
    let mut can_update_curr_loop = false;
    if let Some(current_loop) = self.loop_scopes.last_mut() {
      can_update_curr_loop = current_loop.can_update;
      current_loop.can_update = false;
    };

    let new_func = FuncObj {
      name,
      defaults: vec![],
      min_arity: node.min_arity,
      max_arity: node.max_arity,
      chunk: Default::default(),
    };

    // Add the function object to the constant pool so we can start compiling its chunk
    let const_pos = self.emit_const_gc_obj(new_func.into(), name, false);
    let prev_func_pos = self.current_fn;
    let prev_table = self.current_table;

    // Make the new function the current chunk to compile into.
    self.current_fn = GcId(self.constants.len() - 1);
    self.current_table = node.table_pos;

    self.visit_new_block(&node.body, false);

    // Implicit Return
    self.emit_op(OpCode::LoadImmNone, node.name);
    self.emit_op(OpCode::Return, node.name);

    // Go back to compiling into the previous chunk.
    self.current_fn = prev_func_pos;
    self.current_table = prev_table;

    // Load the function onto the stack and declare it
    self.emit_op_with_usize(OpCode::LoadConstant, OpCode::LoadConstantLong, const_pos, name);

    // Bind default parameters, if any, to the function at runtime before it is declared.
    self.visit_params_list(&node.params, node.name);

    // Unlock the current loop, if there exists one.
    // Done here so that the declaration below is captured in the current loop.
    if let Some(current_loop) = self.loop_scopes.last_mut() {
      current_loop.can_update = can_update_curr_loop;
    };

    self.declare_id(name);
  }

  fn ast_visit_lambda(&mut self, _: &ASTLambdaNode, _: Self::Data) -> Self::Res {
    todo!()
  }

  fn ast_visit_return_stmt(&mut self, node: &ASTReturnStmtNode, _: Self::Data) -> Self::Res {
    self.ast_visit_node(node.val, ());
    self.emit_op(OpCode::Return, node.token);
  }

  fn ast_visit_yield_stmt(&mut self, _: &ASTNodeIdx, _: Self::Data) -> Self::Res {
    todo!()
  }

  fn ast_visit_binary_expr(&mut self, node: &ASTBinaryExprNode, data: Self::Data) -> Self::Res {
    if matches![node.kind, BinaryExprKind::LogicOR | BinaryExprKind::LogicAND] {
      return self.visit_logic_or_or_and_expr(node, data);
    }

    // Compile the operands
    self.ast_visit_node(node.left, data);
    self.ast_visit_node(node.right, data);

    let operator = match node.kind {
      BinaryExprKind::Add => OpCode::Add,
      BinaryExprKind::BitAND => OpCode::BitwiseAnd,
      BinaryExprKind::BitOR => OpCode::BitwiseOr,
      BinaryExprKind::BitShiftLeft => OpCode::BitwiseShiftLeft,
      BinaryExprKind::BitShiftRight => OpCode::BitwiseShiftRight,
      BinaryExprKind::BitXOR => OpCode::BitwiseXor,
      BinaryExprKind::Div => OpCode::Divide,
      BinaryExprKind::Equals => OpCode::Equals,
      BinaryExprKind::GreaterThan => OpCode::GreaterThan,
      BinaryExprKind::GreaterThanEQ => OpCode::GreaterThanEq,
      BinaryExprKind::In => todo!("BinaryExprKind::In"),
      BinaryExprKind::InstOf => todo!("BinaryExprKind::InstOf"),
      BinaryExprKind::LessThan => OpCode::LessThan,
      BinaryExprKind::LessThanEQ => OpCode::LessThanEq,
      BinaryExprKind::LogicAND => unreachable!("Logic 'AND' expressions not compiled here."),
      BinaryExprKind::LogicOR => unreachable!("Logic 'OR' expressions not compiled here."),
      BinaryExprKind::MatMult => todo!("BinaryExprKind::MatMult"),
      BinaryExprKind::Mod => OpCode::Modulus,
      BinaryExprKind::Mult => OpCode::Multiply,
      BinaryExprKind::Nonish => OpCode::Nonish,
      BinaryExprKind::NotEquals => OpCode::NotEq,
      BinaryExprKind::Pipe => todo!("BinaryExprKind::Pipe"),
      BinaryExprKind::Pow => OpCode::Pow,
      BinaryExprKind::Range => OpCode::MakeRange,
      BinaryExprKind::RangeEQ => OpCode::MakeRangeEq,
      BinaryExprKind::Subtract => OpCode::Subtract,
    };

    self.emit_op(operator, node.token)
  }

  fn ast_visit_call_expr(&mut self, node: &ASTCallExprNode, data: Self::Data) -> Self::Res {
    self.ast_visit_node(node.target, data);

    for arg in &node.args {
      match arg {
        CallArg::Val(a) | CallArg::Rest(a) => self.ast_visit_node(*a, data),
        // Named params are resolved at runtime.
        // This could be done at compile time if Hinton supported static typing.
        // CallArg::Named { value, .. } => self.ast_visit_node(*value, data),
        CallArg::Named { .. } => todo!("Compile named arguments"),
      }
    }

    self.emit_op_with_usize(OpCode::FuncCall, OpCode::FuncCallLong, node.args.len(), node.token);
  }

  fn ast_visit_member_access(&mut self, _: &ASTMemberAccessNode, _: Self::Data) -> Self::Res {
    todo!()
  }

  fn ast_visit_reassignment(&mut self, node: &ASTReassignmentNode, _: Self::Data) -> Self::Res {
    match self.ast.get(node.target) {
      ASTNodeKind::IdLiteral(i) => self.visit_id_reassignment(*i, &node.kind, node.value),
      ASTNodeKind::MemberAccess(_) => todo!("Member reassignment."),
      ASTNodeKind::Indexing(_) => todo!("Indexed reassignment."),
      _ => unreachable!("Parser should not allow other node kinds as reassignment target"),
    }
  }

  fn ast_visit_spread_expr(&mut self, _: &ASTNodeIdx, _: Self::Data) -> Self::Res {
    todo!()
  }

  fn ast_visit_string_interpol(&mut self, _s: &[ASTNodeIdx], _: Self::Data) -> Self::Res {
    todo!()
  }

  fn ast_visit_ternary_conditional(&mut self, _: &ASTTernaryConditionalNode, _: Self::Data) -> Self::Res {
    todo!()
  }

  fn ast_visit_unary_expr(&mut self, node: &ASTUnaryExprNode, data: Self::Data) -> Self::Res {
    self.ast_visit_node(node.operand, data);

    let instr = match node.kind {
      UnaryExprKind::LogicNot => OpCode::LogicNot,
      UnaryExprKind::Negate => OpCode::Negate,
      UnaryExprKind::BitNot => OpCode::BitwiseNot,
      UnaryExprKind::New => todo!("UnaryExprKind::New"),
      UnaryExprKind::Typeof => OpCode::TypeOf,
      UnaryExprKind::Await => todo!("UnaryExprKind::Await"),
    };

    self.emit_op(instr, node.token);
  }

  fn ast_visit_array_literal(&mut self, node: &ASTArrayLiteralNode, data: Self::Data) -> Self::Res {
    let vals_count = node.values.len();

    if vals_count <= (u16::MAX as usize) {
      // We reverse the list here because at runtime, we pop each value of the stack in the
      // opposite order (because it *is* a stack). Instead of performing that operation during
      // runtime, we execute it once during compile time.
      for node in node.values.iter().rev() {
        self.ast_visit_node(*node, data);
      }

      self.emit_op_with_usize(OpCode::MakeArray, OpCode::MakeArrayLong, vals_count, node.token);
    } else {
      let err_msg = ErrMsg::MaxCapacity("Too many literal values in the array.".to_string());
      self.emit_error(node.token, err_msg, Some("Try creating two separate arrays.".into()));
    }
  }

  fn ast_visit_array_slice(&mut self, _: &ASTArraySliceNode, _: Self::Data) -> Self::Res {
    todo!()
  }

  fn ast_visit_compact_arr_or_tpl(&mut self, _: &ASTCompactArrOrTplNode, _: Self::Data) -> Self::Res {
    todo!()
  }

  fn ast_visit_compact_dict(&mut self, _: &ASTCompactDictNode, _: Self::Data) -> Self::Res {
    todo!()
  }

  fn ast_visit_dict_key_val_pair(&mut self, _: &(ASTNodeIdx, ASTNodeIdx), _: Self::Data) -> Self::Res {
    todo!()
  }

  fn ast_visit_dict_literal(&mut self, _nodes: &[ASTNodeIdx], _: Self::Data) -> Self::Res {
    todo!()
  }

  fn ast_visit_evaluated_dict_key(&mut self, _: &ASTNodeIdx, _: Self::Data) -> Self::Res {
    todo!()
  }

  fn ast_visit_indexing(&mut self, _: &ASTIndexingNode, _: Self::Data) -> Self::Res {
    todo!()
  }

  fn ast_visit_repeat_literal(&mut self, node: &ASTRepeatLiteralNode, data: Self::Data) -> Self::Res {
    self.ast_visit_node(node.value, data);
    self.ast_visit_node(node.count, data);

    let instr = match node.kind {
      RepeatLiteralKind::Array => OpCode::MakeArrayRepeat,
      RepeatLiteralKind::Tuple => OpCode::MakeTupleRepeat,
    };

    self.emit_op(instr, node.token);
  }

  fn ast_visit_tuple_literal(&mut self, node: &ASTTupleLiteralNode, data: Self::Data) -> Self::Res {
    let vals_count = node.values.len();

    if vals_count <= (u16::MAX as usize) {
      // We reverse the list here because at runtime, we pop each value of the stack in the
      // opposite order (because it *is* a stack). Instead of performing that operation at
      // runtime, we execute it once during compile time.
      for node in node.values.iter().rev() {
        self.ast_visit_node(*node, data);
      }

      self.emit_op_with_usize(OpCode::MakeTuple, OpCode::MakeTupleLong, vals_count, node.token);
    } else {
      let err_msg = ErrMsg::MaxCapacity("Too many literal values in the tuple.".to_string());
      self.emit_error(node.token, err_msg, Some("Try creating two separate arrays.".into()));
    }
  }

  fn ast_visit_id_literal(&mut self, node: &TokenIdx, _: Self::Data) -> Self::Res {
    let current_table = self.get_current_table();

    let (instr1, instr2, pos) = match current_table.resolved.iter().find(|x| x.0 == *node) {
      Some((_, SymRes::Stack(pos))) => (OpCode::GetLocal, OpCode::GetLocalLong, pos),
      Some((_, SymRes::UpVal(_))) => todo!("SymRes::UpVal(pos)"),
      Some((_, SymRes::Global(pos))) => (OpCode::GetGlobal, OpCode::GetGlobalLong, pos),
      Some((_, SymRes::Native(pos))) => (OpCode::LoadNative, OpCode::LoadNativeLong, pos),
      Some((_, SymRes::Primitive(_))) => todo!("SymRes::Primitive(pos)"),
      None | Some((_, SymRes::None)) => unreachable!(
        "Identifier '{}' not yet resolved from table #{}.",
        self.tokens.lexeme(*node),
        self.current_table
      ),
    };

    self.emit_op_with_usize(instr1, instr2, *pos as usize, *node);
  }

  fn ast_visit_self_literal(&mut self, _: &TokenIdx, _: Self::Data) -> Self::Res {
    todo!()
  }

  fn ast_visit_super_literal(&mut self, _: &TokenIdx, _: Self::Data) -> Self::Res {
    todo!()
  }

  fn ast_visit_false_literal(&mut self, node: &TokenIdx, _: Self::Data) -> Self::Res {
    self.emit_op(OpCode::LoadImmFalse, *node);
  }

  fn ast_visit_none_literal(&mut self, node: &TokenIdx, _: Self::Data) -> Self::Res {
    self.emit_op(OpCode::LoadImmNone, *node);
  }

  fn ast_visit_num_literal(&mut self, node: &TokenIdx, _: Self::Data) -> Self::Res {
    let node = *node;
    let kind = self.tokens[node].kind;

    let emit_smart_int = |s: &mut Compiler, node: TokenIdx, val: Result<i64, ParseIntError>| match val {
      Ok(0) => s.emit_op(OpCode::LoadImm0I, node),
      Ok(1) => s.emit_op(OpCode::LoadImm1I, node),
      Ok(i) if i < 256 => s.emit_op_with_byte(OpCode::LoadImmN, i as u8, node),
      Ok(i) if i < u16::MAX as i64 => s.emit_op_with_short(OpCode::LoadImmNLong, i as u16, node),
      Ok(i) => {
        s.emit_const(i.into(), node, true);
      }
      Err(_) => {
        let err_msg = ErrMsg::Internal("Could not convert token to integer.".into());
        s.emit_error(node, err_msg, None);
      }
    };

    let emit_smart_float = |s: &mut Compiler, node: TokenIdx, val: Result<f64, ParseFloatError>| match val {
      Ok(f) if f == 0f64 => s.emit_op(OpCode::LoadImm0F, node),
      Ok(f) if f == 1f64 => s.emit_op(OpCode::LoadImm1F, node),
      Ok(f) => {
        s.emit_const(f.into(), node, true);
      }
      Err(_) => {
        let err_msg = ErrMsg::Internal("Could not convert token to float.".into());
        s.emit_error(node, err_msg, None);
      }
    };

    match kind {
      TokenKind::INT_LIT => emit_smart_int(self, node, parse_int_lexeme(self.tokens.lexeme(node))),
      TokenKind::HEX_LIT => emit_smart_int(self, node, parse_int_from_lexeme_base(self.tokens.lexeme(node), 16)),
      TokenKind::OCTAL_LIT => emit_smart_int(self, node, parse_int_from_lexeme_base(self.tokens.lexeme(node), 8)),
      TokenKind::BINARY_LIT => emit_smart_int(self, node, parse_int_from_lexeme_base(self.tokens.lexeme(node), 2)),
      TokenKind::FLOAT_LIT => emit_smart_float(self, node, parse_float_lexeme(self.tokens.lexeme(node))),
      TokenKind::SCIENTIFIC_LIT => {
        emit_smart_float(self, node, parse_scientific_literal_lexeme(self.tokens.lexeme(node)))
      }
      _ => unreachable!("Should have parsed a numeric token kind. Found '{:?}' instead", kind),
    }
  }

  fn ast_visit_string_literal(&mut self, node: &TokenIdx, _: Self::Data) -> Self::Res {
    self.emit_const_gc_obj(self.tokens.lexeme(*node).into(), *node, true);
  }

  fn ast_visit_true_literal(&mut self, node: &TokenIdx, _: Self::Data) -> Self::Res {
    self.emit_op(OpCode::LoadImmTrue, *node);
  }
}
