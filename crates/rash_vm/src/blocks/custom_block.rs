use cranelift::{
    codegen::ir::types::I8,
    prelude::{FunctionBuilder, InstBuilder, StackSlotData, StackSlotKind, types::I64},
};

use crate::{
    callbacks,
    compiler::Compiler,
    data_types::ScratchObject,
    effects::Effects,
    input_primitives::{CompObj, Input},
    runtime::CustomBlockId,
};

impl Compiler<'_> {
    pub fn call_custom_block(
        &mut self,
        custom_block_id: CustomBlockId,
        builder: &mut FunctionBuilder<'_>,
        args: &[Input],
        is_screen_refresh: bool,
    ) {
        let mut effects = (self.custom_block_effects)(custom_block_id);
        effects.set_direct(true);
        effects
            .writes
            .retain(|p, _| !self.clobber_stack.contains(p));
        let custom_block_id = self.constants.get_int(custom_block_id.0 as i64, builder);

        self.vars
            .save(builder, &mut self.constants, self.memory, &effects);
        let args: Vec<CompObj> = args
            .iter()
            .map(|n| n.get_object(self, builder))
            .collect::<Vec<_>>();

        // TODO: pool slots
        let stack_slot = builder.create_sized_stack_slot(StackSlotData {
            kind: StackSlotKind::ExplicitSlot,
            size: (std::mem::size_of::<ScratchObject>() * args.len()) as u32,
            align_shift: 0,
            key: None,
        });
        for (i, obj) in args.into_iter().enumerate() {
            let [i1, i2, i3] = obj.0;
            let o = (i * std::mem::size_of::<ScratchObject>()) as i32;
            // This will later be dropped inside the callback
            builder.ins().stack_store(i1, stack_slot, o);
            builder.ins().stack_store(i2, stack_slot, o + 8);
            builder.ins().stack_store(i3, stack_slot, o + 16);
        }
        let slot_ptr = builder.ins().stack_addr(I64, stack_slot, 0);

        if is_screen_refresh && self.is_inherently_pausable {
            let inst = self.call_function(
                builder,
                callbacks::custom_block::CALL_SCREEN_REFRESH,
                &[I64, I64, I64, I64, I64, I8],
                &[I64],
                &[
                    slot_ptr,
                    custom_block_id,
                    self.jit_args.script_ptr,
                    self.jit_args.graphics_ptr,
                    self.jit_args.child_thread_ptr,
                    self.jit_args.is_called_by_pausable,
                ],
            );

            let is_paused = builder.inst_results(inst)[0];

            let inside_block = builder.create_block();
            let end_block = builder.create_block();

            builder
                .ins()
                .brif(is_paused, inside_block, &[], end_block, &[]);

            builder.switch_to_block(inside_block);
            self.break_counter += 1;
            let break_counter = builder.ins().iconst(I64, self.break_counter as i64);

            builder.ins().return_(&[break_counter]);

            self.break_points.push(end_block);
            builder.switch_to_block(end_block);

            self.constants.clear();
        } else {
            self.call_function(
                builder,
                callbacks::custom_block::CALL_NO_SCREEN_REFRESH,
                &[I64, I64, I64, I64],
                &[],
                &[
                    slot_ptr,
                    custom_block_id,
                    self.jit_args.script_ptr,
                    self.jit_args.graphics_ptr,
                ],
            );
        }

        self.vars
            .reinit(builder, &mut self.constants, self.memory, &effects);
    }

    pub fn screen_refresh(&mut self, builder: &mut FunctionBuilder<'_>) {
        if !self.is_inherently_pausable {
            return;
        }

        for _ in 0..2 {
            // TODO: Hacky workaround for too-fast timing
            self.break_counter += 1;
            self.save_vars(builder);
            let break_counter = self.constants.get_int(self.break_counter as i64, builder);

            builder.ins().return_(&[break_counter]);
            self.constants.clear();

            let b = builder.create_block();
            self.break_points.push(b);
            builder.switch_to_block(b);

            self.vars.reinit(
                builder,
                &mut self.constants,
                self.memory,
                &Effects::unknown(),
            );
        }
    }
}
