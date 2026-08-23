use std::collections::{HashMap, HashSet};

use cranelift::{
    codegen::ir::{
        Block, InstBuilder,
        types::{F64, I64},
    },
    frontend::FunctionBuilder,
};

use crate::{
    Input, Ptr, ScratchBlock,
    compiler::{VarType, VarTypeChecked},
    input_primitives::ScratchValue,
    runtime::CustomBlockId,
    variable_storage::{VarStore, VariableSlot},
};

#[cfg(test)]
mod tests;

#[derive(Clone, Copy, Debug)]
pub struct VariableWrite {
    pub ty: VarTypeChecked,
    pub skip_nan: bool,
    pub direct: bool,
}

impl Default for VariableWrite {
    fn default() -> Self {
        Self::normal(VarTypeChecked::Object)
    }
}

impl VariableWrite {
    pub fn normal(ty: VarTypeChecked) -> Self {
        Self {
            ty,
            skip_nan: false,
            direct: true,
        }
    }

    pub fn skip_nan(ty: VarTypeChecked) -> Self {
        Self {
            ty,
            skip_nan: true,
            direct: true,
        }
    }

    pub fn merge(self, other: Self) -> Self {
        if self.ty == other.ty {
            Self {
                ty: self.ty,
                skip_nan: self.skip_nan && other.skip_nan,
                direct: self.direct || other.direct,
            }
        } else {
            Self::default()
        }
    }
}

impl From<VariableWrite> for VarTypeChecked {
    fn from(val: VariableWrite) -> Self {
        val.ty
    }
}

impl From<VariableWrite> for Option<VarType> {
    fn from(val: VariableWrite) -> Self {
        val.ty.into()
    }
}

impl From<VariableSlot> for VariableWrite {
    fn from(val: VariableSlot) -> Self {
        let ty = match val.val {
            ScratchValue::Num(_) => VarTypeChecked::Number,
            ScratchValue::Bool(_) => VarTypeChecked::Bool,
            ScratchValue::String(_) => VarTypeChecked::String,
            ScratchValue::Object(_) => VarTypeChecked::Object,
        };
        Self {
            ty,
            skip_nan: val.skip_nan,
            direct: true,
        }
    }
}

#[derive(Clone, Debug)]
pub struct Effects {
    pub reads: HashSet<Ptr>,
    pub writes: HashMap<Ptr, VariableWrite>,
    pub is_unknown: bool,
    pub yields: bool,

    pub may_not_happen: bool,
}

impl Effects {
    pub fn new() -> Self {
        Effects {
            reads: HashSet::new(),
            writes: HashMap::new(),
            is_unknown: false,
            yields: false,
            may_not_happen: false,
        }
    }

    /// An [`Effects`] that represents that *anything* could happen.
    ///
    /// For example, if you do a screen refresh, the variables could change to
    /// *anything* during that time (impossible to analyze),
    /// so we use `unknown` to represent that.
    pub fn unknown() -> Self {
        Effects {
            is_unknown: true,
            ..Self::new()
        }
    }

    pub fn that_yields() -> Self {
        Effects {
            yields: true,
            ..Self::unknown()
        }
    }

    pub fn with_write(ptr: Ptr, ty: VariableWrite) -> Self {
        let mut effects = Effects::new();
        effects.writes = HashMap::from([(ptr, ty)]);
        effects
    }

    pub fn with_read(ptr: Ptr) -> Self {
        let mut effects = Effects::new();
        effects.reads = HashSet::from([ptr]);
        effects
    }

    pub fn sequence(&mut self, other: &Effects, var_type: &dyn Fn(Ptr) -> VariableWrite) {
        if other.may_not_happen {
            self.merge(other, var_type);
            return;
        }

        self.union_reads(other);

        for (ptr, write) in other.writes.iter() {
            if self
                .writes
                .get(ptr)
                .is_some_and(|old| old.direct && !write.direct)
            {
                // Can't downgrade a write!
                continue;
            }
            self.writes.insert(*ptr, *write);
        }
    }

