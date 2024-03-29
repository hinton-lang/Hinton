use crate::built_in::BuiltIn;
use crate::core::ast::{BinaryExprType, UnaryExprType};
use crate::core::bytecode::OpCode;
use crate::errors::RuntimeErrorType;
use crate::objects::class_obj::{ClassField, ClassObject};
use crate::objects::dictionary_obj::DictObject;
use crate::objects::indexing::to_bounded_index;
use crate::objects::iter_obj::{get_next_in_iter, make_iter};
use crate::objects::*;
use crate::virtual_machine::{RuntimeResult, VM};
use hashbrown::HashMap;
use std::cell::RefCell;
use std::rc::Rc;

impl VM {
   /// Executes the instructions in a chunk.
   pub(crate) fn run(&mut self) -> RuntimeResult {
      loop {
         let instruction = self.next_op_code();

         let exec = match instruction {
            OpCode::PopStackTop => {
               self.pop_stack();
               RuntimeResult::Continue
            }

            // Object loaders
            OpCode::LoadConstant | OpCode::LoadConstantLong => self.op_load_constant(),
            OpCode::LoadImm0F => self.push_stack(Object::Float(0f64)),
            OpCode::LoadImm0I => self.push_stack(Object::Int(0i64)),
            OpCode::LoadImm1F => self.push_stack(Object::Float(1f64)),
            OpCode::LoadImm1I => self.push_stack(Object::Int(1i64)),
            OpCode::LoadImmFalse => self.push_stack(Object::Bool(false)),
            OpCode::LoadImmN | OpCode::LoadImmNLong => self.op_load_immediate_n(),
            OpCode::LoadImmNull => self.push_stack(Object::Null),
            OpCode::LoadImmTrue => self.push_stack(Object::Bool(true)),
            OpCode::LoadNative => self.op_load_native(),
            OpCode::LoadPrimitive => self.op_load_primitive(),

            // Object makers
            OpCode::MakeArray | OpCode::MakeArrayLong => self.op_make_array(),
            OpCode::MakeClosure | OpCode::MakeClosureLong => self.op_make_closure(),
            OpCode::MakeClosureLarge | OpCode::MakeClosureLongLarge => self.op_make_closure_large(),
            OpCode::MakeDict | OpCode::MakeDictLong => self.op_make_dictionary(),
            OpCode::MakeIter => self.op_make_iter(),
            OpCode::MakeRange => self.op_make_range(),
            OpCode::MakeTuple | OpCode::MakeTupleLong => self.op_make_tuple(),

            // Global declarations
            OpCode::DefineGlobal | OpCode::DefineGlobalLong => self.op_define_global(),
            OpCode::GetGlobal | OpCode::GetGlobalLong => self.op_get_global(),
            OpCode::SetGlobal | OpCode::SetGlobalLong => self.op_set_global(),

            // Local declarations
            OpCode::GetLocal | OpCode::GetLocalLong => self.op_get_local(),
            OpCode::SetLocal | OpCode::SetLocalLong => self.op_set_local(),

            // Operators
            OpCode::Add => self.binary_operation(BinaryExprType::Addition),
            OpCode::BitwiseAnd => self.binary_operation(BinaryExprType::BitwiseAND),
            OpCode::BitwiseNot => self.unary_operation(UnaryExprType::BitwiseNeg),
            OpCode::BitwiseOr => self.binary_operation(BinaryExprType::BitwiseOR),
            OpCode::BitwiseShiftLeft => self.binary_operation(BinaryExprType::BitwiseShiftLeft),
            OpCode::BitwiseShiftRight => self.binary_operation(BinaryExprType::BitwiseShiftRight),
            OpCode::BitwiseXor => self.binary_operation(BinaryExprType::BitwiseXOR),
            OpCode::Divide => self.binary_operation(BinaryExprType::Division),
            OpCode::Equals => self.binary_operation(BinaryExprType::LogicEQ),
            OpCode::Expo => self.binary_operation(BinaryExprType::Expo),
            OpCode::GreaterThan => self.binary_operation(BinaryExprType::LogicGreaterThan),
            OpCode::GreaterThanEq => self.binary_operation(BinaryExprType::LogicGreaterThanEQ),
            OpCode::LessThan => self.binary_operation(BinaryExprType::LogicLessThan),
            OpCode::LessThanEq => self.binary_operation(BinaryExprType::LogicLessThanEQ),
            OpCode::LogicNot => self.unary_operation(UnaryExprType::LogicNeg),
            OpCode::Modulus => self.binary_operation(BinaryExprType::Modulus),
            OpCode::Multiply => self.binary_operation(BinaryExprType::Multiplication),
            OpCode::Negate => self.unary_operation(UnaryExprType::ArithmeticNeg),
            OpCode::NotEq => self.binary_operation(BinaryExprType::LogicNotEQ),
            OpCode::NullishCoalescing => self.binary_operation(BinaryExprType::Nullish),
            OpCode::Subscript => self.op_subscript(),
            OpCode::Subtract => self.binary_operation(BinaryExprType::Minus),

            // Jumps
            OpCode::ForIterNextOrJump => self.op_get_iter_next_or_jump(),
            OpCode::JumpForward => self.op_jump_forward(),
            OpCode::JumpIfFalseOrPop => self.op_jump_if_false_or_pop(),
            OpCode::JumpIfTrueOrPop => self.op_jump_if_true_or_pop(),
            OpCode::LoopJump | OpCode::LoopJumpLong => self.op_loop_jump(),
            OpCode::PopJumpIfFalse => self.op_pop_and_jump_if_false(),

            // Functions and Closures
            OpCode::BindDefaults => self.op_bind_function_defaults(),
            OpCode::CloseUpVal | OpCode::CloseUpValLong => self.up_close_up_value(),
            OpCode::FuncCall => self.op_func_call(),
            OpCode::GetUpVal | OpCode::GetUpValLong => self.op_get_up_value(),
            OpCode::PopCloseUpVal => self.op_pop_stack_and_close_up_value(),
            OpCode::Return => self.op_function_return(),
            OpCode::SetUpVal | OpCode::SetUpValLong => self.op_set_up_value(),

            // Classes & Instances
            OpCode::AppendClassField => self.append_class_field(),
            OpCode::MakeClass | OpCode::MakeClassLong => self.op_make_class(),
            OpCode::MakeInstance => self.op_make_instance(),

            // Collection manipulators
            OpCode::GetProp | OpCode::GetPropLong => self.op_get_property(),
            OpCode::SetProp | OpCode::SetPropLong => self.op_set_property(),
            OpCode::SubscriptAssign => self.op_subscript_assign(),

            // VM-Specific
            OpCode::EndVirtualMachine => self.op_end_virtual_machine(),
         };

         // Prints the execution of the program.
         // self.print_execution(instruction);

         match exec {
            RuntimeResult::Continue => continue,
            _ => return exec,
         }
      }
   }

