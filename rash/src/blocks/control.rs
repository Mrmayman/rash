use std::collections::HashMap;

use cranelift::prelude::*;
use types::I64;

use crate::{
    compiler::{Compiler, ScratchBlock, VarType, VarTypeChecked},
    data_types::ScratchObject,
    input_primitives::{Input, Ptr},
};

pub fn repeat(
    compiler: &mut Compiler,
    builder: &mut FunctionBuilder<'_>,
    input: &Input,
    vec: &Vec<ScratchBlock>,
    memory: &[ScratchObject],
) {
    let loop_block = builder.create_block();
    builder.append_block_param(loop_block, I64);
    let body_block = builder.create_block();
    builder.append_block_param(body_block, I64);
    let end_block = builder.create_block();

    let number = input.get_number(compiler, builder, memory);
    let number = builder.ins().fcvt_to_sint(I64, number);

    let counter = compiler.constants.get_int(0, builder);
    compiler.constants.clear();
    builder.ins().jump(loop_block, &[counter]);
    builder.seal_block(compiler.code_block);

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

    let mut inside_types = compiler.variable_type_data.clone();
    update_type_data_for_block(&mut inside_types, memory, vec);
    let mut inside_types = common_entries(&inside_types, &compiler.variable_type_data);

    let temp_block = compiler.code_block;
    compiler.code_block = body_block;

    std::mem::swap(&mut inside_types, &mut compiler.variable_type_data);

    for block in vec {
        compiler.compile_block(block, builder, memory);
    }
    std::mem::swap(&mut inside_types, &mut compiler.variable_type_data);
    compiler.code_block = temp_block;
    compiler.variable_type_data = common_entries(&compiler.variable_type_data, &inside_types);
    builder.ins().jump(loop_block, &[incremented]);
    // builder.seal_block(body_block);
    builder.seal_block(loop_block);

    builder.switch_to_block(end_block);
    compiler.constants.clear();
    compiler.code_block = end_block;
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
    compiler: &mut Compiler,
    input: &Input,
    builder: &mut FunctionBuilder<'_>,
    vec: &Vec<ScratchBlock>,
    memory: &[ScratchObject],
) {
    let input = input.get_bool(compiler, builder, memory);
    let inside_block = builder.create_block();
    let end_block = builder.create_block();

    compiler.constants.clear();
    builder.ins().brif(input, inside_block, &[], end_block, &[]);
    builder.seal_block(compiler.code_block);

    builder.switch_to_block(inside_block);

    let temp_types = compiler.variable_type_data.clone();
    let temp_block = compiler.code_block;
    compiler.code_block = inside_block;
    for block in vec {
        compiler.compile_block(block, builder, memory);
    }
    compiler.code_block = temp_block;
    // Only keep the variable type data that hasn't been changed by the if statement.
    compiler.variable_type_data = common_entries(&compiler.variable_type_data, &temp_types);

    builder.ins().jump(end_block, &[]);
    // builder.seal_block(inside_block);

    builder.switch_to_block(end_block);
    compiler.constants.clear();
    compiler.code_block = end_block;
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
    compiler: &mut Compiler,
    input: &Input,
    builder: &mut FunctionBuilder<'_>,
    memory: &[ScratchObject],
    then_blocks: &Vec<ScratchBlock>,
    else_blocks: &Vec<ScratchBlock>,
) {
    let input = input.get_bool(compiler, builder, memory);
    let inside_block = builder.create_block();
    let else_block = builder.create_block();
    let end_block = builder.create_block();

    builder
        .ins()
        .brif(input, inside_block, &[], else_block, &[]);
    compiler.constants.clear();
    builder.seal_block(compiler.code_block);

    builder.switch_to_block(inside_block);
    let old_types = compiler.variable_type_data.clone();
    let current_block = compiler.code_block;
    compiler.code_block = inside_block;
    for block in then_blocks {
        compiler.compile_block(block, builder, memory);
    }
    compiler.code_block = current_block;
    builder.ins().jump(end_block, &[]);
    builder.seal_block(inside_block);

    builder.switch_to_block(else_block);
    compiler.constants.clear();
    compiler.variable_type_data = old_types.clone();

    compiler.code_block = else_block;
    for block in else_blocks {
        compiler.compile_block(block, builder, memory);
    }
    compiler.code_block = current_block;

    compiler.variable_type_data = common_entries(&old_types, &compiler.variable_type_data);
    builder.ins().jump(end_block, &[]);
    builder.seal_block(else_block);

    builder.switch_to_block(end_block);
    compiler.constants.clear();
    compiler.code_block = end_block;
}

pub fn repeat_until(
    compiler: &mut Compiler,
    builder: &mut FunctionBuilder<'_>,
    input: &Input,
    memory: &[ScratchObject],
    vec: &Vec<ScratchBlock>,
) {
    let loop_block = builder.create_block();
    let body_block = builder.create_block();
    let end_block = builder.create_block();
    builder.ins().jump(loop_block, &[]);
    compiler.constants.clear();
    builder.seal_block(compiler.code_block);

    builder.switch_to_block(loop_block);
    let condition = input.get_bool(compiler, builder, memory);
    compiler.constants.clear();
    builder
        .ins()
        .brif(condition, end_block, &[], body_block, &[]);

    builder.switch_to_block(body_block);

    let mut inside_types = compiler.variable_type_data.clone();
    update_type_data_for_block(&mut inside_types, memory, vec);
    let old_types = compiler.variable_type_data.clone();

    let current_block = compiler.code_block;
    compiler.code_block = body_block;
    for block in vec {
        compiler.compile_block(block, builder, memory);
    }
    compiler.code_block = current_block;
    compiler.variable_type_data = common_entries(&compiler.variable_type_data, &old_types);

    builder.ins().jump(loop_block, &[]);
    // builder.seal_block(body_block);
    builder.seal_block(loop_block);

    builder.switch_to_block(end_block);
    compiler.constants.clear();
    compiler.code_block = end_block;
}
