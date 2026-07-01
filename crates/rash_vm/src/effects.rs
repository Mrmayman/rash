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
};

pub struct Effects {
    pub reads: HashSet<Ptr>,
    pub reads_is_unknown: bool,
    pub writes: HashMap<Ptr, VarTypeChecked>,
    pub writes_is_unknown: bool,

    pub may_not_happen: bool,
}

impl Effects {
    pub fn new() -> Self {
        Effects {
            reads: HashSet::new(),
            reads_is_unknown: false,
            writes: HashMap::new(),
            writes_is_unknown: false,
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

    pub fn with_write(ptr: Ptr, ty: VarTypeChecked) -> Self {
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

        // Just intersect writes, but if the types don't match, set to unknown
        for (ptr, ty) in self.writes.iter_mut() {
            if let Some(other_ty) = other.writes.get(ptr) {
                if ty != other_ty {
                    *ty = VarTypeChecked::Object;
                }
            } else if !other.may_not_happen {
                *ty = VarTypeChecked::Object;
            }
        }
        for ptr in other.writes.keys() {
            if !self.writes.contains_key(ptr) {
                self.writes.insert(*ptr, VarTypeChecked::Object);
            }
        }
    }

    pub fn generate_params(
        &self,
        builder: &mut FunctionBuilder,
        block: Block,
        original_vals: &impl Fn(Ptr) -> VarTypeChecked,
    ) -> Vec<(Ptr, ReturnValue)> {
        let mut param_values = Vec::new();

        for (ptr, ty) in self.writes.iter() {
            let ty = if original_vals(*ptr) == *ty {
                *ty
            } else {
                // Downgrade if it diverges
                VarTypeChecked::Object
            };

            match ty {
                VarTypeChecked::Number => {
                    let value = builder.append_block_param(block, F64);
                    param_values.push((*ptr, ReturnValue::Num(value)));
                }
                VarTypeChecked::Bool => {
                    let value = builder.append_block_param(block, I64);
                    param_values.push((*ptr, ReturnValue::Bool(value)));
                }
                VarTypeChecked::String => {
                    let vals = [
                        builder.append_block_param(block, I64),
                        builder.append_block_param(block, I64),
                        builder.append_block_param(block, I64),
                    ];
                    param_values.push((*ptr, ReturnValue::String(vals)));
                }
                VarTypeChecked::Object => {
                    let vals = [
                        builder.append_block_param(block, I64),
                        builder.append_block_param(block, I64),
                        builder.append_block_param(block, I64),
                        builder.append_block_param(block, I64),
                    ];
                    param_values.push((*ptr, ReturnValue::Object(vals)));
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
        var_ty: &dyn Fn(Ptr) -> Option<VarType>,
    ) -> Effects;
}

impl CheckEffects for ScratchBlock {
    fn effects(
        &self,
        c: &mut impl FnMut(CustomBlockId) -> Effects,
        v: &dyn Fn(Ptr) -> Option<VarType>,
    ) -> Effects {
        match self {
            ScratchBlock::VarSet(ptr, input) => {
                let ty = input.expected_type(|ptr| v(ptr)).into();
                Effects::with_write(*ptr, ty)
            }
            ScratchBlock::VarChange(ptr, _) => {
                let ty = VarTypeChecked::Number;
                Effects::with_write(*ptr, ty)
            }
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
            ScratchBlock::ScreenRefresh => Effects::unknown(),

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
        v: &dyn Fn(Ptr) -> Option<VarType>,
    ) -> Effects {
        self.as_slice().effects(c, v)
    }
}

impl<T: CheckEffects> CheckEffects for &[T] {
    fn effects(
        &self,
        c: &mut impl FnMut(CustomBlockId) -> Effects,
        v: &dyn Fn(Ptr) -> Option<VarType>,
    ) -> Effects {
        let mut effects = Effects::new();
        for item in *self {
            effects.sequence(item.effects(c, &|var| {
                effects.writes.get(&var).and_then(|n| {
                    let n: Option<VarType> = (*n).into();
                    n.or_else(|| v(var))
                })
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
        v: &dyn Fn(Ptr) -> Option<VarType>,
    ) -> Effects {
        match self {
            Input::Block(block) => block.effects(c, v),
            Input::Obj(_) => Effects::new(),
        }
    }
}
