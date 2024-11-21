use cranelift::prelude::*;
use types::I64;

use crate::{
    callbacks,
    compiler::{Compiler, VarType},
    data_types::{ScratchObject, ID_BOOL, ID_NUMBER},
    input_primitives::{Input, Ptr, ReturnValue},
};

use super::call_function;

pub fn read(compiler: &mut Compiler, builder: &mut FunctionBuilder<'_>, ptr: Ptr) -> ReturnValue {
    match compiler.variable_type_data.get(&ptr) {
        Some(VarType::Number) => ReturnValue::Num(compiler.cache.load_f64(ptr, builder)),
        Some(VarType::Bool) => ReturnValue::Bool(compiler.cache.load_bool(ptr, builder)),
        _ => {
            let mem_ptr = compiler.cache.get_ptr(ptr, builder);
            let output_stack_slot = builder.create_sized_stack_slot(StackSlotData::new(
                StackSlotKind::ExplicitSlot,
                4 * std::mem::size_of::<usize>() as u32,
                8,
            ));
            let output_stack_ptr = builder.ins().stack_addr(I64, output_stack_slot, 0);

            call_function(
                compiler,
                builder,
                callbacks::var_read as usize,
                &[I64, I64],
                &[],
                &[mem_ptr, output_stack_ptr],
            );
            ReturnValue::ObjectPointer(output_stack_ptr, output_stack_slot)
        }
    }
}

pub fn set(compiler: &mut Compiler, obj: &Input, builder: &mut FunctionBuilder<'_>, ptr: Ptr) {
    match obj {
        Input::Obj(obj) => {
            if !matches!(
                compiler.variable_type_data.get(&ptr),
                Some(VarType::Number | VarType::Bool)
            ) {
                compiler.ins_drop_obj(builder, ptr);
            }
            match obj {
                ScratchObject::Number(num) => {
                    compiler
                        .cache
                        .store_f64(ptr, builder, *num, &mut compiler.constants);
                    compiler.variable_type_data.insert(ptr, VarType::Number);
                }
                ScratchObject::Bool(num) => {
                    compiler
                        .cache
                        .store_bool(ptr, builder, *num, &mut compiler.constants);
                    compiler.variable_type_data.insert(ptr, VarType::Bool);
                }
                ScratchObject::String(string) => {
                    compiler
                        .cache
                        .store_string(ptr, builder, string, &mut compiler.constants);
                    compiler.variable_type_data.insert(ptr, VarType::String);
                }
            }
        }
        Input::Block(block) => {
            // compile block
            let val = compiler.compile_block(block, builder);
            let val = val.unwrap();
            if !matches!(
                compiler.variable_type_data.get(&ptr),
                Some(VarType::Number | VarType::Bool)
            ) {
                compiler.ins_drop_obj(builder, ptr);
            }
            match val {
                ReturnValue::Num(value) | ReturnValue::Bool(value) => {
                    compiler.cache.store_small_value(
                        ptr,
                        builder,
                        value,
                        &mut compiler.constants,
                        if val.is_bool() { ID_BOOL } else { ID_NUMBER },
                    );

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
                    compiler.cache.store_object(ptr, builder, i1, i2, i3, i4);
                    compiler.variable_type_data.remove(&ptr);
                }
                ReturnValue::ObjectPointer(_value, slot) => {
                    let i1 = builder.ins().stack_load(I64, slot, 0);
                    let i2 = builder.ins().stack_load(I64, slot, 8);
                    let i3 = builder.ins().stack_load(I64, slot, 16);
                    let i4 = builder.ins().stack_load(I64, slot, 24);

                    compiler.cache.store_object(ptr, builder, i1, i2, i3, i4);
                    compiler.variable_type_data.remove(&ptr);
                }
            }
        }
    };
}

pub fn change(compiler: &mut Compiler, input: &Input, builder: &mut FunctionBuilder<'_>, ptr: Ptr) {
    let input = input.get_number(compiler, builder);
    let old_value = read(compiler, builder, ptr);
    let old_value = old_value.get_number(compiler, builder);
    let new_value = builder.ins().fadd(old_value, input);

    compiler
        .cache
        .store_small_value(ptr, builder, new_value, &mut compiler.constants, ID_NUMBER);
    compiler.variable_type_data.insert(ptr, VarType::Number);
}