    pub fn then(mut self, other: Effects, var_type: &dyn Fn(Ptr) -> VariableWrite) -> Self {
        self.sequence(&other, var_type);
        self
    }

    fn union_reads(&mut self, other: &Effects) {
        if other.is_unknown {
            self.is_unknown = true;
        }
        if other.yields {
            self.yields = true;
        }

        self.reads.extend(other.reads.iter().copied());
    }

    pub fn merge(&mut self, other: &Effects, var_type: &dyn Fn(Ptr) -> VariableWrite) {
        self.union_reads(other);

        // There are two scenarios we may be dealing with
        //
        //  1) other.may_not_happen:
        //
        //     [PreviousState(var_type)]
        //           │
        //           ▼
        //        [self]
        //        │    │
        //        │  [other]
        //        │    │
        //        ▼    ▼
        //     ┌──────────────┐
        //     │    Result    │
        //     └──────────────┘
        //
        //  2) !other.may_not_happen:
        //
        //     [PreviousState(var_type)]
        //        │        │
        //        ▼        ▼
        //      [self]  [other]
        //        │        │
        //        ▼        ▼
        //     ┌──────────────┐
        //     │    Result    │
        //     └──────────────┘

        // 1. Process keys that exist in BOTH branches
        for (ptr, ty) in &mut self.writes {
            if let Some(other_ty) = other.writes.get(ptr) {
                *ty = ty.merge(*other_ty);
            } else if !other.may_not_happen {
                // Written in `self`, but missing in guaranteed `other` path
                *ty = ty.merge(var_type(*ptr));
            }
        }

        // 2. Process keys that exist ONLY in `other`
        for (ptr, var) in &other.writes {
            if !self.writes.contains_key(ptr) {
                let old_var = var_type(*ptr);
                self.writes.insert(*ptr, var.merge(old_var));
            }
        }
    }

    pub fn or(mut self, other: Effects, var_type: &dyn Fn(Ptr) -> VariableWrite) -> Self {
        self.merge(&other, var_type);
        self
    }

    pub fn generate_params(
        &self,
        builder: &mut FunctionBuilder,
        block: Block,
        vars: &dyn VarStore,
    ) -> Vec<(Ptr, VariableSlot)> {
        let mut param_values = Vec::new();

        let izero = (!vars.uses_block_params()).then(|| builder.ins().iconst(I64, 0));
        let fzero = (!vars.uses_block_params()).then(|| builder.ins().f64const(0.0));

        for (ptr, ty) in &self.writes {
            let original = vars.get_type(*ptr);
            if !ty.direct {
                continue;
            }

            let skip_nan = ty.skip_nan && original.skip_nan;
            let ty = if original.ty == ty.ty {
                VariableWrite {
                    ty: ty.ty,
                    skip_nan,
                    direct: original.direct,
                }
            } else {
                // Downgrade if it diverges
                VariableWrite::default()
            };

            match ty.ty {
                VarTypeChecked::Number => {
                    let value = fzero.unwrap_or_else(|| builder.append_block_param(block, F64));
                    param_values.push((
                        *ptr,
                        VariableSlot {
                            val: ScratchValue::Num(value),
                            skip_nan,
                        },
                    ));
                }
                VarTypeChecked::Bool => {
                    let value = izero.unwrap_or_else(|| builder.append_block_param(block, I64));
                    param_values.push((
                        *ptr,
                        VariableSlot {
                            val: ScratchValue::Bool(value),
                            skip_nan,
                        },
                    ));
                }
                VarTypeChecked::String => {
                    let vals = [
                        izero.unwrap_or_else(|| builder.append_block_param(block, I64)),
                        izero.unwrap_or_else(|| builder.append_block_param(block, I64)),
                        izero.unwrap_or_else(|| builder.append_block_param(block, I64)),
                    ];
                    param_values.push((
                        *ptr,
                        VariableSlot {
                            val: ScratchValue::String(vals),
                            skip_nan,
                        },
                    ));
                }
                VarTypeChecked::Object => {
                    let vals = [
                        izero.unwrap_or_else(|| builder.append_block_param(block, I64)),
                        izero.unwrap_or_else(|| builder.append_block_param(block, I64)),
                        izero.unwrap_or_else(|| builder.append_block_param(block, I64)),
                        izero.unwrap_or_else(|| builder.append_block_param(block, I64)),
                    ];
                    param_values.push((
                        *ptr,
                        VariableSlot {
                            val: ScratchValue::Object(vals),
                            skip_nan,
                        },
                    ));
                }
            }
        }

        param_values
    }

