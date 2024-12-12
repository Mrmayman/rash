use cranelift::prelude::{
    types::I64, FunctionBuilder, InstBuilder, StackSlotData, StackSlotKind, Value,
};

use crate::{
    callbacks, compiler::Compiler, data_types::ScratchObject, input_primitives::Input,
    scheduler::CustomBlockId,
};

impl Compiler<'_> {
    pub fn call_no_screen_refresh(
        &mut self,
        custom_block_id: &CustomBlockId,
        builder: &mut FunctionBuilder<'_>,
        args: &[Input],
    ) {
        let custom_block_id = self.constants.get_int(custom_block_id.0 as i64, builder);

        self.variable_type_data.clear();
        self.cache.save(builder, &mut self.constants, self.memory);
        let args: Vec<[Value; 4]> = args
            .iter()
            .map(|n| n.get_object(self, builder))
            .collect::<Vec<_>>();

        let stack_slot = builder.create_sized_stack_slot(StackSlotData {
            kind: StackSlotKind::ExplicitSlot,
            size: (std::mem::size_of::<ScratchObject>() * args.len()) as u32,
            align_shift: 0,
        });
        for [i1, i2, i3, i4] in args {
            builder.ins().stack_store(i1, stack_slot, 0);
            builder.ins().stack_store(i2, stack_slot, 8);
            builder.ins().stack_store(i3, stack_slot, 16);
            builder.ins().stack_store(i4, stack_slot, 24);
        }
        let slot_ptr = builder.ins().stack_addr(I64, stack_slot, 0);

        self.call_function(
            builder,
            callbacks::custom_block::call_no_screen_refresh as usize,
            &[I64, I64, I64],
            &[],
            &[slot_ptr, custom_block_id, self.scheduler_ptr],
        );
        self.cache.init(builder, self.memory, &mut self.constants);
    }
}
