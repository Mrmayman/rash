use codegen::ir::Inst;
use cranelift::prelude::*;
use isa::CallConv;
use types::{F64, I64};

use crate::{
    callbacks,
    compiler::MEMORY,
    data_types::{ScratchObject, ID_BOOL, ID_NUMBER, ID_STRING},
    input_primitives::Ptr,
};

pub fn ins_call_to_num(
    builder: &mut FunctionBuilder<'_>,
    i1: Value,
    i2: Value,
    i3: Value,
    i4: Value,
) -> Inst {
    let to_num_func = builder
        .ins()
        .iconst(I64, callbacks::types::to_number as i64);
    let sig = builder.import_signature({
        let mut sig = Signature::new(CallConv::SystemV);
        sig.params.push(AbiParam::new(I64));
        sig.params.push(AbiParam::new(I64));
        sig.params.push(AbiParam::new(I64));
        sig.params.push(AbiParam::new(I64));
        sig.returns.push(AbiParam::new(F64));
        sig
    });
    builder
        .ins()
        .call_indirect(sig, to_num_func, &[i1, i2, i3, i4])
}

pub fn ins_mem_write_string(string: &str, builder: &mut FunctionBuilder<'_>, ptr: Ptr) {
    let string = string.to_owned();

    // Transmute the String into a [i64; 4] array
    let arr: [i64; 3] = unsafe { std::mem::transmute(string) };
    let i1 = builder.ins().iconst(I64, arr[0]);
    let i2 = builder.ins().iconst(I64, arr[1]);
    let i3 = builder.ins().iconst(I64, arr[2]);

    let mem_ptr = builder.ins().iconst(
        I64,
        MEMORY.as_ptr() as i64 + (ptr.0 * std::mem::size_of::<ScratchObject>()) as i64,
    );

    builder.ins().store(MemFlags::new(), i1, mem_ptr, 8);
    builder.ins().store(MemFlags::new(), i2, mem_ptr, 16);
    builder.ins().store(MemFlags::new(), i3, mem_ptr, 24);

    let id = builder.ins().iconst(I64, ID_STRING as i64);
    builder.ins().store(MemFlags::new(), id, mem_ptr, 0);
}

pub fn ins_mem_write_bool(builder: &mut FunctionBuilder<'_>, ptr: Ptr, num: bool) {
    let mem_ptr = builder.ins().iconst(
        I64,
        MEMORY.as_ptr() as i64 + (ptr.0 * std::mem::size_of::<ScratchObject>()) as i64,
    );

    let num = builder.ins().iconst(I64, num as i64);
    builder.ins().store(MemFlags::new(), num, mem_ptr, 8);

    let id = builder.ins().iconst(I64, ID_BOOL as i64);
    builder.ins().store(MemFlags::new(), id, mem_ptr, 0);
}

pub fn ins_mem_write_f64(builder: &mut FunctionBuilder<'_>, ptr: Ptr, num: f64) {
    let mem_ptr = builder.ins().iconst(
        I64,
        MEMORY.as_ptr() as i64 + (ptr.0 * std::mem::size_of::<ScratchObject>()) as i64,
    );

    // write to the ptr
    let num = builder.ins().f64const(num);
    builder.ins().store(MemFlags::new(), num, mem_ptr, 8);

    let id = builder.ins().iconst(I64, ID_NUMBER as i64);
    builder.ins().store(MemFlags::new(), id, mem_ptr, 0);
}

pub fn ins_create_string_stack_slot(builder: &mut FunctionBuilder<'_>) -> Value {
    let stack_slot = builder.create_sized_stack_slot(StackSlotData::new(
        StackSlotKind::ExplicitSlot,
        3 * std::mem::size_of::<i64>() as u32,
        0,
    ));
    let stack_ptr = builder.ins().stack_addr(I64, stack_slot, 0);
    stack_ptr
}

pub fn ins_drop_obj(builder: &mut FunctionBuilder<'_>, ptr: Ptr) {
    let func = builder.ins().iconst(I64, callbacks::types::drop_obj as i64);
    let sig = builder.import_signature({
        let mut sig = Signature::new(CallConv::SystemV);
        sig.params.push(AbiParam::new(I64));
        sig
    });
    let mem_ptr = builder.ins().iconst(
        I64,
        MEMORY.as_ptr() as i64 + (ptr.0 * std::mem::size_of::<ScratchObject>()) as i64,
    );

    builder.ins().call_indirect(sig, func, &[mem_ptr]);
}
