use cranelift::{
    codegen::ir::BlockArg,
    prelude::{FunctionBuilder, InstBuilder, IntCC, Value, types::I64},
};

use crate::{
    callbacks,
    compiler::{Compiler, ScratchBlock},
    effects::{CheckEffects, Effects},
    input_primitives::{Input, Ptr, ScratchValue},
    variable_storage::VariableSlot,
};

impl Compiler<'_> {
    pub fn control_stop_this_script(&mut self, builder: &mut FunctionBuilder<'_>) {
        for _ in 0..self.repeat_stack {
            self.call_stack_pop(builder);
        }

        self.cache.save(builder, &mut self.constants, self.memory);
        let minus_one = self.constants.get_int(-1, builder);
        builder.ins().return_(&[minus_one]);
        let new_block = builder.create_block();
        builder.switch_to_block(new_block);
        self.code_block = new_block;
    }

    pub fn control_repeat(
        &mut self,
        builder: &mut FunctionBuilder<'_>,
        input: &Input,
        blocks: &[ScratchBlock],
    ) {
        let effects = blocks.effects(&mut |_| Effects::unknown(), &|v| self.cache.get_type(v));

        let zero = self.constants.get_int(0, builder);
        let old_constants = self.constants.clone();

        let loop_value = input.get_number_int(self, builder);

        let loop_block = builder.create_block();
        let end_block = builder.create_block();

        let final_params_loop = effects.generate_params(builder, loop_block, &*self.cache);
        let mut loop_value_param = builder.append_block_param(loop_block, I64); // loop counter
        let final_params_end = effects.generate_params(builder, end_block, &*self.cache);

        let entry_params = self.generate_params(builder, &final_params_loop);
        let mut entry_params2 = entry_params.clone();
        entry_params2.push(loop_value.into());
        let condition = builder
            .ins()
            .icmp(IntCC::SignedGreaterThan, loop_value, zero);
        builder.ins().brif(
            condition,
            loop_block,
            &entry_params2,
            end_block,
            &entry_params,
        );

        builder.switch_to_block(loop_block);
        self.code_block = loop_block;
        self.cache.extend(final_params_loop.clone());

        if effects.yields {
            self.constants.clear();
            self.repeat_stack += 1;
            self.call_stack_push(builder, loop_value_param);
        }
        for block in blocks {
            self.compile_block(block, builder);
        }
        if effects.yields {
            self.repeat_stack -= 1;
            loop_value_param = self.call_stack_pop(builder);
        }

        let loop_params = self.generate_params(builder, &final_params_loop);
        let mut loop_params2 = loop_params.clone();

        let one = self.constants.get_int(1, builder);
        let new_loop_value = builder.ins().isub(loop_value_param, one);
        loop_params2.push(new_loop_value.into());

        let zero = self.constants.get_int(0, builder);
        let condition = builder
            .ins()
            .icmp(IntCC::SignedGreaterThan, new_loop_value, zero);
        builder.ins().brif(
            condition,
            loop_block,
            &loop_params2,
            end_block,
            &loop_params,
        );

        builder.switch_to_block(end_block);
        self.cache.extend(final_params_end.clone());
        self.code_block = end_block;
        if effects.yields {
            self.constants.clear();
        } else {
            self.constants = old_constants;
        }
    }

    pub fn control_forever(&mut self, builder: &mut FunctionBuilder<'_>, blocks: &[ScratchBlock]) {
        let loop_block = builder.create_block();
        let end_block = builder.create_block();

        // TODO: inter-function analysis
        let effects = blocks.effects(&mut |_| Effects::unknown(), &|v| self.cache.get_type(v));

        let final_params = effects.generate_params(builder, loop_block, &*self.cache);

        let entry_params = self.generate_params(builder, &final_params);

        builder.ins().jump(loop_block, &entry_params);
        self.code_block = loop_block;
        builder.switch_to_block(loop_block);
        self.cache.extend(final_params.clone());

        for block in blocks {
            self.compile_block(block, builder);
        }
        let loop_params = self.generate_params(builder, &final_params);
        builder.ins().jump(loop_block, &loop_params);

        builder.switch_to_block(end_block);
        self.code_block = end_block;
    }

    fn call_stack_pop(&mut self, builder: &mut FunctionBuilder<'_>) -> Value {
        let inst = self.call_function(
            builder,
            callbacks::repeat_stack::STACK_POP,
            &[I64],
            &[I64],
            &[self.loop_stack_ptr],
        );
        builder.inst_results(inst)[0]
    }

    fn call_stack_push(&mut self, builder: &mut FunctionBuilder<'_>, incremented: Value) {
        self.call_function(
            builder,
            callbacks::repeat_stack::STACK_PUSH,
            &[I64, I64],
            &[],
            &[self.loop_stack_ptr, incremented],
        );
    }

    pub fn control_if(
        &mut self,
        condition: &Input,
        builder: &mut FunctionBuilder<'_>,
        then: &[ScratchBlock],
    ) {
        // TODO: inter-function analysis
        let effects = then.effects(&mut |_| Effects::unknown(), &|v| self.cache.get_type(v));

        let inside_block = builder.create_block();
        let end_block = builder.create_block();
        let final_params = effects.generate_params(builder, end_block, &*self.cache);

        // Before the code runs...
        let direct_params = self.generate_params(builder, &final_params);

        let condition = condition.get_bool(self, builder);
        builder
            .ins()
            .brif(condition, inside_block, &[], end_block, &direct_params);

        builder.switch_to_block(inside_block);
        self.code_block = inside_block;
        let old_consts = self.constants.clone();
        for block in then {
            self.compile_block(block, builder);
        }

        // After the code runs...
        let end_params = self.generate_params(builder, &final_params);
        builder.ins().jump(end_block, &end_params);
        self.code_block = end_block;
        builder.switch_to_block(end_block);
        self.constants = old_consts;
        self.cache.extend(final_params);
    }

    fn generate_params(
        &mut self,
        builder: &mut FunctionBuilder<'_>,
        final_params: &Vec<(Ptr, VariableSlot)>,
    ) -> Vec<BlockArg> {
        let mut direct_params = Vec::new();
        if !self.cache.uses_block_params() {
            return direct_params;
        }

        for (ptr, param) in final_params {
            let val = self.cache.get(*ptr, builder, &mut self.constants);
            match param.val {
                ScratchValue::Num(_) => {
                    let ScratchValue::Num(v) = val.val else {
                        panic!("Not a number? Some type checking went wrong (val: {val:?})");
                    };
                    direct_params.push(v.into());
                }
                ScratchValue::Bool(_) => {
                    let ScratchValue::Bool(v) = val.val else {
                        panic!("Not a bool? Some type checking went wrong (val: {val:?})");
                    };
                    direct_params.push(v.into());
                }
                ScratchValue::String(_) => {
                    let ScratchValue::String([i1, i2, i3]) = val.val else {
                        panic!("Not a string? Some type checking went wrong (val: {val:?})");
                    };
                    direct_params.push(i1.into());
                    direct_params.push(i2.into());
                    direct_params.push(i3.into());
                }
                ScratchValue::Object(_) => {
                    let [i1, i2, i3, i4] = val.val.get_object(builder, &mut self.constants);
                    direct_params.push(i1.into());
                    direct_params.push(i2.into());
                    direct_params.push(i3.into());
                    direct_params.push(i4.into());
                }
            }
        }
        direct_params
    }

    pub fn control_if_else(
        &mut self,
        condition: &Input,
        builder: &mut FunctionBuilder<'_>,

        then_blocks: &[ScratchBlock],
        else_blocks: &[ScratchBlock],
    ) {
        // TODO: inter-function analysis
        let effects = then_blocks.effects(&mut |_| Effects::unknown(), &|v| self.cache.get_type(v))
            & else_blocks.effects(&mut |_| Effects::unknown(), &|v| self.cache.get_type(v));

        let then_block = builder.create_block();
        let else_block = builder.create_block();
        let end_block = builder.create_block();

        let final_params = effects.generate_params(builder, end_block, &*self.cache);

        let condition = condition.get_bool(self, builder);
        builder
            .ins()
            .brif(condition, then_block, &[], else_block, &[]);

        let old_consts = self.constants.clone();
        let old_cache = self.cache.clone_box();

        builder.switch_to_block(then_block);
        self.code_block = then_block;
        for block in then_blocks {
            self.compile_block(block, builder);
        }

        let then_params = self.generate_params(builder, &final_params);
        builder.ins().jump(end_block, &then_params);

        builder.switch_to_block(else_block);
        self.constants = old_consts.clone();
        self.cache = old_cache;
        self.code_block = else_block;
        for block in else_blocks {
            self.compile_block(block, builder);
        }

        let else_params = self.generate_params(builder, &final_params);
        builder.ins().jump(end_block, &else_params);

        self.code_block = end_block;
        builder.switch_to_block(end_block);
        self.constants = old_consts;
        self.cache.extend(final_params);
    }

    pub fn control_repeat_until(
        &mut self,
        builder: &mut FunctionBuilder<'_>,
        input: &Input,
        body: &[ScratchBlock],
    ) {
        let condition_block = builder.create_block();
        let loop_block = builder.create_block();
        let end_block = builder.create_block();

        // TODO: inter-function analysis
        let effects = body.effects(&mut |_| Effects::unknown(), &|v| self.cache.get_type(v));

        let final_params = effects.generate_params(builder, condition_block, &*self.cache);

        let entry_params = self.generate_params(builder, &final_params);
        builder.ins().jump(condition_block, &entry_params);

        self.cache.extend(final_params.clone());
        builder.switch_to_block(condition_block);
        self.code_block = condition_block;

        let condition = input.get_bool(self, builder);
        builder
            .ins()
            .brif(condition, end_block, &[], loop_block, &[]);

        builder.switch_to_block(loop_block);
        self.code_block = loop_block;
        let old_constants = self.constants.clone();

        for block in body {
            self.compile_block(block, builder);
        }
        let loop_params = self.generate_params(builder, &final_params);
        builder.ins().jump(condition_block, &loop_params);
        builder.switch_to_block(end_block);
        self.code_block = end_block;
        self.cache.extend(final_params);
        self.constants = old_constants;
    }
}
