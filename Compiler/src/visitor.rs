use analyzers::symbols::SymRes;
use core::ast::*;
use core::bytecode::OpCode;
use core::objects::str_obj;
use core::tokens::{TokenIdx, TokenKind};
use core::utils::*;
use std::num::{ParseFloatError, ParseIntError};

use crate::{Compiler, ErrMsg, Value};

impl<'a> ASTVisitor<'a> for Compiler<'a> {
  type Res = ();
  type Data = ();

  fn get_ast(&self) -> &'a ASTArena {
    self.ast
  }

  fn ast_visit_module(&mut self, node: &ASTModuleNode, data: Self::Data) -> Self::Res {
    self.ast_visit_all(&node.children, data)
  }

  fn ast_visit_block_stmt(&mut self, _: &BlockNode, _: Self::Data) -> Self::Res {
    todo!()
  }

  fn ast_visit_del_stmt(&mut self, _: &ASTNodeIdx, _: Self::Data) -> Self::Res {
    todo!()
  }

  fn ast_visit_export_decl(&mut self, _: &ASTImportExportNode, _: Self::Data) -> Self::Res {
    todo!()
  }

  fn ast_visit_expr_stmt(&mut self, node: &ASTExprStmt, data: Self::Data) -> Self::Res {
    self.ast_visit_node(node.expr, data);
    self.emit_op_code(OpCode::PopStackTop, node.token);
  }

  fn ast_visit_if_stmt(&mut self, _: &ASTIfStmtNode, _: Self::Data) -> Self::Res {
    todo!()
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

  fn ast_visit_var_const_decl(&mut self, _: &ASTVarConsDeclNode, _: Self::Data) -> Self::Res {
    todo!()
  }

  fn ast_visit_with_stmt(&mut self, _: &ASTWithStmtNode, _: Self::Data) -> Self::Res {
    todo!()
  }

  fn ast_visit_break_stmt(&mut self, _: &ASTBreakStmtNode, _: Self::Data) -> Self::Res {
    todo!()
  }

  fn ast_visit_continue_stmt(&mut self, _: &TokenIdx, _: Self::Data) -> Self::Res {
    todo!()
  }

  fn ast_visit_for_loop(&mut self, _: &ASTForLoopNode, _: Self::Data) -> Self::Res {
    todo!()
  }

  fn ast_visit_loop_expr(&mut self, _: &ASTLoopExprNode, _: Self::Data) -> Self::Res {
    todo!()
  }

  fn ast_visit_while_loop(&mut self, _: &ASTWhileLoopNode, _: Self::Data) -> Self::Res {
    todo!()
  }

  fn ast_visit_class_decl(&mut self, _: &ASTClassIdx, _: Self::Data) -> Self::Res {
    todo!()
  }

  fn ast_visit_func_decl(&mut self, _: &ASTFuncDeclNode, _: Self::Data) -> Self::Res {
    todo!()
  }

  fn ast_visit_lambda(&mut self, _: &ASTLambdaNode, _: Self::Data) -> Self::Res {
    todo!()
  }

  fn ast_visit_return_stmt(&mut self, _: &ASTReturnStmtNode, _: Self::Data) -> Self::Res {
    todo!()
  }

  fn ast_visit_yield_stmt(&mut self, _: &ASTNodeIdx, _: Self::Data) -> Self::Res {
    todo!()
  }

  fn ast_visit_binary_expr(&mut self, node: &ASTBinaryExprNode, data: Self::Data) -> Self::Res {
    if let BinaryExprKind::LogicOR = node.kind {
      todo!("Compile logic 'OR' expressions")
    }

    if let BinaryExprKind::LogicOR = node.kind {
      todo!("Compile logic 'AND' expressions")
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

    self.emit_op_code(operator, node.token)
  }

  fn ast_visit_call_expr(&mut self, _: &ASTCallExprNode, _: Self::Data) -> Self::Res {
    todo!()
  }

  fn ast_visit_member_access(&mut self, _: &ASTMemberAccessNode, _: Self::Data) -> Self::Res {
    todo!()
  }

  fn ast_visit_reassignment(&mut self, _: &ASTReassignmentNode, _: Self::Data) -> Self::Res {
    todo!()
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

    self.emit_op_code(instr, node.token);
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

      self.emit_op_code_with_usize(OpCode::MakeArray, OpCode::MakeArrayLong, vals_count, node.token);
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

    self.emit_op_code(instr, node.token);
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

      self.emit_op_code_with_usize(OpCode::MakeTuple, OpCode::MakeTupleLong, vals_count, node.token);
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
      Some((_, SymRes::Native(_))) => todo!("SymRes::Native(pos)"),
      Some((_, SymRes::Primitive(_))) => todo!("SymRes::Primitive(pos)"),
      None | Some((_, SymRes::None)) => unreachable!(
        "Identifier '{}' not yet resolved from table #{}.",
        self.tokens.lexeme(*node),
        self.current_table
      ),
    };

    self.emit_op_code_with_usize(instr1, instr2, *pos as usize, *node);
  }

  fn ast_visit_self_literal(&mut self, _: &TokenIdx, _: Self::Data) -> Self::Res {
    todo!()
  }

  fn ast_visit_super_literal(&mut self, _: &TokenIdx, _: Self::Data) -> Self::Res {
    todo!()
  }

  fn ast_visit_false_literal(&mut self, node: &TokenIdx, _: Self::Data) -> Self::Res {
    self.emit_op_code(OpCode::LoadImmFalse, *node);
  }

  fn ast_visit_none_literal(&mut self, node: &TokenIdx, _: Self::Data) -> Self::Res {
    self.emit_op_code(OpCode::LoadImmNone, *node);
  }

  fn ast_visit_num_literal(&mut self, node: &TokenIdx, _: Self::Data) -> Self::Res {
    let node = *node;
    let kind = self.tokens[node].kind;

    let emit_smart_int = |s: &mut Compiler, node: TokenIdx, val: Result<i64, ParseIntError>| match val {
      Ok(0) => s.emit_op_code(OpCode::LoadImm0I, node),
      Ok(1) => s.emit_op_code(OpCode::LoadImm1I, node),
      Ok(i) if i < 256 => s.emit_op_code_with_byte(OpCode::LoadImmN, i as u8, node),
      Ok(i) if i < u16::MAX as i64 => s.emit_op_code_with_short(OpCode::LoadImmN, i as u16, node),
      Ok(i) => s.emit_const(Value::Int(i), node, true),
      Err(_) => {
        let err_msg = ErrMsg::Internal("Could not convert token to integer.".into());
        s.emit_error(node, err_msg, None);
      }
    };

    let emit_smart_float = |s: &mut Compiler, node: TokenIdx, val: Result<f64, ParseFloatError>| match val {
      Ok(f) if f == 0f64 => s.emit_op_code(OpCode::LoadImm0F, node),
      Ok(f) if f == 1f64 => s.emit_op_code(OpCode::LoadImm1F, node),
      Ok(f) => s.emit_const(Value::Float(f), node, true),
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
    self.emit_const(str_obj::StrObj(self.tokens.lexeme(*node)).into(), *node, true)
  }

  fn ast_visit_true_literal(&mut self, node: &TokenIdx, _: Self::Data) -> Self::Res {
    self.emit_op_code(OpCode::LoadImmTrue, *node);
  }
}
