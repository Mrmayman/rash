use codegen::ir::Inst;
use cranelift::prelude::*;
use isa::CallConv;
use types::{F64, I64};

use crate::{
    callbacks,
    compiler::Compiler,
    data_types::{ScratchObject, ID_BOOL, ID_NUMBER, ID_STRING},
    input_primitives::Ptr,
};

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

    pub fn ins_mem_write_string(
        &mut self,
        string: &str,
        builder: &mut FunctionBuilder<'_>,
        ptr: Ptr,
        memory: &[ScratchObject],
    ) {
        let string = string.to_owned();

        // Transmute the String into a [i64; 4] array
        let arr: [i64; 3] = unsafe { std::mem::transmute(string) };
        let i1 = self.constants.get_int(arr[0], builder);
        let i2 = self.constants.get_int(arr[1], builder);
        let i3 = self.constants.get_int(arr[2], builder);

        let mem_ptr = ptr.constant(self, builder, memory);

        builder.ins().store(MemFlags::new(), i1, mem_ptr, 8);
        builder.ins().store(MemFlags::new(), i2, mem_ptr, 16);
        builder.ins().store(MemFlags::new(), i3, mem_ptr, 24);

        let id = self.constants.get_int(ID_STRING, builder);
        builder.ins().store(MemFlags::new(), id, mem_ptr, 0);
    }

    pub fn ins_mem_write_bool(
        &mut self,
        builder: &mut FunctionBuilder<'_>,
        ptr: Ptr,
        num: bool,
        memory: &[ScratchObject],
    ) {
        let mem_ptr = ptr.constant(self, builder, memory);

        let num = self.constants.get_int(i64::from(num), builder);
        builder.ins().store(MemFlags::new(), num, mem_ptr, 8);

        let id = self.constants.get_int(ID_BOOL, builder);
        builder.ins().store(MemFlags::new(), id, mem_ptr, 0);
    }

    pub fn ins_mem_write_f64(
        &mut self,
        builder: &mut FunctionBuilder<'_>,
        ptr: Ptr,
        num: f64,
        memory: &[ScratchObject],
    ) {
        let mem_ptr = ptr.constant(self, builder, memory);

        // write to the ptr
        let num = self.constants.get_float(num, builder);
        builder.ins().store(MemFlags::new(), num, mem_ptr, 8);

        let id = self.constants.get_int(ID_NUMBER, builder);
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

    pub fn ins_drop_obj(
        &mut self,
        builder: &mut FunctionBuilder<'_>,
        ptr: Ptr,
        memory: &[ScratchObject],
    ) {
        let func = self
            .constants
            .get_int(callbacks::types::drop_obj as usize as i64, builder);
        let sig = builder.import_signature({
            let mut sig = Signature::new(CallConv::SystemV);
            sig.params.push(AbiParam::new(I64));
            sig
        });
        let mem_ptr = ptr.constant(self, builder, memory);

        builder.ins().call_indirect(sig, func, &[mem_ptr]);
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
