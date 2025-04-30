use cranelift::prelude::{types::I64, FunctionBuilder, InstBuilder, StackSlotData, StackSlotKind};

use crate::{
    callbacks,
    compiler::{Compiler, VarType},
    data_types::{ScratchObject, ID_BOOL, ID_NUMBER},
    input_primitives::{Input, Ptr, ReturnValue},
};

impl Compiler<'_> {
    pub fn var_read(&mut self, builder: &mut FunctionBuilder<'_>, ptr: Ptr) -> ReturnValue {
        match self.variable_type_data.get(&ptr) {
            Some(VarType::Number) => ReturnValue::Num(self.cache.load_f64(ptr, builder)),
            Some(VarType::Bool) => ReturnValue::Bool(self.cache.load_bool(ptr, builder)),
            _ => {
                let mem_ptr = self.cache.get_ptr(ptr, builder);
                let output_stack_slot = builder.create_sized_stack_slot(StackSlotData::new(
                    StackSlotKind::ExplicitSlot,
                    4 * std::mem::size_of::<usize>() as u32,
                    8,
                ));
                let output_stack_ptr = builder.ins().stack_addr(I64, output_stack_slot, 0);

                self.call_function(
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

    pub fn var_set(&mut self, obj: &Input, builder: &mut FunctionBuilder<'_>, ptr: Ptr) {
        match obj {
            Input::Obj(obj) => {
                if !matches!(
                    self.variable_type_data.get(&ptr),
                    Some(VarType::Number | VarType::Bool)
                ) {
                    self.ins_drop_obj(builder, ptr);
                }
                match obj {
                    ScratchObject::Number(num) => {
                        self.cache
                            .store_f64(ptr, builder, *num, &mut self.constants);
                        self.variable_type_data.insert(ptr, VarType::Number);
                    }
                    ScratchObject::Bool(num) => {
                        self.cache
                            .store_bool(ptr, builder, *num, &mut self.constants);
                        self.variable_type_data.insert(ptr, VarType::Bool);
                    }
                    ScratchObject::String(string) => {
                        self.cache
                            .store_string(ptr, builder, string, &mut self.constants);
                        self.variable_type_data.insert(ptr, VarType::String);
                    }
                }
            }
            Input::Block(block) => {
                let val = self.compile_block(block, builder)
                    .expect("blocks inside other blocks (like an add operator in a set var block) should return something!");

                if matches!(self.variable_type_data.get(&ptr), Some(VarType::String)) {
                    self.ins_drop_obj(builder, ptr);
                }

                match val {
                    ReturnValue::Num(value) | ReturnValue::Bool(value) => {
                        self.cache.store_small_value(
                            ptr,
                            builder,
                            value,
                            &mut self.constants,
                            if val.is_bool() { ID_BOOL } else { ID_NUMBER },
                        );

                        self.variable_type_data.insert(
                            ptr,
                            if val.is_bool() {
                                VarType::Bool
                            } else {
                                VarType::Number
                            },
                        );
                    }
                    ReturnValue::Object([i1, i2, i3, i4]) => {
                        self.cache.store_object(ptr, builder, i1, i2, i3, i4);
                        self.variable_type_data.remove(&ptr);
                    }
                    ReturnValue::ObjectPointer(_value, slot) => {
                        let i1 = builder.ins().stack_load(I64, slot, 0);
                        let i2 = builder.ins().stack_load(I64, slot, 8);
                        let i3 = builder.ins().stack_load(I64, slot, 16);
                        let i4 = builder.ins().stack_load(I64, slot, 24);

                        self.cache.store_object(ptr, builder, i1, i2, i3, i4);
                        self.variable_type_data.remove(&ptr);
                    }
                }
            }
        };
    }

    pub fn var_change(&mut self, input: &Input, builder: &mut FunctionBuilder<'_>, ptr: Ptr) {
        let input = input.get_number(self, builder);
        let old_value = self.var_read(builder, ptr);
        let old_value = old_value.get_number(self, builder);
        let new_value = builder.ins().fadd(old_value, input);

        self.cache
            .store_small_value(ptr, builder, new_value, &mut self.constants, ID_NUMBER);
        self.variable_type_data.insert(ptr, VarType::Number);
    }
}
