use std::collections::{HashMap, HashSet};

use bimap::BiHashMap;
use cranelift::{
    codegen::{
        ir::{
            AbiParam, ExtFuncData, ExternalName, FuncRef, SigRef, StackSlot, Type,
            UserExternalName, UserExternalNameRef,
        },
        isa::CallConv,
    },
    prelude::{
        Block, FunctionBuilder, InstBuilder, Signature, Value,
        types::{F64, I64},
    },
};
use ecow::EcoString;

use crate::{
    callbacks::{
        self,
        external::{looks, motion, sensing},
    },
    compiler::lists::ListManager,
    constant_set::ConstantMap,
    data_types::ScratchObject,
    effects::{Effects, VariableWrite, analyze},
    input_primitives::{CompObj, CompStr, Ptr, RtStr, ScratchValue},
    jit_func::JitArgs,
    runtime::CustomBlockId,
    variable_storage::{GenericVarStore, SsaVarStore, VarStore},
};
use rash_core::SpriteId;

mod block;
pub mod lists;
pub use block::ScratchBlock;
mod display;

#[derive(Debug, Clone, Copy)]
pub struct FunctionArg {
    pub val: ScratchValue,
    pub skip_nan: bool,
}

pub struct Compiler<'compiler> {
    pub args_list: Vec<FunctionArg>,
    pub break_counter: usize,
    pub break_points: Vec<Block>,
    pub memory: &'compiler [ScratchObject],
    pub effects: Effects,
    pub call_conv: CallConv,
    pub static_strings: Vec<EcoString>,
    pub lists: ListManager<'compiler>,

    /// See [`Compiler::vars_to_avoid_clobbering`] for more info.
    pub clobber_stack: HashSet<Ptr>,
    pub locals_stack: Vec<Vec<(ScratchValue, bool)>>,
    /// Before:
    ///
    /// ```txt
    /// temp = some_operation(x.clone())
    /// drop(x)
    /// x = temp
    /// ```
    ///
    /// After:
    ///
    /// ```txt
    /// x = some_operation(x) // dropped inside the function
    /// ```
    pub move_optimize: Ptr,

    pub constants: ConstantMap,
    pub func_store: FunctionStore,
    pub vars: Box<dyn VarStore>,
    pub jit_args: JitArgs,

    /// A function that is called when a custom block is encountered.
    ///
    /// It returns the effects of that custom block. Useful for
    /// type related optimizations, just put in a `|_| Effects::unknown()` to ignore.
    pub custom_block_effects: &'compiler dyn Fn(CustomBlockId) -> Effects,

    /// Temporary buffer to store some data.
    /// Be careful, don't let multiple things write to this at the same time.
    pub temp_slot4: (Value, StackSlot),

    /// Storing how many loops inside we are right now
    /// while compiling the current code.
    /// This is a **compile time value**
    ///
    /// # Example
    ///
    /// ```txt
    /// # fn repeat(_: usize) {}
    /// repeat(10) {
    ///     repeat(15) {
    ///         // repeat_stack: 2
    ///         // since we are 2 loops inside
    ///         your code
    ///     }
    ///     // repeat_stack: 1
    ///     // since we are 1 loop inside
    /// }
    /// ```
    ///
    /// This is used with [`ScratchBlock::ControlRepeat`] and [`ScratchBlock::ControlRepeatUntil`]
    pub repeat_stack: usize,

    pub sprite_id: SpriteId,
    pub is_inherently_pausable: bool,
}

