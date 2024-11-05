use std::collections::HashMap;

use codegen::{
    control::ControlPlane,
    ir::{Function, UserFuncName},
};
use cranelift::prelude::*;
use isa::CallConv;
use lazy_static::lazy_static;
use target_lexicon::Triple;
use types::{F64, I64};

use crate::{
    callbacks,
    compiler_blocks::{c_var_read, c_var_set},
    data_types::{self, ScratchObject, ID_NUMBER, ID_STRING},
    input_primitives::{Input, Ptr, ReturnValue},
    test_programs,
};

lazy_static! {
    pub static ref MEMORY: Box<[ScratchObject]> =
        vec![ScratchObject::Number(0.0); 1024].into_boxed_slice();
}

pub struct Compiler {
    // pub json: JsonStruct,
    // pub project_dir: TempDir,
}

#[derive(Debug)]
pub enum ScratchBlock {
    WhenFlagClicked,
    VarSet(Ptr, Input),
    VarChange(Ptr, Input),
    VarRead(Ptr),
    OpAdd(Input, Input),
    OpSub(Input, Input),
    OpMul(Input, Input),
    OpDiv(Input, Input),
    OpJoin(Input, Input),
    OpMod(Input, Input),
    ControlRepeat(Input, Vec<ScratchBlock>),
}

#[derive(Debug)]
pub enum VarType {
    Number,
    Bool,
    String,
}

struct CodeSprite {
    pub scripts: Vec<Vec<ScratchBlock>>,
}

impl Compiler {
    pub fn new() -> Self {
        // let file_bytes = std::fs::read(path).unwrap();

        // let loaded_project_dir = tempfile::TempDir::new().unwrap();

        // zip_extract::extract(
        //     std::io::Cursor::new(file_bytes),
        //     loaded_project_dir.path(),
        //     false,
        // )
        // .unwrap();

        // let json = std::fs::read_to_string(loaded_project_dir.path().join("project.json")).unwrap();
        // let json: JsonStruct = serde_json::from_str(&json).unwrap();

        Self {
            // json,
            // project_dir: loaded_project_dir,
        }
    }

    pub fn compile(&self) {
        let mut builder = settings::builder();
        builder.set("opt_level", "speed").unwrap();
        let flags = settings::Flags::new(builder);

        let isa = match isa::lookup(Triple::host()) {
            Err(err) => panic!("Error looking up target: {}", err),
            Ok(isa_builder) => isa_builder.finish(flags).unwrap(),
        };

        // let code_sprites = self.get_block_code();
        let code_sprites = vec![CodeSprite {
            scripts: vec![test_programs::repeated_join_string()],
        }];
        for sprite in &code_sprites {
            for script in &sprite.scripts {
                let sig = Signature::new(CallConv::SystemV);
                let mut func = Function::with_name_signature(UserFuncName::default(), sig);

                let mut func_ctx = FunctionBuilderContext::new();
                let mut builder = FunctionBuilder::new(&mut func, &mut func_ctx);

                let mut code_block = builder.create_block();

                builder.append_block_params_for_function_params(code_block);
                builder.switch_to_block(code_block);

                let mut variable_type_data: HashMap<Ptr, VarType> = HashMap::new();

                for block in script {
                    compile_block(
                        block,
                        &mut builder,
                        &mut code_block,
                        &mut variable_type_data,
                    );
                }

                builder.seal_block(code_block);

                let ins = builder.ins();
                ins.return_(&[]);

                builder.finalize();

                println!("{}", func.display());

                let mut ctx = codegen::Context::for_function(func);
                let mut plane = ControlPlane::default();
                ctx.optimize(isa.as_ref(), &mut plane).unwrap();

                let code = ctx.compile(&*isa, &mut plane).unwrap();

                let mut buffer = memmap2::MmapOptions::new()
                    .len(code.code_buffer().len())
                    .map_anon()
                    .unwrap();

                buffer.copy_from_slice(code.code_buffer());

                // Machine code dump
                // let ptr = buffer.as_ptr();
                // let bytes = unsafe { std::slice::from_raw_parts(ptr, code.code_buffer().len()) };
                // for (_i, byte) in bytes.iter().enumerate() {
                //     print!("{:#04x} ", byte);
                // }
                // println!();
                // std::fs::write("func.bin", bytes).unwrap();

                let buffer = buffer.make_exec().unwrap();

                unsafe {
                    let code_fn: unsafe extern "sysv64" fn() = std::mem::transmute(buffer.as_ptr());

                    let instant = std::time::Instant::now();
                    code_fn();
                    println!("Time: {:?}", instant.elapsed());
                    println!("Types: {variable_type_data:?}");
                }
            }
        }
    }
}

