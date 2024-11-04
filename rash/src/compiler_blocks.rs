use std::collections::HashMap;

use cranelift::prelude::*;
use isa::CallConv;
use types::{F64, I64};

use crate::{
    callbacks,
    compiler::{compile_block, VarType, MEMORY},
    data_types::{ScratchObject, ID_BOOL, ID_NUMBER},
    input_primitives::{Input, Ptr, ReturnValue},
    ins_shortcuts::{ins_drop_obj, ins_mem_write_bool, ins_mem_write_f64, ins_mem_write_string},
};

pub fn c_var_read(
    builder: &mut FunctionBuilder<'_>,
    ptr: Ptr,
    variable_type_data: &HashMap<Ptr, VarType>,
) -> ReturnValue {
    match variable_type_data.get(&ptr) {
        Some(VarType::Number) => {
            println!("reading number {}", ptr.0);
            // if ptr.0 == 2 {
            //     return ReturnValue::Num(builder.ins().f64const(10.0));
            // }
            let mem_ptr = builder
                .ins()
                .iconst(I64, unsafe { MEMORY.as_ptr().offset(ptr.0 as isize) }
                    as i64);
            let value = builder.ins().load(F64, MemFlags::new(), mem_ptr, 8);
            return ReturnValue::Num(value);
        }
        Some(VarType::Bool) => {
            let mem_ptr = builder.ins().iconst(
                I64,
                MEMORY.as_ptr() as i64 + (ptr.0 * std::mem::size_of::<ScratchObject>()) as i64,
            );
            let value = builder.ins().load(I64, MemFlags::new(), mem_ptr, 8);
            return ReturnValue::Num(value);
        }
        _ => {
            let func = builder.ins().iconst(I64, callbacks::var_read as i64);
            let sig = builder.import_signature({
                let mut sig = Signature::new(CallConv::SystemV);
                sig.params.push(AbiParam::new(I64));
                sig.params.push(AbiParam::new(I64));
                sig
            });
            let mem_ptr = builder.ins().iconst(
                I64,
                MEMORY.as_ptr() as i64 + (ptr.0 * std::mem::size_of::<ScratchObject>()) as i64,
            );
            let stack_slot = builder.create_sized_stack_slot(StackSlotData::new(
                StackSlotKind::ExplicitSlot,
                4 * std::mem::size_of::<usize>() as u32,
                8,
            ));
            let stack_ptr = builder.ins().stack_addr(I64, stack_slot, 0);
            builder
                .ins()
                .call_indirect(sig, func, &[mem_ptr, stack_ptr]);
            return ReturnValue::ObjectPointer(stack_ptr, stack_slot);
        }
    }

    // let obj = ScratchObject::Number(3.0);
    // let transmuted_obj: [usize; 4] = unsafe { std::mem::transmute(obj) };
    // let i1 = builder.ins().iconst(I64, transmuted_obj[0]);
    // let i2 = builder.ins().iconst(I64, transmuted_obj[1]);
    // let i3 = builder.ins().iconst(I64, transmuted_obj[2]);
    // let i4 = builder.ins().iconst(I64, transmuted_obj[3]);
    // return Some(ReturnValue::Object((i1, i2, i3, i4)));
}

pub fn c_var_set(
    obj: &Input,
    builder: &mut FunctionBuilder<'_>,
    ptr: &Ptr,
    variable_type_data: &mut HashMap<Ptr, VarType>,
    code_block: &mut Block,
) {
    match obj {
        Input::Obj(obj) => {
            ins_drop_obj(builder, *ptr);
            match obj {
                ScratchObject::Number(num) => {
                    ins_mem_write_f64(builder, *ptr, *num);
                    variable_type_data.insert(*ptr, VarType::Number);
                }
                ScratchObject::Bool(num) => {
                    ins_mem_write_bool(builder, *ptr, *num);
                    variable_type_data.insert(*ptr, VarType::Bool);
                }
                ScratchObject::String(string) => {
                    ins_mem_write_string(string, builder, *ptr);
                    variable_type_data.insert(*ptr, VarType::String);
                }
            }
        }
        Input::Block(block) => {
            // compile block
            let val = compile_block(block, builder, code_block, variable_type_data);
            let val = val.unwrap();
            if !matches!(
                variable_type_data.get(ptr),
                Some(VarType::Number) | Some(VarType::Bool)
            ) {
                ins_drop_obj(builder, *ptr);
            }
            match val {
                ReturnValue::Num(value) | ReturnValue::Bool(value) => {
                    let mem_ptr = builder.ins().iconst(
                        I64,
                        MEMORY.as_ptr() as i64
                            + (ptr.0 * std::mem::size_of::<ScratchObject>()) as i64,
                    );
                    builder.ins().store(MemFlags::new(), value, mem_ptr, 8);

                    let id = builder
                        .ins()
                        .iconst(I64, if val.is_bool() { ID_BOOL } else { ID_NUMBER } as i64);
                    builder.ins().store(MemFlags::new(), id, mem_ptr, 0);

                    variable_type_data.insert(
                        *ptr,
                        if val.is_bool() {
                            VarType::Bool
                        } else {
                            VarType::Number
                        },
                    );
                }
                ReturnValue::Object((i1, i2, i3, i4)) => {
                    let mem_ptr = builder.ins().iconst(
                        I64,
                        MEMORY.as_ptr() as i64
                            + (ptr.0 * std::mem::size_of::<ScratchObject>()) as i64,
                    );

                    builder.ins().store(MemFlags::new(), i1, mem_ptr, 0);
                    builder.ins().store(MemFlags::new(), i2, mem_ptr, 8);
                    builder.ins().store(MemFlags::new(), i3, mem_ptr, 16);
                    builder.ins().store(MemFlags::new(), i4, mem_ptr, 24);
                    variable_type_data.remove(ptr);
                }
                ReturnValue::ObjectPointer(_value, slot) => {
                    let i1 = builder.ins().stack_load(I64, slot, 0);
                    let i2 = builder.ins().stack_load(I64, slot, 8);
                    let i3 = builder.ins().stack_load(I64, slot, 16);
                    let i4 = builder.ins().stack_load(I64, slot, 24);

                    let mem_ptr = builder.ins().iconst(
                        I64,
                        MEMORY.as_ptr() as i64
                            + (ptr.0 * std::mem::size_of::<ScratchObject>()) as i64,
                    );

                    builder.ins().store(MemFlags::new(), i1, mem_ptr, 0);
                    builder.ins().store(MemFlags::new(), i2, mem_ptr, 8);
                    builder.ins().store(MemFlags::new(), i3, mem_ptr, 16);
                    builder.ins().store(MemFlags::new(), i4, mem_ptr, 24);
                    variable_type_data.remove(ptr);
                }
            }
        }
    };
}