impl<'a> Compiler<'a> {
    pub fn new(
        jump_from_existing: bool,
        builder: &mut FunctionBuilder<'_>,
        code: &[ScratchBlock],
        memory: &'a [ScratchObject],
        jit_args: JitArgs,
        args_list: Vec<FunctionArg>,
        sprite_id: SpriteId,
        is_inherently_pausable: bool,
        temp_slot4: (Value, StackSlot),
        func_map: FuncMap,
        call_conv: CallConv,
        custom_block_effects: &'a dyn Fn(CustomBlockId) -> Effects,
        types_from_callsite: &'a dyn Fn(Ptr) -> VariableWrite,
        lists: ListManager<'a>,
    ) -> Self {
        let mut constants = ConstantMap::new();

        let code_block = builder.create_block();
        if jump_from_existing {
            builder.ins().jump(code_block, &[]);
        }
        builder.switch_to_block(code_block);

        let effects = analyze(
            &code,
            &custom_block_effects,
            types_from_callsite,
            &mut |_, _| {},
        );

        let vars: Box<dyn VarStore> = if effects.is_unknown {
            Box::new(GenericVarStore::new(memory))
        } else {
            Box::new(SsaVarStore::new(
                builder,
                &effects,
                &mut constants,
                memory,
                types_from_callsite,
            ))
        };

        Self {
            jit_args,
            temp_slot4,
            vars,
            effects,
            constants,
            call_conv,
            break_points: vec![code_block],
            func_store: FunctionStore::new(func_map),
            break_counter: 0,
            repeat_stack: 0,
            static_strings: Vec::new(),
            clobber_stack: HashSet::new(),
            locals_stack: Vec::new(),
            move_optimize: Ptr(usize::MAX),
            memory,
            args_list,
            sprite_id,
            is_inherently_pausable,
            custom_block_effects,
            lists,
        }
    }