   /// Executes the instruction to end the virtual machine with an OK result.
   fn op_end_virtual_machine(&mut self) -> RuntimeResult {
      self.pop_stack(); // Remove the main function off the stack
      self.frames.pop();

      RuntimeResult::EndOK
   }

   /// Executes the instruction to pop the top of the stack, and jump forward by the given
   /// offset if the popped value is falsey.
   fn op_pop_and_jump_if_false(&mut self) -> RuntimeResult {
      // The POP_JUMP_IF_FALSE instruction always has a short as its operand.
      let offset = self.next_short() as usize;

      if self.pop_stack().is_falsey() {
         self.current_frame_mut().ip += offset;
      }

      RuntimeResult::Continue
   }

   /// Executes the instruction to jump forward by the given offset if the top of the stack is
   /// falsey, or pop the top off the stack otherwise.
   fn op_jump_if_false_or_pop(&mut self) -> RuntimeResult {
      // The JUMP_IF_FALSE_OR_POP instruction always has a short as its operand.
      let offset = self.next_short() as usize;

      if self.peek_stack(0).is_falsey() {
         self.current_frame_mut().ip += offset;
      } else {
         self.pop_stack();
      }

      RuntimeResult::Continue
   }

   /// Executes the instruction to pop the top of the stack, and jump forward by the given
   /// offset if the popped value is truthy.
   fn op_jump_if_true_or_pop(&mut self) -> RuntimeResult {
      // The JUMP_IF_TRUE_OR_POP instruction always has a short as its operand.
      let offset = self.next_short() as usize;

      if !self.peek_stack(0).is_falsey() {
         self.current_frame_mut().ip += offset;
      } else {
         self.pop_stack();
      }

      RuntimeResult::Continue
   }

