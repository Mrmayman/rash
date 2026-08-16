use std::{
    cmp::Ordering,
    collections::HashMap,
    sync::{LazyLock, Mutex},
};

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
use smol_str::SmolStr;

use crate::{
    callbacks,
    constant_set::ConstantMap,
    data_types::{ID_BOOL, ID_NUMBER, ID_STRING, ScratchObject},
    effects::{CheckEffects, Effects, VariableWrite},
    graphics::{RunState, SpriteId},
    input_primitives::{Input, Ptr, ScratchValue},
    runtime::CustomBlockId,
    variable_storage::{GenericVarStore, SsaVarStore, VarStore},
};

mod display;

pub static MEMORY: LazyLock<Mutex<Box<[ScratchObject]>>> =
    LazyLock::new(|| Mutex::new(vec![ScratchObject::Number(0.0); 4096].into_boxed_slice()));

#[allow(unused)]
#[derive(Debug, PartialEq)]
pub enum ScratchBlock {
    /// Sets a variable to a value.
    VarSet(Ptr, Input),
    /// Sets a variable to variable + input.
    ///
    /// Basically like the `x += a` operation.
    VarChange(Ptr, Input),
    /// Reads a value from a value and returns it.
    /// Meant to be used as the input to other blocks.
    VarRead(Ptr),
    OpAdd(Input, Input),
    OpSub(Input, Input),
    OpMul(Input, Input),
    OpDiv(Input, Input),
    OpRound(Input),
    OpStrJoin(Input, Input),
    OpMod(Input, Input),
    OpStrLen(Input),
    OpBAnd(Input, Input),
    OpBNot(Input),
    OpBOr(Input, Input),
    OpMFloor(Input),
    OpMAbs(Input),
    OpMSqrt(Input),
    OpMSin(Input),
    OpMCos(Input),
    OpMTan(Input),
    OpCmp(Input, Input, Ordering),
    OpRandom(Input, Input),
    OpStrLetterOf(Input, Input),
    OpStrContains(Input, Input),
    ControlIf(Input, Vec<ScratchBlock>),
    ControlIfElse(Input, Vec<ScratchBlock>, Vec<ScratchBlock>),
    /// A repeat loop that *supports* screen refresh
    /// (pausing/resuming of code).
    ///
    /// This loop doesn't provide screen refresh by default
    /// but you can insert [`ScratchBlock::ScreenRefresh`]
    /// inside it.
    ControlRepeat(Input, Vec<ScratchBlock>),
    /// Repeats until a condition is true.
    ControlRepeatUntil(Input, Vec<ScratchBlock>),
    ControlForever(Vec<ScratchBlock>),
    ControlStopThisScript,
    FunctionCallNoScreenRefresh(CustomBlockId, Vec<Input>),
    FunctionCallScreenRefresh(CustomBlockId, Vec<Input>),
    FunctionGetArg(usize),
    /// A block to trigger a screen refresh.
    ///
    /// Similar to coroutines in other languages,
    /// screen refresh allows a script to pause and
    /// resume.
    ///
    /// Note: don't insert this explicitly in a repeat
    /// loop as screen refreshes are auto-inserted in
    /// loops matching Scratch behaviour.
    ScreenRefresh,
    MotionGoToXY(Input, Input),
    MotionChangeX(Input),
    MotionChangeY(Input),
    MotionSetX(Input),
    MotionSetY(Input),
    MotionGetX,
    MotionGetY,
    LooksShown(bool),
    SensingDaysSince2000,

    Log(Input),
}

#[derive(PartialEq, Eq, Debug, Default, Clone, Copy)]
pub enum VarTypeChecked {
    Number,
    Bool,
    String,
    #[default]
    Object,
}

impl VarTypeChecked {
    pub fn get_id(self) -> Option<i64> {
        match self {
            Self::Number => Some(ID_NUMBER),
            Self::Bool => Some(ID_BOOL),
            Self::String => Some(ID_STRING),
            Self::Object => None,
        }
    }
}

impl From<VarType> for VarTypeChecked {
    fn from(value: VarType) -> Self {
        match value {
            VarType::Number => Self::Number,
            VarType::Bool => Self::Bool,
            VarType::String => Self::String,
        }
    }
}

impl From<Option<VarType>> for VarTypeChecked {
    fn from(value: Option<VarType>) -> Self {
        match value {
            Some(n) => n.into(),
            None => Self::Object,
        }
    }
}