    pub fn compile_block(
        &mut self,
        block: &ScratchBlock,
        builder: &mut FunctionBuilder<'_>,
    ) -> Option<ScratchValue> {
        match block {
            ScratchBlock::VarSet(ptr, obj) => {
                self.var_set(obj, builder, *ptr);
            }
            ScratchBlock::VarRead(ptr) => {
                return Some(self.var_read(builder, *ptr));
            }
            ScratchBlock::OpAdd(a, b) => {
                return Some(ScratchValue::Num(self.op_add(a, b, builder)));
            }
            ScratchBlock::OpSub(a, b) => {
                return Some(ScratchValue::Num(self.op_sub(a, b, builder)));
            }
            ScratchBlock::OpMul(a, b) => {
                let a = a.get_number(self, builder);
                let b = b.get_number(self, builder);
                return Some(ScratchValue::Num(builder.ins().fmul(a, b)));
            }
            ScratchBlock::OpDiv(a, b) => {
                return Some(ScratchValue::Num(self.op_div(a, b, builder)));
            }
            ScratchBlock::OpMod(a, b) => {
                return Some(ScratchValue::Num(self.op_modulo(a, b, builder)));
            }
            ScratchBlock::LogPrint(msg) => self.dbg_log(msg, builder),
            ScratchBlock::ControlRepeat(input, vec) => {
                self.control_repeat(builder, input, vec);
            }
            ScratchBlock::ControlForever(vec) => {
                self.control_forever(builder, vec);
            }
            ScratchBlock::VarChange(ptr, input) => {
                self.var_change(input, builder, *ptr);
            }
            ScratchBlock::ControlIf(input, blocks) => {
                self.control_if(input, builder, blocks);
            }
            ScratchBlock::ControlIfElse(condition, then_block, else_block) => {
                self.control_if_else(condition, builder, then_block, else_block);
            }
            ScratchBlock::ControlRepeatUntil(input, blocks) => {
                self.control_repeat_until(builder, input, blocks);
            }
            ScratchBlock::LocalsWith(locals, blocks) => {
                self.control_locals_with(builder, locals, blocks);
            }
            ScratchBlock::LocalsGet(n) => {
                return self
                    .locals_stack
                    .last()
                    .and_then(|locals| locals.get(*n))
                    .map(|n| n.0)
                    .map(|n| n.clone_in_code(self, builder));
            }
            ScratchBlock::OpCmp(a, b, ordering) => {
                return Some(ScratchValue::Bool(self.op_cmp(a, b, builder, *ordering)));
            }
            ScratchBlock::OpStrJoin(a, b) => {
                return Some(ScratchValue::String(self.op_str_join(a, b, builder)));
            }
            ScratchBlock::OpStrLen(input) => {
                return Some(self.op_str_len(input, builder));
            }
            ScratchBlock::OpStrLetterOf(letter, string) => {
                return Some(ScratchValue::String(
                    self.op_str_letter(letter, string, builder),
                ));
            }
            ScratchBlock::OpStrContains(string, pattern) => {
                return Some(ScratchValue::Bool(
                    self.op_str_contains(string, pattern, builder),
                ));
            }
            ScratchBlock::OpRandom(a, b) => return Some(self.op_random(a, b, builder)),
            ScratchBlock::OpAnd(a, b) => {
                let a = a.get_bool(self, builder);
                let b = b.get_bool(self, builder);
                return Some(ScratchValue::Bool(builder.ins().band(a, b)));
            }
            ScratchBlock::OpNot(a) => {
                let a = a.get_bool(self, builder);
                let one = self.constants.get_int(1, builder);
                return Some(ScratchValue::Bool(builder.ins().isub(one, a)));
            }
            ScratchBlock::OpOr(a, b) => {
                let a = a.get_bool(self, builder);
                let b = b.get_bool(self, builder);
                return Some(ScratchValue::Bool(builder.ins().bor(a, b)));
            }
            ScratchBlock::OpFloor(n) => {
                let n = n.get_number(self, builder);
                let result = builder.ins().floor(n);
                return Some(ScratchValue::Num(result));
            }
            ScratchBlock::OpCeil(n) => {
                let n = n.get_number(self, builder);
                let result = builder.ins().ceil(n);
                return Some(ScratchValue::Num(result));
            }
            ScratchBlock::OpRound(num) => {
                return Some(ScratchValue::Num(self.op_round(num, builder)));
            }
            ScratchBlock::OpAbs(num) => {
                let num = num.get_number(&mut *self, builder);
                return Some(ScratchValue::Num(builder.ins().fabs(num)));
            }
            ScratchBlock::OpSqrt(num) => {
                let num = num.get_number(self, builder);
                return Some(ScratchValue::Num(builder.ins().sqrt(num)));
            }
            ScratchBlock::OpSin(num) => {
                let num = num.get_number(self, builder);
                let inst = self.call_function(builder, callbacks::op::SIN, &[F64], &[F64], &[num]);
                return Some(ScratchValue::Num(builder.inst_results(inst)[0]));
            }
            ScratchBlock::OpCos(num) => {
                let num = num.get_number(self, builder);
                let inst = self.call_function(builder, callbacks::op::COS, &[F64], &[F64], &[num]);
                return Some(ScratchValue::Num(builder.inst_results(inst)[0]));
            }
            ScratchBlock::OpTan(num) => {
                let num = num.get_number(self, builder);
                let inst = self.call_function(builder, callbacks::op::TAN, &[F64], &[F64], &[num]);
                return Some(ScratchValue::Num(builder.inst_results(inst)[0]));
            }
            ScratchBlock::OpAsin(num) => {
                let num = num.get_number(self, builder);
                let inst = self.call_function(builder, callbacks::op::ASIN, &[F64], &[F64], &[num]);
                return Some(ScratchValue::Num(builder.inst_results(inst)[0]));
            }
            ScratchBlock::OpAcos(num) => {
                let num = num.get_number(self, builder);
                let inst = self.call_function(builder, callbacks::op::ACOS, &[F64], &[F64], &[num]);
                return Some(ScratchValue::Num(builder.inst_results(inst)[0]));
            }
            ScratchBlock::OpAtan(num) => {
                let num = num.get_number(self, builder);
                let inst = self.call_function(builder, callbacks::op::ATAN, &[F64], &[F64], &[num]);
                return Some(ScratchValue::Num(builder.inst_results(inst)[0]));
            }
            ScratchBlock::OpLn(num) => {
                let num = num.get_number(self, builder);
                let inst = self.call_function(builder, callbacks::op::LN, &[F64], &[F64], &[num]);
                return Some(ScratchValue::Num(builder.inst_results(inst)[0]));
            }
            ScratchBlock::OpLog(num) => {
                let num = num.get_number(self, builder);
                let inst =
                    self.call_function(builder, callbacks::op::LOG10, &[F64], &[F64], &[num]);
                return Some(ScratchValue::Num(builder.inst_results(inst)[0]));
            }
            ScratchBlock::OpExp(num) => {
                let num = num.get_number(self, builder);
                let inst = self.call_function(builder, callbacks::op::EXP, &[F64], &[F64], &[num]);
                return Some(ScratchValue::Num(builder.inst_results(inst)[0]));
            }
            ScratchBlock::OpExp10(num) => {
                let num = num.get_number(self, builder);
                let inst =
                    self.call_function(builder, callbacks::op::EXP10, &[F64], &[F64], &[num]);
                return Some(ScratchValue::Num(builder.inst_results(inst)[0]));
            }
            ScratchBlock::ScreenRefresh => {
                self.screen_refresh(builder);
            }
            ScratchBlock::ControlStopThisScript => {
                self.control_stop_this_script(builder);
            }
            ScratchBlock::FunctionCallNoScreenRefresh(custom_block_id, args) => {
                self.call_custom_block(*custom_block_id, builder, args, false);
            }
            ScratchBlock::FunctionCallScreenRefresh(custom_block_id, args) => {
                self.call_custom_block(*custom_block_id, builder, args, true);
            }
            ScratchBlock::FunctionGetArg(idx) => {
                let val = self.args_list[*idx];
                return Some(val.val.clone_in_code(self, builder));
            }
            ScratchBlock::MotionGoToXY(x, y) => {
                let x = x.get_number(self, builder);
                let y = y.get_number(self, builder);

                let id = self.constants.get_int(self.sprite_id.0, builder);

                self.call_function(
                    builder,
                    motion::C_GO_TO,
                    &[I64, I64, F64, F64],
                    &[],
                    &[self.jit_args.runstate_ptr, id, x, y],
                );
            }
            ScratchBlock::MotionChangeX(x) => {
                let x = x.get_number(self, builder);

                let id = self.constants.get_int(self.sprite_id.0, builder);

                self.call_function(
                    builder,
                    motion::C_CHANGE_X,
                    &[I64, I64, F64],
                    &[],
                    &[self.jit_args.runstate_ptr, id, x],
                );
            }
            ScratchBlock::MotionChangeY(y) => {
                let y = y.get_number(self, builder);

                let id = self.constants.get_int(self.sprite_id.0, builder);

                self.call_function(
                    builder,
                    motion::C_CHANGE_Y,
                    &[I64, I64, F64],
                    &[],
                    &[self.jit_args.runstate_ptr, id, y],
                );
            }
            ScratchBlock::MotionSetX(x) => {
                let x = x.get_number(self, builder);

                let id = self.constants.get_int(self.sprite_id.0, builder);

                self.call_function(
                    builder,
                    motion::C_SET_X,
                    &[I64, I64, F64],
                    &[],
                    &[self.jit_args.runstate_ptr, id, x],
                );
            }
            ScratchBlock::MotionSetY(y) => {
                let y = y.get_number(self, builder);

                let id = self.constants.get_int(self.sprite_id.0, builder);

                self.call_function(
                    builder,
                    motion::C_SET_Y,
                    &[I64, I64, F64],
                    &[],
                    &[self.jit_args.runstate_ptr, id, y],
                );
            }
            ScratchBlock::MotionGetX => {
                let id = self.constants.get_int(self.sprite_id.0, builder);

                let inst = self.call_function(
                    builder,
                    motion::C_GET_X,
                    &[I64, I64],
                    &[F64],
                    &[self.jit_args.runstate_ptr, id],
                );

                let val = builder.inst_results(inst)[0];
                return Some(ScratchValue::Num(val));
            }
            ScratchBlock::MotionGetY => {
                let id = self.constants.get_int(self.sprite_id.0, builder);

                let inst = self.call_function(
                    builder,
                    motion::C_GET_Y,
                    &[I64, I64],
                    &[F64],
                    &[self.jit_args.runstate_ptr, id],
                );

                let val = builder.inst_results(inst)[0];
                return Some(ScratchValue::Num(val));
            }
            ScratchBlock::LooksSetShown(shown) => {
                let id = self.constants.get_int(self.sprite_id.0, builder);
                let shown = self.constants.get_int(i64::from(*shown), builder);

                self.call_function(
                    builder,
                    looks::C_SET_SHOWN,
                    &[I64, I64, I64],
                    &[],
                    &[self.jit_args.runstate_ptr, id, shown],
                );
            }
            ScratchBlock::SensingDaysSince2000 => {
                let inst = self.call_function(builder, sensing::DAYS_SINCE_2000, &[], &[F64], &[]);
                let val = builder.inst_results(inst)[0];
                return Some(ScratchValue::Num(val));
            }
            ScratchBlock::SensingMouseDown => {
                let inst = self.call_function(
                    builder,
                    sensing::MOUSE_DOWN,
                    &[I64],
                    &[I64],
                    &[self.jit_args.runstate_ptr],
                );
                let val = builder.inst_results(inst)[0];
                return Some(ScratchValue::Bool(val));
            }
            ScratchBlock::ListClear(l) => {
                self.list_clear(builder, l);
            }
        }
        None
    }

