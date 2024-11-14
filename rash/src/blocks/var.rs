use cranelift::prelude::*;
use isa::CallConv;
use types::{F64, I64};

use crate::{
    callbacks,
    compiler::{Compiler, VarType},
    data_types::{ScratchObject, ID_BOOL, ID_NUMBER},
    input_primitives::{Input, Ptr, ReturnValue},
};

pub fn read(
    compiler: &mut Compiler,
    builder: &mut FunctionBuilder<'_>,
    ptr: Ptr,
    memory: &[ScratchObject],
) -> ReturnValue {
    match compiler.variable_type_data.get(&ptr) {
        Some(VarType::Number) => {
            let mem_ptr = ptr.constant(compiler, builder, memory);
            let value = builder.ins().load(F64, MemFlags::new(), mem_ptr, 8);
            ReturnValue::Num(value)
        }
        Some(VarType::Bool) => {
            let mem_ptr = ptr.constant(compiler, builder, memory);
            let value = builder.ins().load(I64, MemFlags::new(), mem_ptr, 8);
            ReturnValue::Bool(value)
        }
        _ => {
            let func = compiler
                .constants
                .get_int(callbacks::var_read as usize as i64, builder);
            let sig = builder.import_signature({
                let mut sig = Signature::new(CallConv::SystemV);
                sig.params.push(AbiParam::new(I64));
                sig.params.push(AbiParam::new(I64));
                sig
            });
            let mem_ptr = ptr.constant(compiler, builder, memory);
            let stack_slot = builder.create_sized_stack_slot(StackSlotData::new(
                StackSlotKind::ExplicitSlot,
                4 * std::mem::size_of::<usize>() as u32,
                8,
            ));
            let stack_ptr = builder.ins().stack_addr(I64, stack_slot, 0);
            builder
                .ins()
                .call_indirect(sig, func, &[mem_ptr, stack_ptr]);
            ReturnValue::ObjectPointer(stack_ptr, stack_slot)
        }
    }
}

pub fn set(
    compiler: &mut Compiler,
    obj: &Input,
    builder: &mut FunctionBuilder<'_>,
    ptr: Ptr,
    memory: &[ScratchObject],
) {
    match obj {
        Input::Obj(obj) => {
            if !matches!(
                compiler.variable_type_data.get(&ptr),
                Some(VarType::Number | VarType::Bool)
            ) {
                compiler.ins_drop_obj(builder, ptr, memory);
            }
            match obj {
                ScratchObject::Number(num) => {
                    compiler.ins_mem_write_f64(builder, ptr, *num, memory);
                    compiler.variable_type_data.insert(ptr, VarType::Number);
                }
                ScratchObject::Bool(num) => {
                    compiler.ins_mem_write_bool(builder, ptr, *num, memory);
                    compiler.variable_type_data.insert(ptr, VarType::Bool);
                }
                ScratchObject::String(string) => {
                    compiler.ins_mem_write_string(string, builder, ptr, memory);
                    compiler.variable_type_data.insert(ptr, VarType::String);
                }
            }
        }
        Input::Block(block) => {
            // compile block
            let val = compiler.compile_block(block, builder, memory);
            let val = val.unwrap();
            if !matches!(
                compiler.variable_type_data.get(&ptr),
                Some(VarType::Number | VarType::Bool)
            ) {
                compiler.ins_drop_obj(builder, ptr, memory);
            }
            match val {
                ReturnValue::Num(value) | ReturnValue::Bool(value) => {
                    let mem_ptr = ptr.constant(compiler, builder, memory);
                    builder.ins().store(MemFlags::new(), value, mem_ptr, 8);

                    let id = compiler
                        .constants
                        .get_int(if val.is_bool() { ID_BOOL } else { ID_NUMBER }, builder);
                    builder.ins().store(MemFlags::new(), id, mem_ptr, 0);

                    compiler.variable_type_data.insert(
                        ptr,
                        if val.is_bool() {
                            VarType::Bool
                        } else {
                            VarType::Number
                        },
                    );
                }
                ReturnValue::Object((i1, i2, i3, i4)) => {
                    let mem_ptr = ptr.constant(compiler, builder, memory);

                    builder.ins().store(MemFlags::new(), i1, mem_ptr, 0);
                    builder.ins().store(MemFlags::new(), i2, mem_ptr, 8);
                    builder.ins().store(MemFlags::new(), i3, mem_ptr, 16);
                    builder.ins().store(MemFlags::new(), i4, mem_ptr, 24);
                    compiler.variable_type_data.remove(&ptr);
                }
                ReturnValue::ObjectPointer(_value, slot) => {
                    let i1 = builder.ins().stack_load(I64, slot, 0);
                    let i2 = builder.ins().stack_load(I64, slot, 8);
                    let i3 = builder.ins().stack_load(I64, slot, 16);
                    let i4 = builder.ins().stack_load(I64, slot, 24);

                    let mem_ptr = ptr.constant(compiler, builder, memory);

                    builder.ins().store(MemFlags::new(), i1, mem_ptr, 0);
                    builder.ins().store(MemFlags::new(), i2, mem_ptr, 8);
                    builder.ins().store(MemFlags::new(), i3, mem_ptr, 16);
                    builder.ins().store(MemFlags::new(), i4, mem_ptr, 24);
                    compiler.variable_type_data.remove(&ptr);
                }
            }
        }
    };
}

pub fn change(
    compiler: &mut Compiler,
    input: &Input,
    builder: &mut FunctionBuilder<'_>,
    ptr: Ptr,
    memory: &[ScratchObject],
) {
    let input = input.get_number(compiler, builder, memory);
    let old_value = read(compiler, builder, ptr, memory);
    let old_value = old_value.get_number(compiler, builder);
    let new_value = builder.ins().fadd(old_value, input);

    let mem_ptr = ptr.constant(compiler, builder, memory);

    builder.ins().store(MemFlags::new(), new_value, mem_ptr, 8);

    if !matches!(compiler.variable_type_data.get(&ptr), Some(VarType::Number)) {
        let id = compiler.constants.get_int(ID_NUMBER, builder);
        builder.ins().store(MemFlags::new(), id, mem_ptr, 0);
        compiler.variable_type_data.insert(ptr, VarType::Number);
    }
}