impl From<VarTypeChecked> for Option<VarType> {
    fn from(val: VarTypeChecked) -> Self {
        Some(match val {
            VarTypeChecked::Number => VarType::Number,
            VarTypeChecked::Bool => VarType::Bool,
            VarTypeChecked::String => VarType::String,
            VarTypeChecked::Object => return None,
        })
    }
}

impl ScratchBlock {
    #[must_use]
    pub fn return_type(
        &self,
        mut vartype: impl FnMut(Ptr) -> VariableWrite,
    ) -> Option<VariableWrite> {
        match self {
            ScratchBlock::VarRead(ptr) => Some(vartype(*ptr)),
            ScratchBlock::OpRandom(_, _)
            | ScratchBlock::OpMSqrt(_)
            | ScratchBlock::OpMod(_, _)
            | ScratchBlock::OpDiv(_, _) => Some(VariableWrite::normal(VarTypeChecked::Number)),
            ScratchBlock::FunctionGetArg(_) => Some(VariableWrite::normal(VarTypeChecked::Object)),

            ScratchBlock::OpAdd(_, _)
            | ScratchBlock::OpSub(_, _)
            | ScratchBlock::OpMul(_, _)
            | ScratchBlock::OpMFloor(_)
            | ScratchBlock::OpRound(_)
            | ScratchBlock::OpMAbs(_)
            | ScratchBlock::OpMSin(_)
            | ScratchBlock::OpMCos(_)
            | ScratchBlock::OpMTan(_)
            | ScratchBlock::MotionGetX
            | ScratchBlock::MotionGetY
            | ScratchBlock::SensingDaysSince2000
            | ScratchBlock::OpStrLen(_) => Some(VariableWrite::skip_nan(VarTypeChecked::Number)),
            ScratchBlock::OpStrLetterOf(_, _) | ScratchBlock::OpStrJoin(_, _) => {
                Some(VariableWrite::normal(VarTypeChecked::String))
            }
            ScratchBlock::OpBAnd(_, _)
            | ScratchBlock::OpBNot(_)
            | ScratchBlock::OpBOr(_, _)
            | ScratchBlock::OpStrContains(_, _)
            | ScratchBlock::OpCmp(_, _, _) => Some(VariableWrite::normal(VarTypeChecked::Bool)),
            ScratchBlock::VarSet(_, _)
            | ScratchBlock::VarChange(_, _)
            | ScratchBlock::ControlIf(_, _)
            | ScratchBlock::ControlIfElse(_, _, _)
            | ScratchBlock::ControlRepeat(_, _)
            | ScratchBlock::ControlForever(_)
            | ScratchBlock::ScreenRefresh
            | ScratchBlock::ControlStopThisScript
            | ScratchBlock::FunctionCallNoScreenRefresh(_, _)
            | ScratchBlock::FunctionCallScreenRefresh(_, _)
            | ScratchBlock::MotionGoToXY(_, _)
            | ScratchBlock::MotionChangeX(_)
            | ScratchBlock::MotionChangeY(_)
            | ScratchBlock::MotionSetX(_)
            | ScratchBlock::MotionSetY(_)
            | ScratchBlock::ControlRepeatUntil(_, _)
            | ScratchBlock::LooksShown(_)
            | ScratchBlock::Log(_) => None,
        }
    }