pub fn compile_block(
    block: &ScratchBlock,
    builder: &mut FunctionBuilder<'_>,
    code_block: &mut Block,
    variable_type_data: &mut HashMap<Ptr, VarType>,
) -> Option<ReturnValue> {
    match block {
        ScratchBlock::WhenFlagClicked => {}
        ScratchBlock::VarSet(ptr, obj) => {
            c_var_set(obj, builder, ptr, variable_type_data, code_block);
        }
        ScratchBlock::OpAdd(a, b) => {
            let a = a.get_number(builder, code_block, variable_type_data);
            let b = b.get_number(builder, code_block, variable_type_data);
            let res = builder.ins().fadd(a, b);
            return Some(ReturnValue::Num(res));
        }
        ScratchBlock::OpSub(a, b) => {
            let a = a.get_number(builder, code_block, variable_type_data);
            let b = b.get_number(builder, code_block, variable_type_data);
            let res = builder.ins().fsub(a, b);
            return Some(ReturnValue::Num(res));
        }
        ScratchBlock::OpMul(a, b) => {
            let a = a.get_number(builder, code_block, variable_type_data);
            let b = b.get_number(builder, code_block, variable_type_data);
            let res = builder.ins().fmul(a, b);
            return Some(ReturnValue::Num(res));
        }
        ScratchBlock::OpDiv(a, b) => {
            let a = a.get_number(builder, code_block, variable_type_data);
            let b = b.get_number(builder, code_block, variable_type_data);
            let res = builder.ins().fdiv(a, b);
            return Some(ReturnValue::Num(res));
        }
        ScratchBlock::OpMod(a, b) => {
            let a = a.get_number(builder, code_block, variable_type_data);
            let b = b.get_number(builder, code_block, variable_type_data);
            let div = builder.ins().fdiv(a, b);

            // Step 1: Truncate the division to an integer (simulates `floor` for positive values)
            let trunc_div = builder.ins().fcvt_to_sint(I64, div);
            let trunc_div = builder.ins().fcvt_from_sint(F64, trunc_div);

            // Step 2: Check if truncation needs adjustment for negative values
            // If `trunc_div > div`, we adjust by subtracting 1 to simulate `floor`
            let needs_adjustment = builder.ins().fcmp(FloatCC::GreaterThan, trunc_div, div);
            let tmp = builder.ins().f64const(-1.0);
            let adjustment = builder.ins().fadd(trunc_div, tmp);
            let floor_div = builder
                .ins()
                .select(needs_adjustment, adjustment, trunc_div);

            // Step 3: Calculate the decimal part and modulo as before
            let decimal_part = builder.ins().fsub(div, floor_div);
            let modulo = builder.ins().fmul(decimal_part, b);

            return Some(ReturnValue::Num(modulo));
        }
        ScratchBlock::VarRead(ptr) => {
            return Some(c_var_read(builder, *ptr, variable_type_data));
        }
        ScratchBlock::OpJoin(a, b) => {
            // Get strings
            let (a, a_is_const) = a.get_string(builder, code_block, variable_type_data);
            let (b, b_is_const) = b.get_string(builder, code_block, variable_type_data);

            // Create stack slot for result
            let stack_slot = builder.create_sized_stack_slot(StackSlotData::new(
                StackSlotKind::ExplicitSlot,
                3 * std::mem::size_of::<i64>() as u32,
                0,
            ));
            let stack_ptr = builder.ins().stack_addr(I64, stack_slot, 0);

            // Call join_string function
            let func = builder.ins().iconst(I64, callbacks::op_join_string as i64);
            let sig = builder.import_signature({
                let mut sig = Signature::new(CallConv::SystemV);
                sig.params.push(AbiParam::new(I64));
                sig.params.push(AbiParam::new(I64));
                sig.params.push(AbiParam::new(I64));
                sig.params.push(AbiParam::new(I64));
                sig.params.push(AbiParam::new(I64));
                sig
            });
            let a_is_const = builder.ins().iconst(I64, a_is_const as i64);
            let b_is_const = builder.ins().iconst(I64, b_is_const as i64);
            builder
                .ins()
                .call_indirect(sig, func, &[a, b, stack_ptr, a_is_const, b_is_const]);

            // Read resulting string
            let id = builder.ins().iconst(I64, ID_STRING as i64);
            let i1 = builder.ins().stack_load(I64, stack_slot, 0);
            let i2 = builder.ins().stack_load(I64, stack_slot, 8);
            let i3 = builder.ins().stack_load(I64, stack_slot, 16);

            return Some(ReturnValue::Object((id, i1, i2, i3)));
        }
        ScratchBlock::ControlRepeat(input, vec) => {
            let loop_block = builder.create_block();
            builder.append_block_param(loop_block, I64);
            let mut body_block = builder.create_block();
            builder.append_block_param(body_block, I64);
            let end_block = builder.create_block();

            let number = input.get_number(builder, code_block, variable_type_data);
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
                compile_block(block, builder, &mut body_block, variable_type_data);
            }
            builder.ins().jump(loop_block, &[incremented]);
            builder.seal_block(body_block);
            builder.seal_block(loop_block);

            builder.switch_to_block(end_block);
            *code_block = end_block;
        }
        ScratchBlock::VarChange(ptr, input) => {
            let input = input.get_number(builder, code_block, variable_type_data);
            let old_value = c_var_read(builder, *ptr, variable_type_data);
            let old_value = old_value.get_number(builder);
            let new_value = builder.ins().fadd(old_value, input);

            let mem_ptr = builder
                .ins()
                .iconst(I64, unsafe { MEMORY.as_ptr().offset(ptr.0 as isize) }
                    as i64);

            builder.ins().store(MemFlags::new(), new_value, mem_ptr, 8);

            if !matches!(variable_type_data.get(ptr), Some(VarType::Number)) {
                let id = builder.ins().iconst(I64, ID_NUMBER as i64);
                builder.ins().store(MemFlags::new(), id, mem_ptr, 0);
            }
        }
    }
    None
}

pub fn print_func_addresses() {
    println!("var_read: {:X}", callbacks::var_read as usize);
    println!("op_join_string: {:X}", callbacks::op_join_string as usize);
    println!(
        "to_string_from_num: {:X}",
        callbacks::types::to_string_from_num as usize
    );
    println!("to_string: {:X}", callbacks::types::to_string as usize);
    println!(
        "to_string_from_bool: {:X}",
        callbacks::types::to_string_from_bool as usize
    );
    println!("to_number: {:X}", callbacks::types::to_number as usize);
    println!("drop_obj: {:X}", callbacks::types::drop_obj as usize);
}