   /// Executes the instruction to jump forward by the given offset.
   fn op_jump_forward(&mut self) -> RuntimeResult {
      // The JUMP_FORWARD instruction always has a short as its operand.
      let offset = self.next_short() as usize;
      self.current_frame_mut().ip += offset;
      RuntimeResult::Continue
   }

   /// Executes the instruction to jump (loop) back by the given offset.
   fn op_loop_jump(&mut self) -> RuntimeResult {
      let offset = self.get_std_or_long_operand(OpCode::LoopJump);
      self.current_frame_mut().ip -= offset;
      RuntimeResult::Continue
   }

   /// Executes the instruction to load a native function onto the stack.
   fn op_load_native(&mut self) -> RuntimeResult {
      let native = self.next_byte() as usize;

      match self.built_in.natives.get_native_fn_object(native) {
         Ok(f) => self.push_stack(Object::Native(Box::new(f))),
         Err(e) => e,
      }
   }

   /// Executes the instruction to load a native function onto the stack.
   fn op_load_primitive(&mut self) -> RuntimeResult {
      let name_idx = self.next_byte() as usize;

      let name = match self.read_constant(name_idx) {
         Object::String(s) => s,
         _ => unreachable!("Expected String for primitive property name."),
      };

      match self.built_in.primitives.get_class_object(&name) {
         Ok(f) => self.push_stack(Object::Class(f)),
         Err(e) => e,
      }
   }

   /// Executes the instruction to call a function object.
   fn op_func_call(&mut self) -> RuntimeResult {
      let arg_count = self.next_byte(); // Functions can only have 255-MAX parameters
      let maybe_function = self.peek_stack(arg_count as usize).clone();
      self.call_object(maybe_function, arg_count)
   }

   /// Executes the instruction to make a closure object from a function object.
   /// This method only covers the `OP_MAKE_CLOSURE` and `OP_MAKE_CLOSURE_LONG` instructions
   /// with a variable number of operands. The byte or short immediately following the
   /// instruction encodes the position of the function object in the constant pool to be
   /// converted into a closure. Each consecutive operand is two bytes long.
   fn op_make_closure(&mut self) -> RuntimeResult {
      let pos = self.get_std_or_long_operand(OpCode::MakeClosure);

      let function = match self.read_constant(pos) {
         Object::Function(obj) => obj,
         _ => unreachable!("Expected a Function object for closure."),
      };

      let up_val_count = function.borrow().up_val_count;
      let mut up_values: Vec<Rc<RefCell<UpValRef>>> = Vec::with_capacity(up_val_count);

      for _ in 0..up_val_count {
         let is_local = self.next_byte() == 1u8;
         let index = self.next_byte() as usize;

         let up = if is_local {
            self.create_up_value(self.current_frame().return_index + index)
         } else {
            self.get_up_val(index)
         };

         up_values.push(up.clone());
      }

      self.push_stack(Object::Closure(ClosureObject { function, up_values }))
   }

   /// Executes the instruction to make a closure object from a function object.
   /// This method only covers the `OP_MAKE_CLOSURE_LARGE` and `OP_MAKE_CLOSURE_LONG_LARGE`
   /// instructions with a variable number of operands. The byte or short immediately following
   /// the instruction encodes the position of the function object in the constant pool to be
   /// converted into a closure. Each consecutive operand is three bytes long.
   fn op_make_closure_large(&mut self) -> RuntimeResult {
      let pos = self.get_std_or_long_operand(OpCode::MakeClosureLarge);

      let function = match self.read_constant(pos) {
         Object::Function(obj) => obj,
         _ => unreachable!("Expected a Function object for closure."),
      };

      let up_val_count = function.borrow().up_val_count;
      let mut up_values: Vec<Rc<RefCell<UpValRef>>> = Vec::with_capacity(up_val_count);

      for _ in 0..up_val_count {
         let is_local = self.next_byte() == 1u8;
         let index = self.next_short() as usize;

         let up = if is_local {
            self.create_up_value(self.current_frame().return_index + index)
         } else {
            self.get_up_val(index)
         };

         up_values.push(up.clone());
      }

      self.push_stack(Object::Closure(ClosureObject { function, up_values }))
   }

   /// Executes the instruction to get an UpValue from the current call frame's closure.
   fn op_get_up_value(&mut self) -> RuntimeResult {
      let pos = self.get_std_or_long_operand(OpCode::GetUpVal);

      let val = match &*self.get_up_val(pos).borrow() {
         UpValRef::Open(l) => self.peek_stack_abs(*l).clone(),
         UpValRef::Closed(o) => o.clone(),
      };

      self.push_stack(val)
   }

