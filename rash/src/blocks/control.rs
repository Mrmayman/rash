use std::collections::HashMap;

use cranelift::prelude::*;
use types::I64;

use crate::{
    compiler::{compile_block, ScratchBlock, VarType},
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
    for block in vec {
        compile_block(block, builder, &mut body_block, variable_type_data, memory);
    }
    builder.ins().jump(loop_block, &[incremented]);
    builder.seal_block(body_block);
    builder.seal_block(loop_block);

    builder.switch_to_block(end_block);
    *code_block = end_block;
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
    for block in vec {
        compile_block(
            block,
            builder,
            &mut inside_block,
            variable_type_data,
            memory,
        );
    }
    builder.ins().jump(end_block, &[]);
    builder.seal_block(inside_block);

    builder.switch_to_block(end_block);
    *code_block = end_block;
}

pub fn if_else(
    input: &Input,
    builder: &mut FunctionBuilder<'_>,
    code_block: &mut Block,
    variable_type_data: &mut HashMap<Ptr, VarType>,
    memory: &[ScratchObject],
    vec: &Vec<ScratchBlock>,
    vec1: &Vec<ScratchBlock>,
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
    for block in vec {
        compile_block(
            block,
            builder,
            &mut inside_block,
            variable_type_data,
            memory,
        );
    }
    builder.ins().jump(end_block, &[]);
    builder.seal_block(inside_block);

    builder.switch_to_block(else_block);
    for block in vec1 {
        compile_block(block, builder, &mut else_block, variable_type_data, memory);
    }
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
    for block in vec {
        compile_block(block, builder, &mut body_block, variable_type_data, memory);
    }
    builder.ins().jump(loop_block, &[]);
    builder.seal_block(body_block);
    builder.seal_block(loop_block);

    builder.switch_to_block(end_block);
    *code_block = end_block;
}
