use std::collections::HashMap;

use cranelift::prelude::*;
use types::I64;

use crate::{
    callbacks,
    compiler::{Compiler, ScratchBlock, VarType, VarTypeChecked},
    input_primitives::{Input, Ptr},
};

pub fn repeat(
    compiler: &mut Compiler,
    builder: &mut FunctionBuilder<'_>,
    input: &Input,
    vec: &Vec<ScratchBlock>,
) {
    let loop_block = builder.create_block();
    builder.append_block_param(loop_block, I64);
    builder.append_block_param(loop_block, I64);
    let body_block = builder.create_block();
    builder.append_block_param(body_block, I64);
    let end_block = builder.create_block();

    let number = input.get_number(compiler, builder);
    let number = builder.ins().fcvt_to_sint(I64, number);

    let counter = compiler.constants.get_int(0, builder);
    builder.ins().jump(loop_block, &[counter, number]);
    // builder.seal_block(compiler.code_block);

    builder.switch_to_block(loop_block);
    // (counter < number)
    let counter = builder.block_params(loop_block)[0];
    let number = builder.block_params(loop_block)[1];
    let condition = builder.ins().icmp(IntCC::SignedLessThan, counter, number);

    // if (counter < number) jump to body_block else jump to end_block
    builder
        .ins()
        .brif(condition, body_block, &[counter], end_block, &[]);

    builder.switch_to_block(body_block);
    let counter = builder.block_params(body_block)[0];
    let incremented = builder.ins().iadd_imm(counter, 1);

    let mut inside_types = compiler.variable_type_data.clone();
    update_type_data_for_block(compiler, &mut inside_types, vec);
    let mut inside_types = common_entries(&inside_types, &compiler.variable_type_data);

    let temp_block = compiler.code_block;
    compiler.code_block = body_block;

    std::mem::swap(&mut inside_types, &mut compiler.variable_type_data);

    call_stack_push(compiler, builder, incremented);
    call_stack_push(compiler, builder, number);
    compiler.constants.clear();
    for block in vec {
        compiler.compile_block(block, builder);
    }
    let number = call_stack_pop(compiler, builder);
    let incremented = call_stack_pop(compiler, builder);
    std::mem::swap(&mut inside_types, &mut compiler.variable_type_data);
    compiler.code_block = temp_block;
    compiler.variable_type_data = common_entries(&compiler.variable_type_data, &inside_types);
    builder.ins().jump(loop_block, &[incremented, number]);
    // // builder.seal_block(body_block);
    // builder.seal_block(loop_block);

    builder.switch_to_block(end_block);
    compiler.constants.clear();
    compiler.code_block = end_block;
}

fn call_stack_pop(compiler: &mut Compiler<'_>, builder: &mut FunctionBuilder<'_>) -> Value {
    let inst = compiler.call_function(
        builder,
        callbacks::repeat_stack::stack_pop as usize,
        &[I64],
        &[I64],
        &[compiler.vec_ptr],
    );
    builder.inst_results(inst)[0]
}

fn call_stack_push(
    compiler: &mut Compiler<'_>,
    builder: &mut FunctionBuilder<'_>,
    incremented: Value,
) {
    compiler.call_function(
        builder,
        callbacks::repeat_stack::stack_push as usize,
        &[I64, I64],
        &[],
        &[compiler.vec_ptr, incremented],
    );
}

