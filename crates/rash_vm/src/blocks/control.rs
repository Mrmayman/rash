use std::collections::HashMap;

use cranelift::{
    codegen::ir::BlockArg,
    prelude::{FunctionBuilder, InstBuilder, IntCC, Value, types::I64},
};

use crate::{
    callbacks,
    compiler::{Compiler, ScratchBlock, VarType, VarTypeChecked},
    effects::{CheckEffects, Effects},
    input_primitives::{Input, Ptr, ReturnValue},
};

impl Compiler<'_> {
    pub fn control_stop_this_script(&mut self, builder: &mut FunctionBuilder<'_>) {
        for _ in 0..(self.repeat_stack * 2) {
            self.call_stack_pop(builder);
        }

        self.cache.save(builder, &mut self.constants);
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
        if effects.writes_is_unknown {
            todo!("Gotta implement fallback behaviour");
        }
        if effects.yields {
            todo!("Gotta implement yield behaviour");
        }

        let zero = self.constants.get_int(0, builder);
        let one = self.constants.get_int(1, builder);
        let old_constants = self.constants.clone();

        let loop_value = input.get_number_int(self, builder);

        let loop_block = builder.create_block();
        let end_block = builder.create_block();

        let final_params_loop =
            effects.generate_params(builder, loop_block, &|ptr| self.cache.get_type(ptr).into());
        let loop_value_param = builder.append_block_param(loop_block, I64); // loop counter
        let final_params_end =
            effects.generate_params(builder, end_block, &|ptr| self.cache.get_type(ptr).into());

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
        self.cache.variable_vals.extend(final_params_loop.clone());

        for block in blocks {
            self.compile_block(block, builder);
        }
        let loop_params = self.generate_params(builder, &final_params_loop);
        let mut loop_params2 = loop_params.clone();

        let new_loop_value = builder.ins().isub(loop_value_param, one);
        loop_params2.push(new_loop_value.into());

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
        self.cache.variable_vals.extend(final_params_end.clone());
        self.code_block = end_block;
        self.constants = old_constants;

        // // Basically,
        // //
        // // for (i = 0; i < number; i += 1) {
        // //      your code
        // // }
        // //
        // // The different parts will be annotated

        // let is_screen_refresh = vec.iter().any(|n| n.could_trigger_refresh());
        // let number = input.get_number_int(self, builder);

        // let loop_block = builder.create_block();
        // builder.append_block_param(loop_block, I64);
        // builder.append_block_param(loop_block, I64);
        // let body_block = builder.create_block();
        // builder.append_block_param(body_block, I64);
        // let end_block = builder.create_block();

        // // i = 0
        // // Note: counter is the `i` here
        // let counter = self.constants.get_int(0, builder);
        // builder
        //     .ins()
        //     .jump(loop_block, &[counter.into(), number.into()]);

        // builder.switch_to_block(loop_block);
        // // (i < number)
        // let counter = builder.block_params(loop_block)[0];
        // let mut number = builder.block_params(loop_block)[1];
        // let condition = builder.ins().icmp(IntCC::SignedLessThan, counter, number);

        // // if (i < number):
        // //      jump to body_block (continue)
        // // else:
        // //      jump to end_block (break)
        // builder
        //     .ins()
        //     .brif(condition, body_block, &[counter.into()], end_block, &[]);

        // builder.switch_to_block(body_block);
        // // i += 1
        // let counter = builder.block_params(body_block)[0];
        // let mut incremented = builder.ins().iadd_imm(counter, 1);

        // let mut inside_types = self.variable_type_data.clone();
        // self.update_type_data_for_block(&mut inside_types, vec);
        // let mut inside_types = common_entries(&inside_types, &self.variable_type_data);

        // let temp_block = self.code_block;
        // self.code_block = body_block;

        // std::mem::swap(&mut inside_types, &mut self.variable_type_data);

        // if is_screen_refresh {
        //     self.call_stack_push(builder, incremented);
        //     self.call_stack_push(builder, number);
        // }
        // self.constants.clear();
        // self.repeat_stack += 1;
        // for block in vec {
        //     self.compile_block(block, builder);
        // }
        // if is_screen_refresh && !vec.ends_with(&[ScratchBlock::ScreenRefresh]) {
        //     self.screen_refresh(builder);
        // }
        // self.repeat_stack -= 1;
        // if is_screen_refresh {
        //     number = self.call_stack_pop(builder);
        //     incremented = self.call_stack_pop(builder);
        // }
        // std::mem::swap(&mut inside_types, &mut self.variable_type_data);
        // self.code_block = temp_block;
        // self.variable_type_data = common_entries(&self.variable_type_data, &inside_types);
        // builder
        //     .ins()
        //     .jump(loop_block, &[incremented.into(), number.into()]);
        // // // builder.seal_block(body_block);
        // // builder.seal_block(loop_block);

        // builder.switch_to_block(end_block);
        // self.constants.clear();
        // self.code_block = end_block;
    }

    pub fn control_forever(&mut self, builder: &mut FunctionBuilder<'_>, blocks: &[ScratchBlock]) {
        let loop_block = builder.create_block();
        let end_block = builder.create_block();

        // TODO: inter-function analysis
        let effects = blocks.effects(&mut |_| Effects::unknown(), &|v| self.cache.get_type(v));

        if effects.writes_is_unknown {
            todo!("Gotta implement fallback behaviour");
        }
        let final_params =
            effects.generate_params(builder, loop_block, &|ptr| self.cache.get_type(ptr).into());

        let entry_params = self.generate_params(builder, &final_params);

        builder.ins().jump(loop_block, &entry_params);
        self.code_block = loop_block;
        builder.switch_to_block(loop_block);
        self.cache.variable_vals.extend(final_params.clone());

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
            callbacks::repeat_stack::stack_pop as *const (),
            &[I64],
            &[I64],
            &[self.loop_stack_ptr],
        );
        builder.inst_results(inst)[0]
    }

    fn call_stack_push(&mut self, builder: &mut FunctionBuilder<'_>, incremented: Value) {
        self.call_function(
            builder,
            callbacks::repeat_stack::stack_push as *const (),
            &[I64, I64],
            &[],
            &[self.loop_stack_ptr, incremented],
        );
    }

    pub fn update_type_data_for_block(
        &self,
        variable_type_data: &mut HashMap<Ptr, VarType>,
        code: &[ScratchBlock],
    ) {
        variable_type_data.clear();
        for var in (0..self.memory.len()).map(Ptr) {
            if let Some(var_type) = code
                .iter()
                .filter_map(|block| block.affects_var(var, variable_type_data))
                .next_back()
            {
                match var_type {
                    VarTypeChecked::Number => {
                        variable_type_data.insert(var, VarType::Number);
                    }
                    VarTypeChecked::Bool => {
                        variable_type_data.insert(var, VarType::Bool);
                    }
                    VarTypeChecked::String => {
                        variable_type_data.insert(var, VarType::String);
                    }
                    VarTypeChecked::Object => {
                        variable_type_data.remove(&var);
                    }
                }
            }
        }
    }

    pub fn control_if(
        &mut self,
        condition: &Input,
        builder: &mut FunctionBuilder<'_>,
        then: &[ScratchBlock],
    ) {
        // TODO: inter-function analysis
        let effects = then.effects(&mut |_| Effects::unknown(), &|v| self.cache.get_type(v));

        if effects.writes_is_unknown {
            todo!("Gotta implement fallback behaviour");
        }

        let inside_block = builder.create_block();
        let end_block = builder.create_block();
        let final_params =
            effects.generate_params(builder, end_block, &|ptr| self.cache.get_type(ptr).into());

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
        self.cache.variable_vals.extend(final_params);
    }

    fn generate_params(
        &mut self,
        builder: &mut FunctionBuilder<'_>,
        final_params: &Vec<(Ptr, ReturnValue)>,
    ) -> Vec<BlockArg> {
        let mut direct_params = Vec::new();
        for (ptr, param) in final_params {
            match param {
                ReturnValue::Num(_) => {
                    let val = self.cache.variable_vals.get(ptr).unwrap();
                    let ReturnValue::Num(v) = val else {
                        panic!("Not a number? Some type checking went wrong (val: {val:?})");
                    };
                    direct_params.push((*v).into());
                }
                ReturnValue::Bool(_) => {
                    let val = self.cache.variable_vals.get(ptr).unwrap();
                    let ReturnValue::Bool(v) = val else {
                        panic!("Not a bool? Some type checking went wrong (val: {val:?})");
                    };
                    direct_params.push((*v).into());
                }
                ReturnValue::String(_) => {
                    let val = self.cache.variable_vals.get(ptr).unwrap();
                    let ReturnValue::String([i1, i2, i3]) = val else {
                        panic!("Not a string? Some type checking went wrong (val: {val:?})");
                    };
                    direct_params.push((*i1).into());
                    direct_params.push((*i2).into());
                    direct_params.push((*i3).into());
                }
                ReturnValue::Object(_) => {
                    let obj = self.cache.variable_vals.get(ptr).unwrap();
                    let [i1, i2, i3, i4] = obj.get_object(builder, &mut self.constants);
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

        if effects.writes_is_unknown {
            todo!("Gotta implement fallback behaviour");
        }

        let then_block = builder.create_block();
        let else_block = builder.create_block();
        let end_block = builder.create_block();

        let final_params =
            effects.generate_params(builder, end_block, &|ptr| self.cache.get_type(ptr).into());

        let condition = condition.get_bool(self, builder);
        builder
            .ins()
            .brif(condition, then_block, &[], else_block, &[]);

        let old_consts = self.constants.clone();
        let old_cache = self.cache.clone();

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
        self.cache.variable_vals.extend(final_params);
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
        if effects.writes_is_unknown {
            todo!("Gotta implement fallback behaviour");
        }

        let final_params = effects.generate_params(builder, condition_block, &|ptr| {
            self.cache.get_type(ptr).into()
        });

        let entry_params = self.generate_params(builder, &final_params);
        builder.ins().jump(condition_block, &entry_params);

        self.cache.variable_vals.extend(final_params.clone());
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
        self.cache.variable_vals.extend(final_params);
        self.constants = old_constants;
    }
}
