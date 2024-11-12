use std::collections::HashMap;

use cranelift::prelude::*;
use types::I64;

use crate::{
    compiler::{compile_block, ScratchBlock, VarType, VarTypeChecked},
    data_types::ScratchObject,
    input_primitives::{Input, Ptr},
};

pub fn repeat(
    builder: &mut FunctionBuilder<'_>,
    input: &Input,
    code_block: &mut Block,
    variable_type_data: &mut HashMap<Ptr, VarType>,
    vec: &Vec<ScratchBlock>,
    memory: &[ScratchObject],
) {
    let loop_block = builder.create_block();
    builder.append_block_param(loop_block, I64);
    let mut body_block = builder.create_block();
    builder.append_block_param(body_block, I64);
    let end_block = builder.create_block();

    let number = input.get_number(builder, code_block, variable_type_data, memory);
    let number = builder.ins().fcvt_to_sint(I64, number);

    let counter = builder.ins().iconst(I64, 0);
    builder.ins().jump(loop_block, &[counter]);
    builder.seal_block(*code_block);

    builder.switch_to_block(loop_block);
    // (counter < number)
    let counter = builder.block_params(loop_block)[0];
    let condition = builder.ins().icmp(IntCC::SignedLessThan, counter, number);

    // if (counter < number) jump to body_block else jump to end_block
    builder
        .ins()
        .brif(condition, body_block, &[counter], end_block, &[]);

    builder.switch_to_block(body_block);
    let counter = builder.block_params(body_block)[0];
    let incremented = builder.ins().iadd_imm(counter, 1);

    let mut inside_types = variable_type_data.clone();
    update_type_data_for_block(&mut inside_types, memory, vec);
    let mut inside_types = common_entries(&inside_types, variable_type_data);
    for block in vec {
        compile_block(block, builder, &mut body_block, &mut inside_types, memory);
    }
    *variable_type_data = common_entries(variable_type_data, &inside_types);
    builder.ins().jump(loop_block, &[incremented]);
    builder.seal_block(body_block);
    builder.seal_block(loop_block);

    builder.switch_to_block(end_block);
    *code_block = end_block;
}

fn update_type_data_for_block(
    variable_type_data: &mut HashMap<Ptr, VarType>,
    memory: &[ScratchObject],
    vec: &[ScratchBlock],
) {
    variable_type_data.clear();
    for var in (0..memory.len()).map(Ptr) {
        if let Some(var_type) = vec
            .iter()
            .filter_map(|block| block.affects_var(var, variable_type_data))
            .last()
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
                VarTypeChecked::Unknown => {
                    variable_type_data.remove(&var);
                }
            }
        }
    }
}

pub fn if_statement(
    input: &Input,
    builder: &mut FunctionBuilder<'_>,
    code_block: &mut Block,
    variable_type_data: &mut HashMap<Ptr, VarType>,
    vec: &Vec<ScratchBlock>,
    memory: &[ScratchObject],
) {
    let input = input.get_bool(builder, code_block, variable_type_data, memory);
    let mut inside_block = builder.create_block();
    let end_block = builder.create_block();

    builder.ins().brif(input, inside_block, &[], end_block, &[]);
    builder.seal_block(*code_block);

    builder.switch_to_block(inside_block);

    let mut types = variable_type_data.clone();
    for block in vec {
        compile_block(block, builder, &mut inside_block, &mut types, memory);
    }
    // Only keep the variable type data that hasn't been changed by the if statement.
    *variable_type_data = common_entries(variable_type_data, &types);

    builder.ins().jump(end_block, &[]);
    builder.seal_block(inside_block);

    builder.switch_to_block(end_block);
    *code_block = end_block;
}

fn common_entries<K, V>(map1: &HashMap<K, V>, map2: &HashMap<K, V>) -> HashMap<K, V>
where
    K: Eq + std::hash::Hash + Clone,
    V: PartialEq + Clone,
{
    let mut common_map = HashMap::new();
    for (key, value) in map1 {
        if let Some(other_value) = map2.get(key) {
            if value == other_value {
                common_map.insert(key.clone(), value.clone());
            }
        }
    }
    common_map
}

pub fn if_else(
    input: &Input,
    builder: &mut FunctionBuilder<'_>,
    code_block: &mut Block,
    variable_type_data: &mut HashMap<Ptr, VarType>,
    memory: &[ScratchObject],
    then_blocks: &Vec<ScratchBlock>,
    else_blocks: &Vec<ScratchBlock>,
) {
    let input = input.get_bool(builder, code_block, variable_type_data, memory);
    let mut inside_block = builder.create_block();
    let mut else_block = builder.create_block();
    let end_block = builder.create_block();

    builder
        .ins()
        .brif(input, inside_block, &[], else_block, &[]);
    builder.seal_block(*code_block);

    builder.switch_to_block(inside_block);
    let mut then_block_types = variable_type_data.clone();
    for block in then_blocks {
        compile_block(
            block,
            builder,
            &mut inside_block,
            &mut then_block_types,
            memory,
        );
    }
    builder.ins().jump(end_block, &[]);
    builder.seal_block(inside_block);

    builder.switch_to_block(else_block);
    let mut else_block_types = variable_type_data.clone();
    for block in else_blocks {
        compile_block(
            block,
            builder,
            &mut else_block,
            &mut else_block_types,
            memory,
        );
    }
    let common_types = common_entries(&then_block_types, &else_block_types);
    *variable_type_data = common_entries(variable_type_data, &common_types);
    builder.ins().jump(end_block, &[]);
    builder.seal_block(else_block);

    builder.switch_to_block(end_block);
    *code_block = end_block;
}

pub fn repeat_until(
    builder: &mut FunctionBuilder<'_>,
    code_block: &mut Block,
    input: &Input,
    variable_type_data: &mut HashMap<Ptr, VarType>,
    memory: &[ScratchObject],
    vec: &Vec<ScratchBlock>,
) {
    let loop_block = builder.create_block();
    let mut body_block = builder.create_block();
    let end_block = builder.create_block();
    builder.ins().jump(loop_block, &[]);
    builder.seal_block(*code_block);

    builder.switch_to_block(loop_block);
    let condition = input.get_bool(builder, code_block, variable_type_data, memory);
    builder
        .ins()
        .brif(condition, end_block, &[], body_block, &[]);

    builder.switch_to_block(body_block);

    let mut inside_types = variable_type_data.clone();
    update_type_data_for_block(&mut inside_types, memory, vec);
    let mut inside_types = common_entries(&inside_types, variable_type_data);
    for block in vec {
        compile_block(block, builder, &mut body_block, &mut inside_types, memory);
    }
    *variable_type_data = common_entries(variable_type_data, &inside_types);

    builder.ins().jump(loop_block, &[]);
    builder.seal_block(body_block);
    builder.seal_block(loop_block);

    builder.switch_to_block(end_block);
    *code_block = end_block;
}