    pub fn set_direct(&mut self, direct: bool) {
        for w in self.writes.values_mut() {
            w.direct = direct;
        }
    }
}

pub trait CheckEffects {
    fn effects(
        &self,
        block_eff: &mut impl FnMut(CustomBlockId) -> Effects,
        var_ty: &dyn Fn(Ptr) -> VariableWrite,
        throw_early_ret: &mut dyn FnMut(Effects),
        throw_call_func: &mut dyn FnMut(CustomBlockId, Effects),
    ) -> Effects;
}

impl CheckEffects for ScratchBlock {
    fn effects(
        &self,
        c: &mut impl FnMut(CustomBlockId) -> Effects,
        v: &dyn Fn(Ptr) -> VariableWrite,
        e: &mut dyn FnMut(Effects),
        f: &mut dyn FnMut(CustomBlockId, Effects),
    ) -> Effects {
        match self {
            ScratchBlock::VarSet(ptr, input) => Effects::with_write(*ptr, input.expected_type(v)),
            ScratchBlock::VarChange(ptr, _) => Effects::with_write(
                *ptr,
                VariableWrite {
                    ty: VarTypeChecked::Number,
                    skip_nan: true,
                    direct: true,
                },
            ),
            ScratchBlock::VarRead(ptr) => Effects::with_read(*ptr),
            ScratchBlock::MotionGoToXY(a, b)
            | ScratchBlock::OpAdd(a, b)
            | ScratchBlock::OpSub(a, b)
            | ScratchBlock::OpMul(a, b)
            | ScratchBlock::OpDiv(a, b)
            | ScratchBlock::OpStrJoin(a, b)
            | ScratchBlock::OpMod(a, b)
            | ScratchBlock::OpRandom(a, b)
            | ScratchBlock::OpStrLetterOf(a, b)
            | ScratchBlock::OpStrContains(a, b)
            | ScratchBlock::OpBAnd(a, b)
            | ScratchBlock::OpCmp(a, b, _)
            | ScratchBlock::OpBOr(a, b) => a.effects(c, v, e, f).then(b.effects(c, v, e, f), v),

            ScratchBlock::MotionChangeX(input)
            | ScratchBlock::MotionChangeY(input)
            | ScratchBlock::MotionSetX(input)
            | ScratchBlock::MotionSetY(input)
            | ScratchBlock::Log(input)
            | ScratchBlock::OpRound(input)
            | ScratchBlock::OpStrLen(input)
            | ScratchBlock::OpBNot(input)
            | ScratchBlock::OpMFloor(input)
            | ScratchBlock::OpMAbs(input)
            | ScratchBlock::OpMSqrt(input)
            | ScratchBlock::OpMSin(input)
            | ScratchBlock::OpMCos(input)
            | ScratchBlock::OpMTan(input) => input.effects(c, v, e, f),

            ScratchBlock::ControlRepeatUntil(input, blocks)
            | ScratchBlock::ControlRepeat(input, blocks)
            | ScratchBlock::ControlIf(input, blocks) => {
                let mut final_effects = input
                    .effects(c, v, e, f)
                    .then(blocks.effects(c, v, e, f), v);
                // Setting input.effects to may_not_happen is fine,
                // since input would be read-only and this only affects writes
                final_effects.may_not_happen = true;
                final_effects
            }
            ScratchBlock::ControlForever(blocks) => {
                // It's guaranteed to run at least once
                blocks.effects(c, v, e, f)
            }
            ScratchBlock::ControlIfElse(input, b1, b2) => {
                let b1 = b1.effects(c, v, e, f);
                let b2 = b2.effects(c, v, e, f);
                input.effects(c, v, e, f).then(b1.or(b2, v), v)
            }

            ScratchBlock::ControlStopThisScript => {
                // Propagate early-return upwards
                e(Effects::new());
                Effects::new()
            }

            ScratchBlock::FunctionCallNoScreenRefresh(id, inputs) => {
                f(*id, Effects::new());
                let mut code_eff = c(*id);
                code_eff.set_direct(false);
                inputs.effects(c, v, e, f).then(code_eff, v)
            }
            // Screen refresh blocks could yield
            ScratchBlock::FunctionCallScreenRefresh(id, _) => {
                f(*id, Effects::new());
                Effects::that_yields()
            }
            ScratchBlock::ScreenRefresh => Effects::that_yields(),

            ScratchBlock::FunctionGetArg(_)
            | ScratchBlock::LooksShown(_)
            | ScratchBlock::MotionGetX
            | ScratchBlock::MotionGetY
            | ScratchBlock::SensingDaysSince2000 => Effects::new(),
        }
    }
}

