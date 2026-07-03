use std::collections::{HashMap, HashSet};

use cranelift::{
    codegen::ir::{
        Block,
        types::{F64, I64},
    },
    frontend::FunctionBuilder,
};

use crate::{
    Input, Ptr, ScratchBlock,
    compiler::{VarType, VarTypeChecked},
    input_primitives::ReturnValue,
    runtime::CustomBlockId,
    variable_storage::VariableSlot,
};

#[derive(Default, Clone, Copy)]
pub struct VariableWrite {
    pub ty: VarTypeChecked,
    pub skip_nan: bool,
}

impl VariableWrite {
    pub fn normal(ty: VarTypeChecked) -> Self {
        Self {
            ty,
            skip_nan: false,
        }
    }

    pub fn skip_nan(ty: VarTypeChecked) -> Self {
        Self { ty, skip_nan: true }
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

impl From<&VariableSlot> for VariableWrite {
    fn from(val: &VariableSlot) -> Self {
        let ty = match val.val {
            ReturnValue::Num(_) => VarTypeChecked::Number,
            ReturnValue::Bool(_) => VarTypeChecked::Bool,
            ReturnValue::String(_) => VarTypeChecked::String,
            ReturnValue::Object(_) => VarTypeChecked::Object,
        };
        Self {
            ty,
            skip_nan: val.skip_nan,
        }
    }
}

pub struct Effects {
    pub reads: HashSet<Ptr>,
    pub reads_is_unknown: bool,
    pub writes: HashMap<Ptr, VariableWrite>,
    pub writes_is_unknown: bool,
    pub yields: bool,

    pub may_not_happen: bool,
}

impl Effects {
    pub fn new() -> Self {
        Effects {
            reads: HashSet::new(),
            reads_is_unknown: false,
            writes: HashMap::new(),
            writes_is_unknown: false,
            yields: false,
            may_not_happen: false,
        }
    }

    pub fn unknown() -> Self {
        Effects {
            reads_is_unknown: true,
            writes_is_unknown: true,
            ..Self::new()
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

    pub fn sequence(&mut self, other: Effects) {
        if other.may_not_happen {
            self.merge(other);
            return;
        }

        self.union_reads(&other);

        if other.writes_is_unknown {
            self.writes_is_unknown = true;
        }
        if other.yields {
            self.yields = true;
        }
        self.writes.extend(other.writes);
    }

    fn union_reads(&mut self, other: &Effects) {
        if other.reads_is_unknown {
            self.reads_is_unknown = true;
        }
        self.reads.extend(other.reads.iter().copied());
    }

    pub fn merge(&mut self, other: Effects) {
        self.union_reads(&other);

        if other.writes_is_unknown {
            self.writes_is_unknown = true;
        }
        if other.yields {
            self.yields = true;
        }

        // Just intersect writes, but if the types don't match, set to unknown
        for (ptr, ty) in &mut self.writes {
            if let Some(other_ty) = other.writes.get(ptr) {
                ty.skip_nan = ty.skip_nan && other_ty.skip_nan;
                if ty.ty != other_ty.ty {
                    ty.ty = VarTypeChecked::Object;
                }
            } else if !other.may_not_happen {
                // Branch A wrote to the variable but
                // branch B did not. Remember, this is a
                // merge operator
                *ty = VariableWrite::default();
            }
        }
        for ptr in other.writes.keys() {
            if !self.writes.contains_key(ptr) {
                self.writes.insert(*ptr, VariableWrite::default());
            }
        }
    }

    pub fn generate_params(
        &self,
        builder: &mut FunctionBuilder,
        block: Block,
        original_vals: &impl Fn(Ptr) -> VariableWrite,
    ) -> Vec<(Ptr, VariableSlot)> {
        let mut param_values = Vec::new();

        for (ptr, ty) in &self.writes {
            let original = original_vals(*ptr);
            let skip_nan = ty.skip_nan && original.skip_nan;
            let ty = if original.ty == ty.ty {
                VariableWrite {
                    ty: ty.ty,
                    skip_nan,
                }
            } else {
                // Downgrade if it diverges
                VariableWrite::default()
            };

            match ty.ty {
                VarTypeChecked::Number => {
                    let value = builder.append_block_param(block, F64);
                    param_values.push((
                        *ptr,
                        VariableSlot {
                            val: ReturnValue::Num(value),
                            skip_nan,
                        },
                    ));
                }
                VarTypeChecked::Bool => {
                    let value = builder.append_block_param(block, I64);
                    param_values.push((
                        *ptr,
                        VariableSlot {
                            val: ReturnValue::Bool(value),
                            skip_nan,
                        },
                    ));
                }
                VarTypeChecked::String => {
                    let vals = [
                        builder.append_block_param(block, I64),
                        builder.append_block_param(block, I64),
                        builder.append_block_param(block, I64),
                    ];
                    param_values.push((
                        *ptr,
                        VariableSlot {
                            val: ReturnValue::String(vals),
                            skip_nan,
                        },
                    ));
                }
                VarTypeChecked::Object => {
                    let vals = [
                        builder.append_block_param(block, I64),
                        builder.append_block_param(block, I64),
                        builder.append_block_param(block, I64),
                        builder.append_block_param(block, I64),
                    ];
                    param_values.push((
                        *ptr,
                        VariableSlot {
                            val: ReturnValue::Object(vals),
                            skip_nan,
                        },
                    ));
                }
            }
        }

        param_values
    }
}

impl std::ops::BitOr for Effects {
    type Output = Self;

    fn bitor(mut self, rhs: Self) -> Self::Output {
        self.sequence(rhs);
        self
    }
}

impl std::ops::BitAnd for Effects {
    type Output = Self;

    fn bitand(mut self, rhs: Self) -> Self::Output {
        self.merge(rhs);
        self
    }
}

pub trait CheckEffects {
    fn effects(
        &self,
        block_eff: &mut impl FnMut(CustomBlockId) -> Effects,
        var_ty: &dyn Fn(Ptr) -> VariableWrite,
    ) -> Effects;
}

impl CheckEffects for ScratchBlock {
    fn effects(
        &self,
        c: &mut impl FnMut(CustomBlockId) -> Effects,
        v: &dyn Fn(Ptr) -> VariableWrite,
    ) -> Effects {
        match self {
            ScratchBlock::VarSet(ptr, input) => Effects::with_write(*ptr, input.expected_type(v)),
            ScratchBlock::VarChange(ptr, _) => Effects::with_write(
                *ptr,
                VariableWrite {
                    ty: VarTypeChecked::Number,
                    skip_nan: true,
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
            | ScratchBlock::OpBOr(a, b) => a.effects(c, v) | b.effects(c, v),

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
            | ScratchBlock::OpMTan(input) => input.effects(c, v),

            ScratchBlock::ControlRepeatUntil(input, blocks)
            | ScratchBlock::ControlRepeat(input, blocks)
            | ScratchBlock::ControlIf(input, blocks) => {
                let mut final_effects = input.effects(c, v) | blocks.effects(c, v);
                // Setting input.effects to may_not_happen is fine,
                // since input would be read-only and this only affects writes
                final_effects.may_not_happen = true;
                final_effects
            }
            ScratchBlock::ControlForever(blocks) => {
                // It's guaranteed to run at least once
                blocks.effects(c, v)
            }
            ScratchBlock::ControlIfElse(input, b1, b2) => {
                let b1 = b1.effects(c, v);
                let b2 = b2.effects(c, v);
                input.effects(c, v) | (b1 & b2)
            }
            // Early returns need some more work
            ScratchBlock::ControlStopThisScript => todo!(),

            ScratchBlock::FunctionCallNoScreenRefresh(custom_block_id, inputs)
            | ScratchBlock::FunctionCallScreenRefresh(custom_block_id, inputs) => {
                inputs.effects(c, v) | c(*custom_block_id)
            }
            ScratchBlock::ScreenRefresh => {
                // Not doing Effects::unknown() here,
                // because even though the variables could change to
                // *anything* during this time, all invalidation
                // is *already handled* after the refresh
                let mut e = Effects::new();
                e.yields = true;
                e
            }

            ScratchBlock::FunctionGetArg(_)
            | ScratchBlock::LooksShown(_)
            | ScratchBlock::MotionGetX
            | ScratchBlock::MotionGetY
            | ScratchBlock::ControlDaysSince2000 => Effects::new(),
        }
    }
}

impl<T: CheckEffects> CheckEffects for Vec<T> {
    fn effects(
        &self,
        c: &mut impl FnMut(CustomBlockId) -> Effects,
        v: &dyn Fn(Ptr) -> VariableWrite,
    ) -> Effects {
        self.as_slice().effects(c, v)
    }
}

impl<T: CheckEffects> CheckEffects for &[T] {
    fn effects(
        &self,
        c: &mut impl FnMut(CustomBlockId) -> Effects,
        v: &dyn Fn(Ptr) -> VariableWrite,
    ) -> Effects {
        let mut effects = Effects::new();
        for item in *self {
            effects.sequence(item.effects(c, &|var| {
                effects
                    .writes
                    .get(&var)
                    .map(|n| {
                        if let VarTypeChecked::Object = n.ty {
                            return v(var);
                        }
                        *n
                    })
                    .unwrap_or_default()
            }));
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
    ) -> Effects {
        match self {
            Input::Block(block) => block.effects(c, v),
            Input::Obj(_) => Effects::new(),
        }
    }
}
