use cranelift::prelude::{
    types::{F64, I64},
    FunctionBuilder, InstBuilder, StackSlotData, StackSlotKind, Value,
};

use crate::{callbacks, compiler::Compiler, input_primitives::Ptr};

impl Compiler<'_> {
    pub fn ins_create_string_stack_slot(builder: &mut FunctionBuilder<'_>) -> Value {
        let stack_slot = builder.create_sized_stack_slot(StackSlotData::new(
            StackSlotKind::ExplicitSlot,
            3 * std::mem::size_of::<i64>() as u32,
            0,
        ));
        let stack_ptr = builder.ins().stack_addr(I64, stack_slot, 0);
        stack_ptr
    }

    pub fn ins_drop_obj(&mut self, builder: &mut FunctionBuilder<'_>, ptr: Ptr) {
        let stack_addr = self.cache.get_ptr(ptr, builder);
        self.call_function(
            builder,
            callbacks::types::drop_obj as usize,
            &[I64],
            &[],
            &[stack_addr],
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
        let stack_slot = builder.create_sized_stack_slot(StackSlotData {
            kind: StackSlotKind::ExplicitSlot,
            size: 2 * std::mem::size_of::<i64>() as u32,
            align_shift: 0,
        });
        let stack_ptr = builder.ins().stack_addr(I64, stack_slot, 0);

        self.call_function(
            builder,
            callbacks::types::to_number_with_decimal_check as usize,
            &[I64, I64, I64, I64, I64],
            &[],
            &[i1, i2, i3, i4, stack_ptr],
        );
        let n = builder.ins().stack_load(F64, stack_slot, 0);
        let b = builder.ins().stack_load(I64, stack_slot, 8);
        (n, b)
    }
}