impl<T: CheckEffects> CheckEffects for Vec<T> {
    fn effects(
        &self,
        c: &mut impl FnMut(CustomBlockId) -> Effects,
        v: &dyn Fn(Ptr) -> VariableWrite,
        e: &mut dyn FnMut(Effects),
        f: &mut dyn FnMut(CustomBlockId, Effects),
    ) -> Effects {
        self.as_slice().effects(c, v, e, f)
    }
}

impl<T: CheckEffects> CheckEffects for &[T] {
    fn effects(
        &self,
        c: &mut impl FnMut(CustomBlockId) -> Effects,
        v: &dyn Fn(Ptr) -> VariableWrite,
        e: &mut dyn FnMut(Effects),
        f: &mut dyn FnMut(CustomBlockId, Effects),
    ) -> Effects {
        let mut effects = Effects::new();
        for item in *self {
            let e2 = effects.clone();
            let var_ty = &|var| {
                // Check if variable has been written to within this scope.
                // If not (unwrap_or_else), check globally outside of the scope.
                e2.writes.get(&var).copied().unwrap_or_else(|| v(var))
            };
            let other = item.effects(
                c,
                var_ty,
                &mut |thrown_effect| {
                    // If a function returns early, the thrown effect is propagated.
                    e(effects.clone().then(thrown_effect, var_ty));
                },
                &mut |id, thrown_effect| {
                    f(id, effects.clone().then(thrown_effect, var_ty));
                },
            );
            effects.sequence(&other, var_ty);
        }
        debug_assert!(
            !effects.may_not_happen,
            "Effects of a block list should not be may_not_happen"
        );
        effects
    }
}

impl CheckEffects for Input {
    fn effects(
        &self,
        c: &mut impl FnMut(CustomBlockId) -> Effects,
        v: &dyn Fn(Ptr) -> VariableWrite,
        e: &mut dyn FnMut(Effects),
        f: &mut dyn FnMut(CustomBlockId, Effects),
    ) -> Effects {
        match self {
            Input::Block(block) => block.effects(c, v, e, f),
            Input::Obj(_) => Effects::new(),
        }
    }
}

pub fn analyze(
    code: &impl CheckEffects,
    block_eff: &mut impl FnMut(CustomBlockId) -> Effects,
    throw_call_func: &mut dyn FnMut(CustomBlockId, Effects),
) -> Effects {
    let mut other_eff: Option<Effects> = None;
    let var_ty = &|_| VariableWrite::default();
    let mut effects = code.effects(
        block_eff,
        var_ty,
        &mut |e| {
            if let Some(other) = &mut other_eff {
                other.merge(&e, var_ty);
            } else {
                other_eff = Some(e);
            }
        },
        throw_call_func,
    );
    if let Some(other_eff) = other_eff {
        effects.merge(&other_eff, var_ty);
    }
    effects
}