    pub fn save_vars(&mut self, builder: &mut FunctionBuilder<'_>) {
        self.vars
            .save(builder, &mut self.constants, self.memory, &self.effects);
    }

    // Lifetime/ownership of the string:
    //
    // `s` is embedded into the machine code as a constant. The machine code
    // references it, but Rust doesn't track that reference or its lifetime.
    //
    // We keep `s` alive by transferring ownership to `self.static_strings`,
    // effectively tying its lifetime to that of the machine code.
    //
    // This is safe because `static_strings`:
    // - Outlives the machine code.
    // - Never accesses or modifies the strings.
    // - Only drops them when the machine code is de-initialized.
    //
    // The string returned by this function is a separate runtime clone,
    // with its own independent lifetime.
    pub fn clone_str(&mut self, builder: &mut FunctionBuilder<'_>, s: RtStr) -> CompStr {
        let [v1, v2] = s.make_constant(builder, &mut self.constants).0;

        self.static_strings
            .push(unsafe { std::mem::transmute::<RtStr, EcoString>(s) });

        self.call_function(
            builder,
            callbacks::types::CLONE_STR,
            &[I64, I64, I64],
            &[],
            &[v1, v2, self.temp_slot4.0],
        );

        CompStr([
            builder.ins().stack_load(I64, I64, self.temp_slot4.1, 0),
            builder.ins().stack_load(I64, I64, self.temp_slot4.1, 8),
        ])
    }

