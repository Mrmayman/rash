use cranelift::prelude::{
    FunctionBuilder, InstBuilder, StackSlotData, StackSlotKind, Value,
    types::{F64, I64},
};

use crate::{
    callbacks,
    compiler::{Compiler, VarTypeChecked},
    data_types::ID_STRING,
    input_primitives::{Ptr, ScratchValue},
};

impl Compiler<'_> {
    pub fn ins_create_string_stack_slot(builder: &mut FunctionBuilder<'_>) -> Value {
        let stack_slot = builder.create_sized_stack_slot(StackSlotData::new(
            StackSlotKind::ExplicitSlot,
            3 * std::mem::size_of::<i64>() as u32,
            0,
        ));
        builder.ins().stack_addr(I64, stack_slot, 0)
    }

    pub fn ins_drop_obj(&mut self, builder: &mut FunctionBuilder<'_>, ptr: Ptr) {
        if let VarTypeChecked::Number | VarTypeChecked::Bool = self.cache.get_type(ptr).ty {
            return;
        }

        let obj = match &self.cache.get(ptr, builder, &mut self.constants).val {
            ScratchValue::String([i2, i3, i4]) => {
                let id = self.constants.get_int(ID_STRING, builder);
                [id, *i2, *i3, *i4]
            }
            ScratchValue::Object(obj) => *obj,
            _ => return,
        };

        self.call_function(
            builder,
            callbacks::types::drop_obj as *const (),
            &[I64, I64, I64, I64],
            &[],
            &obj,
        );
    }

    pub fn ins_call_to_num_with_decimal_check(
        &mut self,
        builder: &mut FunctionBuilder<'_>,
        i1: Value,
        i2: Value,
        i3: Value,
        i4: Value,
    ) -> (Value, Value) {
        self.call_function(
            builder,
            callbacks::types::to_number_with_decimal_check as *const (),
            &[I64, I64, I64, I64, I64],
            &[],
            &[i1, i2, i3, i4, self.temp_slot4.0],
        );
        let n = builder.ins().stack_load(F64, self.temp_slot4.1, 0);
        let b = builder.ins().stack_load(I64, self.temp_slot4.1, 8);
        (n, b)
    }
}