    #[must_use]
    pub fn could_trigger_refresh(&self) -> bool {
        match self {
            ScratchBlock::VarSet(_, _)
            | ScratchBlock::VarChange(_, _)
            | ScratchBlock::VarRead(_)
            | ScratchBlock::OpAdd(_, _)
            | ScratchBlock::OpSub(_, _)
            | ScratchBlock::OpMul(_, _)
            | ScratchBlock::OpDiv(_, _)
            | ScratchBlock::OpRound(_)
            | ScratchBlock::OpStrJoin(_, _)
            | ScratchBlock::OpMod(_, _)
            | ScratchBlock::OpStrLen(_)
            | ScratchBlock::OpBAnd(_, _)
            | ScratchBlock::OpBNot(_)
            | ScratchBlock::OpBOr(_, _)
            | ScratchBlock::OpMFloor(_)
            | ScratchBlock::OpMAbs(_)
            | ScratchBlock::OpMSqrt(_)
            | ScratchBlock::OpMSin(_)
            | ScratchBlock::OpMCos(_)
            | ScratchBlock::OpMTan(_)
            | ScratchBlock::OpCmp(_, _, _)
            | ScratchBlock::OpRandom(_, _)
            | ScratchBlock::OpStrLetterOf(_, _)
            | ScratchBlock::OpStrContains(_, _)
            | ScratchBlock::ControlStopThisScript
            | ScratchBlock::FunctionGetArg(_)
            | ScratchBlock::SensingDaysSince2000
            | ScratchBlock::FunctionCallNoScreenRefresh(_, _)
            | ScratchBlock::MotionGetX
            | ScratchBlock::MotionGetY => false,

            ScratchBlock::ControlIf(_, blocks)
            | ScratchBlock::ControlRepeatUntil(_, blocks)
            | ScratchBlock::ControlForever(blocks)
            | ScratchBlock::ControlRepeat(_, blocks) => {
                blocks.iter().any(ScratchBlock::could_trigger_refresh)
            }
            ScratchBlock::ControlIfElse(_, blocks_then, blocks_else) => {
                blocks_then.iter().any(ScratchBlock::could_trigger_refresh)
                    || blocks_else.iter().any(ScratchBlock::could_trigger_refresh)
            }
            ScratchBlock::LooksShown(b) => *b, // Triggers refresh on show, not hide

            // TODO: This actually depends on the function itself
            // Remove from here eventually
            ScratchBlock::FunctionCallScreenRefresh(_, _) => true,

            ScratchBlock::ScreenRefresh
            | ScratchBlock::Log(_)
            | ScratchBlock::MotionGoToXY(_, _)
            | ScratchBlock::MotionChangeX(_)
            | ScratchBlock::MotionChangeY(_)
            | ScratchBlock::MotionSetX(_)
            | ScratchBlock::MotionSetY(_) => true,
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum VarType {
    Number,
    Bool,
    String,
}

pub struct Compiler<'compiler> {
    pub args_list: Vec<[Value; 4]>,
    pub constants: ConstantMap,
    pub break_counter: usize,
    pub break_points: Vec<Block>,
    pub memory: &'compiler [ScratchObject],
    pub program_analysis: Effects,
    pub func_store: FunctionStore,
    pub call_conv: CallConv,
    pub static_strings: Vec<SmolStr>,
    pub vars: Box<dyn VarStore>,
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

    /// A [`Value`] of `*mut Vec<LoopFrame>` representing the stack
    /// of loops. This is a **compile-time handle to a runtime
    /// value**. See `docs/JIT_SIGNATURE.md` for more info.
    pub loop_stack_ptr: Value,
    pub script_ptr: Value,
    pub graphics_ptr: Value,
    pub child_thread_ptr: Value,

    pub sprite_id: SpriteId,
    pub is_screen_refresh: bool,
    pub is_called_as_refresh: Value,
}

impl<'a> Compiler<'a> {
    pub fn new(
        jump_from_existing: bool,
        builder: &mut FunctionBuilder<'_>,
        code: &[ScratchBlock],
        memory: &'a [ScratchObject],
        loop_stack_ptr: Value,
        script_ptr: Value,
        graphics_ptr: Value,
        args_list: Vec<[Value; 4]>,
        sprite_id: SpriteId,
        is_screen_refresh: bool,
        is_called_as_refresh: Value,
        child_thread_ptr: Value,
        temp_slot4: (Value, StackSlot),
        func_map: FuncMap,
        call_conv: CallConv,
    ) -> Self {
        let mut constants = ConstantMap::new();

        let code_block = builder.create_block();
        if jump_from_existing {
            builder.ins().jump(code_block, &[]);
        }
        builder.switch_to_block(code_block);

        let mut other_eff: Option<Effects> = None;
        let var_ty = &|_| VariableWrite::default();
        let mut program_analysis = code.effects(&mut |_| Effects::unknown(), var_ty, &mut |e| {
            if let Some(other) = &mut other_eff {
                other.merge(&e, var_ty);
            } else {
                other_eff = Some(e);
            }
        });
        if let Some(other_eff) = other_eff {
            program_analysis.merge(&other_eff, var_ty);
        }

        let vars: Box<dyn VarStore> = if program_analysis.is_unknown {
            Box::new(GenericVarStore::new(memory))
        } else {
            Box::new(SsaVarStore::new(
                builder,
                &program_analysis,
                &mut constants,
                memory,
            ))
        };

        Self {
            temp_slot4,
            vars,
            program_analysis,
            constants,
            call_conv,
            break_points: vec![code_block],
            func_store: FunctionStore::new(func_map),
            break_counter: 0,
            repeat_stack: 0,
            static_strings: Vec::new(),
            memory,
            script_ptr,
            loop_stack_ptr,
            args_list,
            graphics_ptr,
            sprite_id,
            is_screen_refresh,
            is_called_as_refresh,
            child_thread_ptr,
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
            ScratchBlock::OpAdd(a, b) => {
                return Some(ScratchValue::Num(self.op_add(a, b, builder)));
            }
            ScratchBlock::OpSub(a, b) => {
                return Some(ScratchValue::Num(self.op_sub(a, b, builder)));
            }
            ScratchBlock::OpMul(a, b) => {
                return Some(ScratchValue::Num(self.op_mul(a, b, builder)));
            }
            ScratchBlock::OpDiv(a, b) => {
                return Some(ScratchValue::Num(self.op_div(a, b, builder)));
            }
            ScratchBlock::OpMod(a, b) => {
                return Some(ScratchValue::Num(self.op_modulo(a, b, builder)));
            }
            ScratchBlock::VarRead(ptr) => {
                return Some(self.var_read(builder, *ptr));
            }
            ScratchBlock::OpStrJoin(a, b) => {
                return Some(ScratchValue::Object(self.op_str_join(a, b, builder)));
            }
            ScratchBlock::Log(msg) => self.dbg_log(msg, builder),
            ScratchBlock::ControlRepeat(input, vec) => {
                self.control_repeat(builder, input, vec);
            }
            ScratchBlock::ControlForever(vec) => {
                self.control_forever(builder, vec);
            }
            ScratchBlock::VarChange(ptr, input) => {
                self.var_change(input, builder, *ptr);
            }
            ScratchBlock::ControlIf(input, vec) => {
                self.control_if(input, builder, vec);
            }
            ScratchBlock::ControlIfElse(condition, then_block, else_block) => {
                self.control_if_else(condition, builder, then_block, else_block);
            }
            ScratchBlock::ControlRepeatUntil(input, vec) => {
                self.control_repeat_until(builder, input, vec);
            }
            ScratchBlock::OpCmp(a, b, ordering) => {
                return Some(ScratchValue::Bool(self.op_cmp(a, b, builder, *ordering)));
            }
            ScratchBlock::OpStrLen(input) => {
                return Some(self.op_str_len(input, builder));
            }
            ScratchBlock::OpRandom(a, b) => return Some(self.op_random(a, b, builder)),
            ScratchBlock::OpBAnd(a, b) => {
                return Some(ScratchValue::Bool(self.op_b_and(a, b, builder)));
            }
            ScratchBlock::OpBNot(a) => {
                return Some(ScratchValue::Bool(self.op_b_not(a, builder)));
            }
            ScratchBlock::OpBOr(a, b) => {
                return Some(ScratchValue::Bool(self.op_b_or(a, b, builder)));
            }
            ScratchBlock::OpMFloor(n) => return Some(self.op_m_floor(n, builder)),
            ScratchBlock::OpStrLetterOf(letter, string) => {
                return Some(ScratchValue::Object(
                    self.op_str_letter(letter, string, builder),
                ));
            }
            ScratchBlock::OpStrContains(string, pattern) => {
                return Some(ScratchValue::Bool(
                    self.op_str_contains(string, pattern, builder),
                ));
            }
            ScratchBlock::OpRound(num) => {
                return Some(ScratchValue::Num(self.op_round(num, builder)));
            }
            ScratchBlock::OpMAbs(num) => {
                return Some(ScratchValue::Num(self.op_m_abs(num, builder)));
            }
            ScratchBlock::OpMSqrt(num) => {
                return Some(ScratchValue::Num(self.op_m_sqrt(num, builder)));
            }
            ScratchBlock::OpMSin(num) => {
                return Some(ScratchValue::Num(self.op_m_sin(num, builder)));
            }
            ScratchBlock::OpMCos(num) => {
                return Some(ScratchValue::Num(self.op_m_cos(num, builder)));
            }
            ScratchBlock::OpMTan(num) => {
                return Some(ScratchValue::Num(self.op_m_tan(num, builder)));
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
                return Some(ScratchValue::Object(
                    self.custom_block_get_arg(builder, *idx),
                ));
            }
            ScratchBlock::MotionGoToXY(x, y) => {
                let x = x.get_number(self, builder);
                let y = y.get_number(self, builder);

                let id = self.constants.get_int(self.sprite_id.0, builder);

                self.call_function_indirect(
                    builder,
                    RunState::c_go_to as *const (),
                    &[I64, I64, F64, F64],
                    &[],
                    &[self.graphics_ptr, id, x, y],
                );
            }
            ScratchBlock::MotionChangeX(x) => {
                let x = x.get_number(self, builder);

                let id = self.constants.get_int(self.sprite_id.0, builder);

                self.call_function_indirect(
                    builder,
                    RunState::c_change_x as *const (),
                    &[I64, I64, F64],
                    &[],
                    &[self.graphics_ptr, id, x],
                );
            }
            ScratchBlock::MotionChangeY(y) => {
                let y = y.get_number(self, builder);

                let id = self.constants.get_int(self.sprite_id.0, builder);

                self.call_function_indirect(
                    builder,
                    RunState::c_change_y as *const (),
                    &[I64, I64, F64],
                    &[],
                    &[self.graphics_ptr, id, y],
                );
            }
            ScratchBlock::MotionSetX(x) => {
                let x = x.get_number(self, builder);

                let id = self.constants.get_int(self.sprite_id.0, builder);

                self.call_function_indirect(
                    builder,
                    RunState::c_set_x as *const (),
                    &[I64, I64, F64],
                    &[],
                    &[self.graphics_ptr, id, x],
                );
            }
            ScratchBlock::MotionSetY(y) => {
                let y = y.get_number(self, builder);

                let id = self.constants.get_int(self.sprite_id.0, builder);

                self.call_function_indirect(
                    builder,
                    RunState::c_set_y as *const (),
                    &[I64, I64, F64],
                    &[],
                    &[self.graphics_ptr, id, y],
                );
            }
            ScratchBlock::MotionGetX => {
                let id = self.constants.get_int(self.sprite_id.0, builder);

                let inst = self.call_function_indirect(
                    builder,
                    RunState::c_get_x as *const (),
                    &[I64, I64],
                    &[F64],
                    &[self.graphics_ptr, id],
                );

                let val = builder.inst_results(inst)[0];
                return Some(ScratchValue::Num(val));
            }
            ScratchBlock::MotionGetY => {
                let id = self.constants.get_int(self.sprite_id.0, builder);

                let inst = self.call_function_indirect(
                    builder,
                    RunState::c_get_y as *const (),
                    &[I64, I64],
                    &[F64],
                    &[self.graphics_ptr, id],
                );

                let val = builder.inst_results(inst)[0];
                return Some(ScratchValue::Num(val));
            }
            ScratchBlock::LooksShown(shown) => {
                let id = self.constants.get_int(self.sprite_id.0, builder);
                let shown = self.constants.get_int(i64::from(*shown), builder);

                self.call_function_indirect(
                    builder,
                    RunState::c_shown as *const (),
                    &[I64, I64, I64],
                    &[],
                    &[self.graphics_ptr, id, shown],
                );
            }
            ScratchBlock::SensingDaysSince2000 => {
                let inst =
                    self.call_function(builder, callbacks::env::DAYS_SINCE_2000, &[], &[F64], &[]);
                let val = builder.inst_results(inst)[0];
                return Some(ScratchValue::Num(val));
            }
        }
        None
    }

    fn custom_block_get_arg(
        &mut self,
        builder: &mut FunctionBuilder<'_>,
        idx: usize,
    ) -> [Value; 4] {
        let [i1, i2, i3, i4] = self.args_list[idx];

        self.call_function(
            builder,
            callbacks::types::CLONE_OBJ,
            &[I64, I64, I64, I64, I64],
            &[],
            &[i1, i2, i3, i4, self.temp_slot4.0],
        );

        let slot = self.temp_slot4.1;
        let i1 = builder.ins().stack_load(I64, slot, 0);
        let i2 = builder.ins().stack_load(I64, slot, 8);
        let i3 = builder.ins().stack_load(I64, slot, 16);
        let i4 = builder.ins().stack_load(I64, slot, 24);
        [i1, i2, i3, i4]
    }

    pub fn screen_refresh(&mut self, builder: &mut FunctionBuilder<'_>) {
        if !self.is_screen_refresh {
            return;
        }

        for _ in 0..2 {
            // TODO: Hacky workaround for too-fast timing
            self.break_counter += 1;
            self.vars.save(builder, &mut self.constants, self.memory);
            let break_counter = self.constants.get_int(self.break_counter as i64, builder);

            builder.ins().return_(&[break_counter]);
            self.constants.clear();

            let b = builder.create_block();
            self.break_points.push(b);
            builder.switch_to_block(b);

            self.vars.reinit(builder, &mut self.constants, self.memory);
        }
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
            panic!("Function not found: {}", name);
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
