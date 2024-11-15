use codegen::ir::Inst;
use cranelift::prelude::*;
use isa::CallConv;
use types::{F64, I64};

use crate::{callbacks, compiler::Compiler, input_primitives::Ptr};

impl Compiler {
    pub fn ins_call_to_num(
        &mut self,
        builder: &mut FunctionBuilder<'_>,
        i1: Value,
        i2: Value,
        i3: Value,
        i4: Value,
    ) -> Inst {
        let to_num_func = self
            .constants
            .get_int(callbacks::types::to_number as usize as i64, builder);
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
        let func = self
            .constants
            .get_int(callbacks::types::drop_obj as usize as i64, builder);
        let sig = builder.import_signature({
            let mut sig = Signature::new(CallConv::SystemV);
            sig.params.push(AbiParam::new(I64));
            sig
        });
        let stack_addr = self.cache.get_ptr(ptr, builder);

        builder.ins().call_indirect(sig, func, &[stack_addr]);
    }

    pub fn ins_call_to_num_with_decimal_check(
        &mut self,
        builder: &mut FunctionBuilder<'_>,
        i1: Value,
        i2: Value,
        i3: Value,
        i4: Value,
    ) -> (Value, Value) {
        let to_num_func = self.constants.get_int(
            callbacks::types::to_number_with_decimal_check as usize as i64,
            builder,
        );
        let stack_slot = builder.create_sized_stack_slot(StackSlotData {
            kind: StackSlotKind::ExplicitSlot,
            size: 2 * std::mem::size_of::<i64>() as u32,
            align_shift: 0,
        });
        let stack_ptr = builder.ins().stack_addr(I64, stack_slot, 0);
        let sig = builder.import_signature({
            let mut sig = Signature::new(CallConv::SystemV);
            sig.params.push(AbiParam::new(I64));
            sig.params.push(AbiParam::new(I64));
            sig.params.push(AbiParam::new(I64));
            sig.params.push(AbiParam::new(I64));
            sig.params.push(AbiParam::new(I64));
            sig
        });
        builder
            .ins()
            .call_indirect(sig, to_num_func, &[i1, i2, i3, i4, stack_ptr]);
        let n = builder.ins().stack_load(F64, stack_slot, 0);
        let b = builder.ins().stack_load(I64, stack_slot, 8);
        (n, b)
    }
}
