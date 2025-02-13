use std::collections::{HashMap, HashSet};

use cranelift::{
    codegen::ir::StackSlot,
    prelude::{
        types::{F64, I64},
        FunctionBuilder, InstBuilder, MemFlags, StackSlotData, StackSlotKind, Value,
    },
};

use crate::{
    compiler::ScratchBlock,
    constant_set::ConstantMap,
    data_types::{ScratchObject, ID_BOOL, ID_NUMBER, ID_STRING},
    input_primitives::Ptr,
};

/// A local cache of accessed variables.
///
/// The interpreter by default stores values on the heap,
/// but this is slow. By having a local stack cache that
/// syncs with the real variable storage, we can get
/// a noticeable speedup (`6.8 ms -> 6.3 ms` in pi benchmark)
pub struct StackCache {
    slot: StackSlot,
    variable_offsets: HashMap<Ptr, usize>,
}

impl StackCache {
    /// Creates a new [`StackCache`], containing variables
    /// accessed by the code in the `code` argument.
    ///
    /// # Warning
    /// Call [`StackCache::init()`] before generating the
    /// code, or you will get memory corruption and crashes.
    pub fn new(builder: &mut FunctionBuilder, code: &[ScratchBlock]) -> Self {
        let mut vars = HashSet::new();
        for block in code {
            accesses_var(block, &mut vars);
        }
        let variable_offsets: HashMap<Ptr, usize> = vars
            .into_iter()
            .enumerate()
            .map(|(i, p)| (p, i * 4 * std::mem::size_of::<i64>()))
            .collect();
        let slot = builder.create_sized_stack_slot(StackSlotData {
            kind: StackSlotKind::ExplicitSlot,
            size: variable_offsets.len() as u32 * 4 * std::mem::size_of::<i64>() as u32,
            align_shift: 0,
        });
        Self {
            slot,
            variable_offsets,
        }
    }

    pub fn init(
        &self,
        builder: &mut FunctionBuilder,
        memory: &[ScratchObject],
        constants: &mut ConstantMap,
    ) {
        for (ptr, offset) in &self.variable_offsets {
            let ptr = ptr.constant(constants, builder, memory);
            let i1 = builder.ins().load(I64, MemFlags::new(), ptr, 0);
            let i2 = builder.ins().load(I64, MemFlags::new(), ptr, 8);
            let i3 = builder.ins().load(I64, MemFlags::new(), ptr, 16);
            let i4 = builder.ins().load(I64, MemFlags::new(), ptr, 24);

            builder.ins().stack_store(i1, self.slot, *offset as i32);
            builder.ins().stack_store(i2, self.slot, *offset as i32 + 8);
            builder
                .ins()
                .stack_store(i3, self.slot, *offset as i32 + 16);
            builder
                .ins()
                .stack_store(i4, self.slot, *offset as i32 + 24);
        }
    }

    pub fn save(
        &self,
        builder: &mut FunctionBuilder,
        constants: &mut ConstantMap,
        memory: &[ScratchObject],
    ) {
        for (ptr, offset) in &self.variable_offsets {
            let ptr = ptr.constant(constants, builder, memory);
            let i1 = builder.ins().stack_load(I64, self.slot, *offset as i32);
            let i2 = builder.ins().stack_load(I64, self.slot, *offset as i32 + 8);
            let i3 = builder
                .ins()
                .stack_load(I64, self.slot, *offset as i32 + 16);
            let i4 = builder
                .ins()
                .stack_load(I64, self.slot, *offset as i32 + 24);

            builder.ins().store(MemFlags::new(), i1, ptr, 0);
            builder.ins().store(MemFlags::new(), i2, ptr, 8);
            builder.ins().store(MemFlags::new(), i3, ptr, 16);
            builder.ins().store(MemFlags::new(), i4, ptr, 24);
        }
    }

    /*pub fn get(&self, ptr: Ptr, builder: &mut FunctionBuilder) -> (Value, Value, Value, Value) {
        let offset = self.variable_offsets[&ptr] as i32;
        (
            builder.ins().stack_load(I64, self.slot, offset),
            builder.ins().stack_load(I64, self.slot, offset + 8),
            builder.ins().stack_load(I64, self.slot, offset + 16),
            builder.ins().stack_load(I64, self.slot, offset + 24),
        )
    }*/

    pub fn store_f64(
        &self,
        ptr: Ptr,
        builder: &mut FunctionBuilder<'_>,
        num: f64,
        constants: &mut ConstantMap,
    ) {
        let mem_ptr = *self.variable_offsets.get(&ptr).unwrap() as i32;

        let num = constants.get_float(num, builder);
        builder.ins().stack_store(num, self.slot, mem_ptr + 8);

        let id = constants.get_int(ID_NUMBER, builder);
        builder.ins().stack_store(id, self.slot, mem_ptr);
    }

    pub fn store_bool(
        &self,
        ptr: Ptr,
        builder: &mut FunctionBuilder<'_>,
        num: bool,
        constants: &mut ConstantMap,
    ) {
        let mem_ptr = *self.variable_offsets.get(&ptr).unwrap() as i32;

        let num = constants.get_int(i64::from(num), builder);
        builder.ins().stack_store(num, self.slot, mem_ptr + 8);

        let id = constants.get_int(ID_BOOL, builder);
        builder.ins().stack_store(id, self.slot, mem_ptr);
    }