   /// Executes the instruction to modify an UpValue in the current call frame's closure.
   fn op_set_up_value(&mut self) -> RuntimeResult {
      let pos = self.get_std_or_long_operand(OpCode::SetUpVal);
      let new_val = self.stack.last().unwrap().clone();

      match &mut *self.get_up_val(pos).borrow_mut() {
         UpValRef::Open(l) => self.stack[*l] = new_val,
         UpValRef::Closed(u) => *u = new_val,
      }

      RuntimeResult::Continue
   }

   /// Executes the instruction to lift the UpValue at the given index from the stack
   /// and onto the heap (close it), without removing the original value from the stack.
   fn up_close_up_value(&mut self) -> RuntimeResult {
      let pos = self.get_std_or_long_operand(OpCode::CloseUpVal);

      for u in self.up_values.iter() {
         if u.borrow().is_open_at(self.current_frame().return_index + pos) {
            let new_val = self.peek_stack_abs(self.current_frame().return_index + pos);
            u.replace(UpValRef::Closed(new_val.clone()));
            break;
         }
      }

      RuntimeResult::Continue
   }

   /// Executes the instruction to pop the last object of the stack and close the
   /// open UpValue referring to it (if such UpValue exists).
   fn op_pop_stack_and_close_up_value(&mut self) -> RuntimeResult {
      let new_val = self.pop_stack();

      for u in self.up_values.iter() {
         if u.borrow().is_open_at(self.stack.len()) {
            u.replace(UpValRef::Closed(new_val));
            break;
         }
      }

      RuntimeResult::Continue
   }

   /// Executes the instruction to bind `N` number of default parameters to a function.
   fn op_bind_function_defaults(&mut self) -> RuntimeResult {
      // Functions can only have 255-MAX parameters
      let param_count = self.next_byte();

      let mut defaults: Vec<Object> = vec![];
      for _ in 0..param_count {
         let p = self.pop_stack();
         defaults.push(p);
      }
      defaults.reverse();

      match self.peek_stack_mut(0) {
         Object::Function(m) => {
            m.borrow_mut().defaults = defaults;
         }
         Object::Closure(m) => {
            m.function.borrow_mut().defaults = defaults;
         }
         _ => unreachable!("Expected a Function object on TOS."),
      }

      RuntimeResult::Continue
   }

   /// Executes the instruction to return out of a function call.
   fn op_function_return(&mut self) -> RuntimeResult {
      let result = self.pop_stack();
      let locals_to_pop = self.stack.len() - self.current_frame().return_index;

      // Pops local declarations from the stack
      for _ in 0..locals_to_pop {
         self.pop_stack();
      }

      // removes the call frame
      self.frames.pop();
      self.push_stack(result)
   }

   /// Executes the instruction to create a class object.
   fn op_make_class(&mut self) -> RuntimeResult {
      let pos = self.get_std_or_long_operand(OpCode::MakeClass);

      let name = match self.read_constant(pos) {
         Object::String(s) => s,
         _ => unreachable!("Expected String for class name."),
      };

      self.push_stack(Object::from(ClassObject::new(&name)))
   }

   /// Executes the instruction to create an instance from a class object.
   fn op_make_instance(&mut self) -> RuntimeResult {
      let arg_count = self.next_byte(); // Instances can only have 255-MAX arguments
      let maybe_class = self.peek_stack(arg_count as usize).clone();
      self.create_instance(maybe_class, arg_count)
   }

   /// Executes the instruction to get a property from an object.
   fn op_get_property(&mut self) -> RuntimeResult {
      let pos = self.get_std_or_long_operand(OpCode::GetProp);

      let prop_name = match self.read_constant(pos) {
         Object::String(name) => name,
         _ => unreachable!("Expected String for property access name."),
      };

      let value = self.pop_stack();
      match value {
         Object::Instance(x) => match x.borrow().get_prop(&self, &prop_name) {
            Ok(o) => match o {
               Object::Closure(c) => self.push_stack(c.into_bound_method(x.clone())),
               Object::Function(f) => self.push_stack(FuncObject::bound_method(f, x.clone())),
               _ => self.push_stack(o),
            },
            Err(e) => e,
         },
         Object::Dict(x) => match x.get_prop(&prop_name) {
            Ok(o) => self.push_stack(o),
            Err(e) => e.to_runtime_error(),
         },
         Object::Int(_) => BuiltIn::primitive_prop(self, value, "Int", prop_name),
         Object::String(_) => BuiltIn::primitive_prop(self, value, "String", prop_name),
         Object::Array(_) => BuiltIn::primitive_prop(self, value, "Array", prop_name),
         Object::Tuple(_) => BuiltIn::primitive_prop(self, value, "Tuple", prop_name),
         Object::Class(c) => match c.borrow().get_static_prop(prop_name) {
            Ok(val) => self.push_stack(val),
            Err(e) => e,
         },
         _ => todo!("Objects of type '{}' also have properties.", value.type_name()),
      }
   }