pub fn update_type_data_for_block(
    compiler: &Compiler,
    variable_type_data: &mut HashMap<Ptr, VarType>,
    code: &[ScratchBlock],
) {
    variable_type_data.clear();
    for var in (0..compiler.memory.len()).map(Ptr) {
        if let Some(var_type) = code
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
    then: &Vec<ScratchBlock>,
) {
    let input = input.get_bool(compiler, builder);
    let inside_block = builder.create_block();
    let end_block = builder.create_block();

    compiler.constants.clear();
    builder.ins().brif(input, inside_block, &[], end_block, &[]);
    // builder.seal_block(compiler.code_block);

    builder.switch_to_block(inside_block);

    let temp_types = compiler.variable_type_data.clone();
    let temp_block = compiler.code_block;
    compiler.code_block = inside_block;
    for block in then {
        compiler.compile_block(block, builder);
    }
    compiler.code_block = temp_block;

    // Only keep the variable type data that hasn't been changed by the if statement.
    // For example:

    // var a = String;
    // var b = Bool;
    // if condition {
    //     var a = Number;
    //     var b = Bool;
    // }

    // Here, the compiler can't tell beforehand if the condition will run.
    // So it can't tell the type of variable a.

    // But the type of variable b doesn't change inside the condition.
    // So the compiler remembers the type of variable b.
    compiler.variable_type_data = common_entries(&compiler.variable_type_data, &temp_types);

    builder.ins().jump(end_block, &[]);

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

    then_blocks: &Vec<ScratchBlock>,
    else_blocks: &Vec<ScratchBlock>,
) {
    let input = input.get_bool(compiler, builder);
    let inside_block = builder.create_block();
    let else_block = builder.create_block();
    let end_block = builder.create_block();

    // If condition then { jump to inside block } else { jump to else block }.
    builder
        .ins()
        .brif(input, inside_block, &[], else_block, &[]);
    compiler.constants.clear();
    // builder.seal_block(compiler.code_block);

    builder.switch_to_block(inside_block);

    // Temporarily store the old type data from before the then block.
    // Will be used later.
    let old_types = compiler.variable_type_data.clone();
    let current_block = compiler.code_block;
    compiler.code_block = inside_block;

    for block in then_blocks {
        compiler.compile_block(block, builder);
    }

    compiler.code_block = current_block;
    let common_then_entries = common_entries(&compiler.variable_type_data, &old_types);
    builder.ins().jump(end_block, &[]);
    // builder.seal_block(inside_block);

    builder.switch_to_block(else_block);
    compiler.constants.clear();
    compiler.variable_type_data.clone_from(&old_types);

    compiler.code_block = else_block;
    for block in else_blocks {
        compiler.compile_block(block, builder);
    }
    compiler.code_block = current_block;

    compiler.variable_type_data = common_entries(&old_types, &compiler.variable_type_data);
    compiler.variable_type_data =
        common_entries(&common_then_entries, &compiler.variable_type_data);

    builder.ins().jump(end_block, &[]);
    // builder.seal_block(else_block);

    builder.switch_to_block(end_block);
    compiler.constants.clear();
    compiler.code_block = end_block;
}

pub fn repeat_until(
    compiler: &mut Compiler,
    builder: &mut FunctionBuilder<'_>,
    input: &Input,

    body: &Vec<ScratchBlock>,
) {
    let loop_block = builder.create_block();
    let body_block = builder.create_block();
    let end_block = builder.create_block();

    builder.ins().jump(loop_block, &[]);
    compiler.constants.clear();
    // builder.seal_block(compiler.code_block);

    builder.switch_to_block(loop_block);
    let condition = input.get_bool(compiler, builder);
    compiler.constants.clear();
    builder
        .ins()
        .brif(condition, end_block, &[], body_block, &[]);

    builder.switch_to_block(body_block);

    let mut inside_types = compiler.variable_type_data.clone();
    update_type_data_for_block(compiler, &mut inside_types, body);
    let old_types = compiler.variable_type_data.clone();

    let current_block = compiler.code_block;
    compiler.code_block = body_block;
    for block in body {
        compiler.compile_block(block, builder);
    }
    compiler.code_block = current_block;
    compiler.variable_type_data = common_entries(&compiler.variable_type_data, &old_types);

    builder.ins().jump(loop_block, &[]);
    // builder.seal_block(loop_block);

    builder.switch_to_block(end_block);
    compiler.constants.clear();
    compiler.code_block = end_block;
}