    pub fn store_string(
        &self,
        ptr: Ptr,
        builder: &mut FunctionBuilder<'_>,
        string: &str,
        constants: &mut ConstantMap,
    ) {
        let mem_ptr = *self.variable_offsets.get(&ptr).unwrap() as i32;
        let string = string.to_owned();

        // Transmute the String into a [i64; 4] array
        let arr: [i64; 3] = unsafe { std::mem::transmute(string) };
        let i1 = constants.get_int(arr[0], builder);
        let i2 = constants.get_int(arr[1], builder);
        let i3 = constants.get_int(arr[2], builder);

        builder.ins().stack_store(i1, self.slot, mem_ptr + 8);
        builder.ins().stack_store(i2, self.slot, mem_ptr + 16);
        builder.ins().stack_store(i3, self.slot, mem_ptr + 24);

        let id = constants.get_int(ID_STRING, builder);
        builder.ins().stack_store(id, self.slot, mem_ptr);
    }

    pub fn get_ptr(&self, ptr: Ptr, builder: &mut FunctionBuilder<'_>) -> Value {
        let mem_ptr = *self.variable_offsets.get(&ptr).unwrap() as i32;
        builder.ins().stack_addr(I64, self.slot, mem_ptr)
    }

    pub fn store_small_value(
        &self,
        ptr: Ptr,
        builder: &mut FunctionBuilder<'_>,
        value: Value,
        constants: &mut ConstantMap,
        id: i64,
    ) {
        let mem_ptr = *self.variable_offsets.get(&ptr).unwrap() as i32;

        builder.ins().stack_store(value, self.slot, mem_ptr + 8);

        let id = constants.get_int(id, builder);
        builder.ins().stack_store(id, self.slot, mem_ptr);
    }

    pub fn store_object(
        &self,
        ptr: Ptr,
        builder: &mut FunctionBuilder<'_>,
        i1: Value,
        i2: Value,
        i3: Value,
        i4: Value,
    ) {
        let mem_ptr = *self.variable_offsets.get(&ptr).unwrap() as i32;

        builder.ins().stack_store(i1, self.slot, mem_ptr);
        builder.ins().stack_store(i2, self.slot, mem_ptr + 8);
        builder.ins().stack_store(i3, self.slot, mem_ptr + 16);
        builder.ins().stack_store(i4, self.slot, mem_ptr + 24);
    }

    pub fn load_f64(&self, ptr: Ptr, builder: &mut FunctionBuilder<'_>) -> Value {
        let mem_ptr = *self.variable_offsets.get(&ptr).unwrap() as i32;
        builder.ins().stack_load(F64, self.slot, mem_ptr + 8)
    }

    pub fn load_bool(&self, ptr: Ptr, builder: &mut FunctionBuilder<'_>) -> Value {
        let mem_ptr = *self.variable_offsets.get(&ptr).unwrap() as i32;
        builder.ins().stack_load(I64, self.slot, mem_ptr + 8)
    }
}

pub fn accesses_var(block: &ScratchBlock, vars: &mut HashSet<Ptr>) {
    match block {
        ScratchBlock::VarChange(ptr, _)
        | ScratchBlock::VarRead(ptr)
        | ScratchBlock::VarSet(ptr, _) => {
            vars.insert(*ptr);
        }
        ScratchBlock::ControlRepeatUntil(_, vec)
        | ScratchBlock::ControlRepeatScreenRefresh(_, vec)
        | ScratchBlock::ControlRepeat(_, vec)
        | ScratchBlock::ControlIf(_, vec) => {
            for block in vec {
                accesses_var(block, vars);
            }
        }
        ScratchBlock::ControlIfElse(_, vec, vec1) => {
            for block in vec {
                accesses_var(block, vars);
            }
            for block in vec1 {
                accesses_var(block, vars);
            }
        }
        ScratchBlock::OpAdd(_, _)
        | ScratchBlock::OpSub(_, _)
        | ScratchBlock::OpMul(_, _)
        | ScratchBlock::OpDiv(_, _)
        | ScratchBlock::OpStrJoin(_, _)
        | ScratchBlock::OpMod(_, _)
        | ScratchBlock::OpStrLen(_)
        | ScratchBlock::OpCmpGreater(_, _)
        | ScratchBlock::OpCmpLesser(_, _)
        | ScratchBlock::OpBAnd(_, _)
        | ScratchBlock::OpBNot(_)
        | ScratchBlock::OpBOr(_, _)
        | ScratchBlock::OpMFloor(_)
        | ScratchBlock::OpStrLetterOf(_, _)
        | ScratchBlock::OpStrContains(_, _)
        | ScratchBlock::OpRound(_)
        | ScratchBlock::OpMAbs(_)
        | ScratchBlock::OpMSqrt(_)
        | ScratchBlock::OpMSin(_)
        | ScratchBlock::OpMCos(_)
        | ScratchBlock::OpMTan(_)
        | ScratchBlock::ScreenRefresh
        | ScratchBlock::ControlStopThisScript
        | ScratchBlock::FunctionCallNoScreenRefresh(_, _)
        | ScratchBlock::FunctionGetArg(_)
        | ScratchBlock::MotionGoToXY(_, _)
        | ScratchBlock::MotionChangeX(_)
        | ScratchBlock::MotionChangeY(_)
        | ScratchBlock::MotionSetX(_)
        | ScratchBlock::MotionSetY(_)
        | ScratchBlock::MotionGetX
        | ScratchBlock::MotionGetY
        | ScratchBlock::OpRandom(_, _) => {}
    }
}