   /// Executes the instruction to modify a property in an object.
   fn op_set_property(&mut self) -> RuntimeResult {
      let pos = self.get_std_or_long_operand(OpCode::SetProp);

      let prop_name = match self.read_constant(pos) {
         Object::String(name) => name,
         _ => unreachable!("Expected String for property assignment name."),
      };

      let value = self.pop_stack();

      return match self.pop_stack() {
         Object::Instance(inst) => match inst.borrow_mut().set_prop(&self, prop_name, value) {
            Ok(o) => self.push_stack(o),
            Err(e) => e,
         },
         Object::Dict(mut dict) => {
            dict.set_prop(&prop_name, value.clone());
            self.push_stack(value)
         }
         _ => todo!("Other objects also have properties."),
      };
   }

   /// Executes the instruction to modify the value of a collection at the provided index.
   fn op_subscript_assign(&mut self) -> RuntimeResult {
      let target = self.pop_stack();
      let index = self.pop_stack();
      let value = self.pop_stack();

      match target {
         Object::Array(arr) => {
            let idx = match index {
               Object::Int(i) => to_bounded_index(i, arr.borrow().len()),
               _ => {
                  return RuntimeResult::Error {
                     error: RuntimeErrorType::TypeError,
                     message: format!(
                        "Array reassignment subscript must be an Int. Found '{}' instead.",
                        index.type_name()
                     ),
                  }
               }
            };

            match idx {
               Some(i) => {
                  arr.borrow_mut()[i] = value.clone();
                  self.push_stack(value)
               }
               None => RuntimeResult::Error {
                  error: RuntimeErrorType::TypeError,
                  message: "Array index out of bounds.".to_string(),
               },
            }
         }
         Object::Dict(mut dict) => match index {
            Object::String(s) => {
               dict.set_prop(&s, value.clone());
               self.push_stack(value)
            }
            _ => RuntimeResult::Error {
               error: RuntimeErrorType::TypeError,
               message: format!(
                  "Dictionary reassignment subscript must be a String. Found '{}' instead.",
                  index.type_name()
               ),
            },
         },
         _ => RuntimeResult::Error {
            error: RuntimeErrorType::TypeError,
            message: format!(
               "Objects of type '{}' do not support subscripted item reassignment.",
               target.type_name()
            ),
         },
      }
   }

   /// Executes the instruction to create a range object with the two objects on the TOS.
   fn op_make_range(&mut self) -> RuntimeResult {
      let right = self.pop_stack();
      let left = self.pop_stack();

      if left.is_int() && right.is_int() {
         let a = left.as_int().unwrap();
         let b = right.as_int().unwrap();
         self.push_stack(Object::Range(RangeObject { min: a, max: b }))
      } else {
         return RuntimeResult::Error {
            error: RuntimeErrorType::TypeError,
            message: format!(
               "Range operation not defined for operands of type '{}' and '{}'.",
               left.type_name(),
               right.type_name()
            ),
         };
      }
   }

   /// Executes the instruction to perform a unary operation with the object at the TOS.
   fn unary_operation(&mut self, opr: UnaryExprType) -> RuntimeResult {
      let val = self.pop_stack();

      let result = match opr {
         UnaryExprType::ArithmeticNeg => -val,
         UnaryExprType::LogicNeg => Ok(Object::Bool(val.is_falsey())),
         UnaryExprType::BitwiseNeg => !val,
      };

      match result {
         Ok(r) => self.push_stack(r),
         Err(e) => e.to_runtime_error(),
      }
   }