    pub fn get_local_type(&self, n: usize) -> VariableWrite {
        self.locals_stack
            .last()
            .and_then(|locals| locals.get(n))
            .map(|(v, skip_nan)| VariableWrite {
                ty: (*v).into(),
                skip_nan: *skip_nan,
                direct: true,
            })
            .unwrap_or_default()
    }

    pub fn create_string_stack_slot(builder: &mut FunctionBuilder<'_>) -> Value {
        let stack_slot =
            builder.create_sized_stack_slot(cranelift::codegen::ir::StackSlotData::new(
                cranelift::codegen::ir::StackSlotKind::ExplicitSlot,
                3 * std::mem::size_of::<i64>() as u32,
                0,
            ));
        builder.ins().stack_addr(I64, stack_slot, 0)
    }

    pub fn call_to_num_with_decimal_check(
        &mut self,
        builder: &mut FunctionBuilder<'_>,
        obj: CompObj,
    ) -> (Value, Value) {
        let [i1, i2, i3] = obj.0;
        self.call_function(
            builder,
            callbacks::types::TO_NUMBER_WITH_DECIMAL_CHECK,
            &[I64, I64, I64, I64],
            &[],
            &[i1, i2, i3, self.temp_slot4.0],
        );
        let n = builder.ins().stack_load(I64, F64, self.temp_slot4.1, 0);
        let b = builder.ins().stack_load(I64, I64, self.temp_slot4.1, 8);
        (n, b)
    }
}

pub type FuncMap = BiHashMap<UserExternalNameRef, UserExternalName>;

pub struct FunctionStore {
    pub func_map: FuncMap,
    pub signatures: HashMap<Signature, SigRef>,
    definitions: HashMap<UserExternalName, FuncRef>,
}

impl FunctionStore {
    pub fn new(func_map: FuncMap) -> Self {
        Self {
            func_map,
            signatures: HashMap::new(),
            definitions: HashMap::new(),
        }
    }

    #[track_caller]
    pub fn get_function(
        &mut self,
        call_conv: CallConv,
        builder: &mut FunctionBuilder,
        name: UserExternalName,
        params: &[Type],
        returns: &[Type],
    ) -> FuncRef {
        if let Some(func_ref) = self.definitions.get(&name) {
            return *func_ref;
        }

        let mut sig = Signature::new(call_conv);
        for param in params {
            sig.params.push(AbiParam::new(*param));
        }
        for ret in returns {
            sig.returns.push(AbiParam::new(*ret));
        }

        let signature = if let Some(sigref) = self.signatures.get(&sig) {
            *sigref
        } else {
            let r = builder.import_signature(sig.clone());
            self.signatures.insert(sig.clone(), r);
            r
        };

        let Some(func_ref) = self.func_map.get_by_right(&name) else {
            crate::print_function_addresses();
            panic!("Function not found: {name}");
        };
        let func = ExtFuncData {
            name: ExternalName::User(*func_ref),
            signature,
            colocated: false,
            patchable: false,
        };
        let func_ref = builder.import_function(func);
        self.definitions.insert(name, func_ref);

        func_ref
    }
}
