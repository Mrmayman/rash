use std::collections::{HashMap, HashSet};

use codegen::ir::StackSlot;
use cranelift::prelude::*;
use types::I64;

use crate::{
    compiler::{Compiler, ScratchBlock},
    data_types::ScratchObject,
    input_primitives::Ptr,
};

pub struct StackCache {
    slot: StackSlot,
    variable_offsets: HashMap<Ptr, usize>,
}

impl StackCache {
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
        compiler: &mut Compiler,
    ) {
        for (ptr, offset) in &self.variable_offsets {
            let ptr = ptr.constant(compiler, builder, memory);
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
        compiler: &mut Compiler,
        memory: &[ScratchObject],
    ) {
        for (ptr, offset) in &self.variable_offsets {
            let ptr = ptr.constant(compiler, builder, memory);
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

    pub fn get(&self, ptr: Ptr, builder: &mut FunctionBuilder) -> (Value, Value, Value, Value) {
        let offset = self.variable_offsets[&ptr] as i32;
        (
            builder.ins().stack_load(I64, self.slot, offset),
            builder.ins().stack_load(I64, self.slot, offset + 8),
            builder.ins().stack_load(I64, self.slot, offset + 16),
            builder.ins().stack_load(I64, self.slot, offset + 24),
        )
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
        ScratchBlock::WhenFlagClicked
        | ScratchBlock::OpAdd(_, _)
        | ScratchBlock::OpSub(_, _)
        | ScratchBlock::OpMul(_, _)
        | ScratchBlock::OpDiv(_, _)
        | ScratchBlock::OpStrJoin(_, _)
        | ScratchBlock::OpMod(_, _)
        | ScratchBlock::OpStrLen(_)
        | ScratchBlock::OpCmpGreater(_, _)
        | ScratchBlock::OpCmpLesser(_, _)
        | ScratchBlock::OpRandom(_, _) => {}
    }
}