   /// Executes the instruction to perform a binary operation with the two objects at the TOS.
   fn binary_operation(&mut self, opr: BinaryExprType) -> RuntimeResult {
      let val2 = self.pop_stack();
      let val1 = self.pop_stack();

      let result = match opr {
         BinaryExprType::Addition => val1 + val2,
         BinaryExprType::BitwiseAND => val1 & val2,
         BinaryExprType::BitwiseOR => val1 | val2,
         BinaryExprType::BitwiseShiftLeft => val1 << val2,
         BinaryExprType::BitwiseShiftRight => val1 >> val2,
         BinaryExprType::BitwiseXOR => val1 ^ val2,
         BinaryExprType::Division => val1 / val2,
         BinaryExprType::Expo => val1.pow(val2),
         BinaryExprType::LogicEQ => Ok(Object::Bool(val1 == val2)),
         BinaryExprType::LogicGreaterThan => val1.gt(val2),
         BinaryExprType::LogicGreaterThanEQ => val1.gteq(val2),
         BinaryExprType::LogicLessThan => val1.lt(val2),
         BinaryExprType::LogicLessThanEQ => val1.lteq(val2),
         BinaryExprType::LogicNotEQ => Ok(Object::Bool(val1 != val2)),
         BinaryExprType::Minus => val1 - val2,
         BinaryExprType::Modulus => val1 % val2,
         BinaryExprType::Multiplication => val1 * val2,
         BinaryExprType::Nullish => {
            if matches!(val1, Object::Null) {
               Ok(val2)
            } else {
               Ok(val1)
            }
         }
         _ => unreachable!("The other binary operations have special instruction methods."),
      };

      match result {
         Ok(r) => self.push_stack(r),
         Err(e) => e.to_runtime_error(),
      }
   }

   /// Executes the instruction to get the next item in the iterator at the TOS, or jump
   /// forward by the given offset if the iterator is empty. This instruction primarily used
   /// in `for-in` loops.
   fn op_get_iter_next_or_jump(&mut self) -> RuntimeResult {
      let jump = self.next_short() as usize;

      match self.peek_stack(0) {
         Object::Iter(i) => match get_next_in_iter(i) {
            Ok(o) => self.push_stack(o),
            Err(_) => {
               self.pop_stack();
               self.current_frame_mut().ip += jump;
               RuntimeResult::Continue
            }
         },
         _ => unreachable!("Expected iterable object on TOS."),
      }
   }

   /// Executes the instruction to create an array object with the top `N` stack objects.
   fn op_make_array(&mut self) -> RuntimeResult {
      // The number of values to pop from the stack. Essentially the size of the array.
      let size = self.get_std_or_long_operand(OpCode::MakeArray);
      let mut arr_values: Vec<Object> = vec![];

      for _ in 0..size {
         arr_values.push(self.pop_stack());
      }

      let arr = Rc::new(RefCell::new(arr_values));
      self.push_stack(Object::Array(arr))
   }

   /// Executes the instruction to create a tuple object with the top `N` stack objects.
   fn op_make_tuple(&mut self) -> RuntimeResult {
      // The number of values to pop from the stack. Essentially the size of the array.
      let size = self.get_std_or_long_operand(OpCode::MakeTuple);
      let mut tuple_values: Vec<Object> = Vec::with_capacity(size);

      for _ in 0..size {
         tuple_values.push(self.pop_stack());
      }

      self.push_stack(Object::Tuple(Rc::new(tuple_values)))
   }

   /// Executes the instruction to create a dictionary object with the top `N` key-value
   /// pairs on the stack, where the key is a Hinton string object and value is any Hinton object.
   fn op_make_dictionary(&mut self) -> RuntimeResult {
      // The number of values to pop from the stack. Essentially the size of the array.
      let size = self.get_std_or_long_operand(OpCode::MakeDict);
      let mut dict: HashMap<String, Object> = HashMap::new();

      for _ in 0..size {
         let value = self.pop_stack();

         match self.pop_stack() {
            Object::String(key) => {
               dict.insert(key, value);
            }
            _ => unreachable!("Expected String for dictionary key."),
         }
      }

      self.push_stack(DictObject::from_hashmap(dict))
   }

   /// Executes the instruction to subscript and object by some index.
   fn op_subscript(&mut self) -> RuntimeResult {
      let index = self.pop_stack();
      let target = self.pop_stack();

      match target.subscript(&index) {
         Ok(r) => self.push_stack(r),
         Err(e) => e.to_runtime_error(),
      }
   }

   /// Executes the instruction to get the value of a local variable.
   fn op_get_local(&mut self) -> RuntimeResult {
      // The position of the local variable's value in the stack
      let pos = self.get_std_or_long_operand(OpCode::GetLocal);

      let idx = self.current_frame().return_index + pos;
      let value = self.peek_stack_abs(idx).clone();
      self.push_stack(value)
   }

   /// Executes the instruction to modify the value of a local variable.
   fn op_set_local(&mut self) -> RuntimeResult {
      // The position of the local variable's value in the stack
      let pos = self.get_std_or_long_operand(OpCode::SetLocal);

      let value = self.stack.last().unwrap();
      let offset = self.current_frame().return_index;

      self.stack[pos + offset] = value.clone();
      RuntimeResult::Continue
   }

   /// Executes the instruction to define a global variable with the object at the TOS.
   fn op_define_global(&mut self) -> RuntimeResult {
      let pos = self.get_std_or_long_operand(OpCode::DefineGlobal);

      // Gets the name from the pool assigns the value to the global
      if let Object::String(name) = self.read_constant(pos) {
         let val = self.pop_stack();
         self.globals.insert(name, val);
         RuntimeResult::Continue
      } else {
         unreachable!("Expected a String for global declaration name.");
      }
   }

   /// Executes the instruction to get the value of a global variable.
   fn op_get_global(&mut self) -> RuntimeResult {
      let pos = self.get_std_or_long_operand(OpCode::GetGlobal);

      // Gets the name from the pool
      if let Object::String(name) = self.read_constant(pos) {
         let val = self.globals.get(&name).unwrap().clone();
         self.push_stack(val)
      } else {
         unreachable!("Expected a String as name of global variable access.");
      }
   }

   /// Executes the instruction to modify the value of a global variable.
   fn op_set_global(&mut self) -> RuntimeResult {
      let pos = self.get_std_or_long_operand(OpCode::SetGlobal);

      // Gets the name from the pool
      if let Object::String(name) = self.read_constant(pos) {
         let val = self.stack.last().unwrap().clone();
         self.globals.insert(name, val);
         RuntimeResult::Continue
      } else {
         unreachable!("Expected a String as global declaration name.");
      }
   }

   /// Executes the instruction to create an iterator object with the object at the TOS.
   fn op_make_iter(&mut self) -> RuntimeResult {
      let tos = self.pop_stack();

      match make_iter(tos) {
         Ok(iter) => self.push_stack(iter),
         Err(e) => e,
      }
   }

   /// Executes the instruction to load the integer `N` onto the stack.
   fn op_load_immediate_n(&mut self) -> RuntimeResult {
      let imm = self.get_std_or_long_operand(OpCode::LoadImmN) as i64;
      self.push_stack(Object::Int(imm))
   }

   /// Executes the instruction to load a constant from the current's call frame's
   /// function's constant pool onto the stack.
   fn op_load_constant(&mut self) -> RuntimeResult {
      let pos = self.get_std_or_long_operand(OpCode::LoadConstant);
      let val = self.read_constant(pos);
      self.push_stack(val)
   }

   /// Appends a field to a class object.
   fn append_class_field(&mut self) -> RuntimeResult {
      let mode = self.next_byte();
      let is_static = (mode & 4) == 4;

      // Computes the new mode without the "static" bit.
      let mode = if (mode & 8) == 8 {
         (mode & 0b1111_0011) | 0b0000_0100
      } else {
         mode & 0b1111_0011
      };

      // Gets the field name string
      let field_name = match self.pop_stack() {
         Object::String(s) => s,
         _ => unreachable!("Expected String on TOS for class field."),
      };

      // Gets the field value
      let value = Box::new(self.pop_stack());

      match self.peek_stack(0).clone() {
         Object::Class(c) => {
            let mut class = c.borrow_mut();

            // If the class initializer is private, then the class cannot be
            // constructed from the outside.
            if field_name == "init" && (mode & 0b_0000_0100) != 4 {
               class.is_constructable = false
            }

            // Gets the mutable reference to the appropriate hash_map to store the field
            let storage = if is_static {
               &mut class.statics
            } else {
               &mut class.members
            };

            storage.insert(field_name, ClassField { value, mode });
         }
         _ => unreachable!("Expected Class object on TOS to bind field '{}'.", field_name),
      }

      RuntimeResult::Continue
   }
}
