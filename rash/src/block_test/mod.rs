use std::collections::HashMap;

use crate::{
    compiler::{Compiler, ScratchBlock, VarType},
    data_types::ScratchObject,
    input_primitives::Ptr,
};

use codegen::{
    control::ControlPlane,
    ir::{Function, UserFuncName},
};
use cranelift::prelude::*;
use isa::CallConv;
use target_lexicon::Triple;

#[allow(unused)]
pub fn repeated_sum() -> Vec<ScratchBlock> {
    vec![
        ScratchBlock::WhenFlagClicked,
        ScratchBlock::VarSet(
            Ptr(7),
            ScratchBlock::OpAdd(ScratchBlock::VarRead(Ptr(7)).into(), false.into()).into(),
        ),
        ScratchBlock::ControlRepeat(
            100_000.0.into(),
            vec![
                ScratchBlock::VarSet(
                    Ptr(7),
                    ScratchBlock::OpAdd(ScratchBlock::VarRead(Ptr(7)).into(), true.into()).into(),
                ),
                ScratchBlock::VarSet(
                    Ptr(7),
                    ScratchBlock::OpAdd(ScratchBlock::VarRead(Ptr(7)).into(), true.into()).into(),
                ),
            ],
        ),
    ]
}

#[allow(unused)]
pub fn repeated_join_string() -> Vec<ScratchBlock> {
    vec![
        ScratchBlock::WhenFlagClicked,
        ScratchBlock::VarSet(Ptr(7), "hello ".into()),
        ScratchBlock::ControlRepeat(
            100.0.into(),
            vec![
                ScratchBlock::VarSet(
                    Ptr(7),
                    ScratchBlock::OpStrJoin(ScratchBlock::VarRead(Ptr(7)).into(), "world".into())
                        .into(),
                ),
                ScratchBlock::VarSet(
                    Ptr(7),
                    ScratchBlock::OpStrJoin(ScratchBlock::VarRead(Ptr(7)).into(), ", ".into())
                        .into(),
                ),
            ],
        ),
    ]
}

#[allow(unused)]
pub fn if_test() -> Vec<ScratchBlock> {
    vec![
        ScratchBlock::WhenFlagClicked,
        ScratchBlock::ControlIf(1.0.into(), vec![ScratchBlock::VarSet(Ptr(0), 1.0.into())]),
        ScratchBlock::ControlIf(0.0.into(), vec![ScratchBlock::VarSet(Ptr(1), 1.0.into())]),
        ScratchBlock::ControlIf(true.into(), vec![ScratchBlock::VarSet(Ptr(2), 1.0.into())]),
        ScratchBlock::ControlIf(false.into(), vec![ScratchBlock::VarSet(Ptr(3), 1.0.into())]),
        ScratchBlock::ControlIf(
            "hello".into(),
            vec![ScratchBlock::VarSet(Ptr(4), 1.0.into())],
        ),
        ScratchBlock::ControlIf(
            String::new().into(),
            vec![ScratchBlock::VarSet(Ptr(5), 1.0.into())],
        ),
        ScratchBlock::ControlIf(
            "true".into(),
            vec![ScratchBlock::VarSet(Ptr(6), 1.0.into())],
        ),
        ScratchBlock::ControlIf(
            "false".into(),
            vec![ScratchBlock::VarSet(Ptr(7), 1.0.into())],
        ),
        // nested statements
        ScratchBlock::ControlIf(
            true.into(),
            vec![
                ScratchBlock::ControlIf(
                    true.into(),
                    vec![ScratchBlock::VarSet(Ptr(8), 1.0.into())],
                ),
                ScratchBlock::ControlIf(
                    false.into(),
                    vec![ScratchBlock::VarSet(Ptr(9), 1.0.into())],
                ),
            ],
        ),
        ScratchBlock::ControlIf(
            f64::NAN.into(),
            vec![ScratchBlock::VarSet(Ptr(10), 1.0.into())],
        ),
        ScratchBlock::ControlIf(
            ScratchBlock::OpDiv(0.0.into(), 0.0.into()).into(),
            vec![ScratchBlock::VarSet(Ptr(11), 1.0.into())],
        ),
    ]
}

// Pass: *1-*5 true
#[allow(unused)]
pub fn if_else_test() -> Vec<ScratchBlock> {
    vec![
        ScratchBlock::WhenFlagClicked,
        ScratchBlock::ControlIfElse(
            true.into(),
            vec![ScratchBlock::VarSet(Ptr(0), 1.0.into())],
            vec![ScratchBlock::VarSet(Ptr(0), 0.0.into())],
        ),
        ScratchBlock::ControlIfElse(
            false.into(),
            vec![ScratchBlock::VarSet(Ptr(1), 0.0.into())],
            vec![ScratchBlock::VarSet(Ptr(1), 1.0.into())],
        ),
        ScratchBlock::ControlIfElse(
            "hello".into(),
            vec![ScratchBlock::VarSet(Ptr(2), 1.0.into())],
            vec![ScratchBlock::VarSet(Ptr(2), 0.0.into())],
        ),
        ScratchBlock::ControlIfElse(
            String::new().into(),
            vec![ScratchBlock::VarSet(Ptr(3), 0.0.into())],
            vec![ScratchBlock::VarSet(Ptr(3), 1.0.into())],
        ),
        ScratchBlock::ControlIfElse(
            "true".into(),
            vec![ScratchBlock::VarSet(Ptr(4), 1.0.into())],
            vec![ScratchBlock::VarSet(Ptr(4), 0.0.into())],
        ),
        ScratchBlock::ControlIfElse(
            "false".into(),
            vec![ScratchBlock::VarSet(Ptr(5), 0.0.into())],
            vec![ScratchBlock::VarSet(Ptr(5), 1.0.into())],
        ),
    ]
}

#[allow(unused)]
pub fn pi() -> Vec<ScratchBlock> {
    const PI: Ptr = Ptr(0);
    const D: Ptr = Ptr(1);
    const I: Ptr = Ptr(2);
    vec![
        ScratchBlock::WhenFlagClicked,
        ScratchBlock::VarSet(PI, 0.0.into()),
        ScratchBlock::VarSet(D, 1.0.into()),
        ScratchBlock::VarSet(I, 0.0.into()),
        // A test of nested repeat loops
        ScratchBlock::ControlRepeat(
            1000_000.0.into(),
            // vec![ScratchBlock::ControlRepeat(
            // 1000.0.into(),
            vec![
                // PI += ((8 * (I % 2)) - 4) / D
                ScratchBlock::VarSet(
                    PI,
                    ScratchBlock::OpAdd(
                        ScratchBlock::VarRead(PI).into(),
                        ScratchBlock::OpDiv(
                            ScratchBlock::OpSub(
                                ScratchBlock::OpMul(
                                    8.0.into(),
                                    ScratchBlock::OpMod(
                                        ScratchBlock::VarRead(I).into(),
                                        2.0.into(),
                                    )
                                    .into(),
                                )
                                .into(),
                                4.0.into(),
                            )
                            .into(),
                            ScratchBlock::VarRead(D).into(),
                        )
                        .into(),
                    )
                    .into(),
                ),
                ScratchBlock::VarChange(D, 2.0.into()),
                ScratchBlock::VarChange(I, 1.0.into()),
            ],
            // )],
        ),
    ]
}

#[allow(unused)]
pub fn nested_repeat() -> Vec<ScratchBlock> {
    vec![
        ScratchBlock::WhenFlagClicked,
        ScratchBlock::ControlRepeat(
            9.0.into(),
            vec![ScratchBlock::ControlRepeat(
                11.0.into(),
                vec![ScratchBlock::VarSet(
                    Ptr(0),
                    ScratchBlock::OpStrJoin(ScratchBlock::VarRead(Ptr(0)).into(), "H".into())
                        .into(),
                )],
            )],
        ),
    ]
}

#[allow(unused)]
pub fn repeat_until() -> Vec<ScratchBlock> {
    vec![
        ScratchBlock::WhenFlagClicked,
        ScratchBlock::VarSet(Ptr(0), 0.0.into()),
        ScratchBlock::ControlRepeatUntil(
            ScratchBlock::OpCmpGreater(ScratchBlock::VarRead(Ptr(0)).into(), 10.0.into()).into(),
            vec![
                ScratchBlock::VarSet(Ptr(1), 0.0.into()),
                ScratchBlock::ControlRepeatUntil(
                    ScratchBlock::OpCmpGreater(ScratchBlock::VarRead(Ptr(1)).into(), 20.0.into())
                        .into(),
                    vec![ScratchBlock::VarChange(Ptr(1), 1.0.into())],
                ),
                ScratchBlock::VarChange(Ptr(0), 1.0.into()),
            ],
        ),
    ]
}

#[allow(unused)]
pub fn str_ops() -> Vec<ScratchBlock> {
    vec![
        ScratchBlock::WhenFlagClicked,
        ScratchBlock::VarSet(
            Ptr(0),
            ScratchBlock::OpStrJoin("hello".into(), "world".into()).into(),
        ),
        ScratchBlock::VarSet(
            Ptr(1),
            ScratchBlock::OpStrJoin("hello".into(), 1.0.into()).into(),
        ),
        ScratchBlock::VarSet(
            Ptr(2),
            ScratchBlock::OpStrJoin(1.0.into(), "world".into()).into(),
        ),
        ScratchBlock::VarSet(
            Ptr(3),
            ScratchBlock::OpStrJoin(true.into(), 2.0.into()).into(),
        ),
        ScratchBlock::VarSet(Ptr(4), ScratchBlock::OpStrLen(true.into()).into()),
    ]
}

#[allow(unused)]
pub fn random() -> Vec<ScratchBlock> {
    vec![
        ScratchBlock::WhenFlagClicked,
        ScratchBlock::VarSet(
            Ptr(0),
            ScratchBlock::OpRandom(0.0.into(), 100.0.into()).into(),
        ),
        ScratchBlock::VarSet(
            Ptr(1),
            ScratchBlock::OpRandom(1.0.into(), 2.5.into()).into(),
        ),
        ScratchBlock::VarSet(
            Ptr(2),
            ScratchBlock::OpRandom("1".into(), "2".into()).into(),
        ),
        ScratchBlock::VarSet(
            Ptr(3),
            ScratchBlock::OpRandom("1.0".into(), "2".into()).into(),
        ),
        ScratchBlock::ControlRepeat(
            100_000.0.into(),
            vec![ScratchBlock::VarSet(
                Ptr(4),
                ScratchBlock::OpRandom(0.0.into(), 100.0.into()).into(),
            )],
        ),
    ]
}

#[allow(unused)]
pub fn math_add_test() -> Vec<ScratchBlock> {
    vec![
        ScratchBlock::WhenFlagClicked,
        ScratchBlock::VarSet(Ptr(0), ScratchBlock::OpAdd(50.0.into(), 25.0.into()).into()),
        ScratchBlock::VarSet(
            Ptr(1),
            ScratchBlock::OpAdd((-500.0).into(), 25.0.into()).into(),
        ),
        ScratchBlock::VarSet(
            Ptr(2),
            ScratchBlock::OpAdd((-500.0).into(), (-25.0).into()).into(),
        ),
        ScratchBlock::VarSet(Ptr(3), ScratchBlock::OpAdd(2.54.into(), 6.25.into()).into()),
        ScratchBlock::VarSet(
            Ptr(4),
            ScratchBlock::OpAdd(2.54.into(), (-6.25).into()).into(),
        ),
        ScratchBlock::VarSet(Ptr(5), ScratchBlock::OpAdd(true.into(), true.into()).into()),
        ScratchBlock::VarSet(
            Ptr(6),
            ScratchBlock::OpAdd((1.0 / 0.0).into(), (1.0 / 0.0).into()).into(),
        ),
        ScratchBlock::VarSet(
            Ptr(7),
            ScratchBlock::OpAdd((1.0 / 0.0).into(), (-1.0 / 0.0).into()).into(),
        ),
        ScratchBlock::VarSet(
            Ptr(8),
            ScratchBlock::OpAdd((-1.0 / 0.0).into(), (1.0 / 0.0).into()).into(),
        ),
        ScratchBlock::VarSet(
            Ptr(9),
            ScratchBlock::OpAdd((-1.0 / 0.0).into(), (-1.0 / 0.0).into()).into(),
        ),
        ScratchBlock::VarSet(
            Ptr(10),
            ScratchBlock::OpAdd(1.0.into(), f64::NAN.into()).into(),
        ),
        ScratchBlock::VarSet(
            Ptr(11),
            ScratchBlock::OpAdd(f64::NAN.into(), 1.0.into()).into(),
        ),
    ]
}

#[allow(unused)]
pub fn math_sub_test() -> Vec<ScratchBlock> {
    vec![
        ScratchBlock::WhenFlagClicked,
        ScratchBlock::VarSet(Ptr(0), ScratchBlock::OpSub(50.0.into(), 25.0.into()).into()),
        ScratchBlock::VarSet(
            Ptr(1),
            ScratchBlock::OpSub((-500.0).into(), 25.0.into()).into(),
        ),
        ScratchBlock::VarSet(
            Ptr(2),
            ScratchBlock::OpSub((-500.0).into(), (-25.0).into()).into(),
        ),
        ScratchBlock::VarSet(Ptr(3), ScratchBlock::OpSub(2.54.into(), 6.25.into()).into()),
        ScratchBlock::VarSet(
            Ptr(4),
            ScratchBlock::OpSub(2.54.into(), (-6.25).into()).into(),
        ),
        ScratchBlock::VarSet(Ptr(5), ScratchBlock::OpSub(true.into(), true.into()).into()),
        ScratchBlock::VarSet(
            Ptr(6),
            ScratchBlock::OpSub((1.0 / 0.0).into(), (1.0 / 0.0).into()).into(),
        ),
        ScratchBlock::VarSet(
            Ptr(7),
            ScratchBlock::OpSub((1.0 / 0.0).into(), (-1.0 / 0.0).into()).into(),
        ),
        ScratchBlock::VarSet(
            Ptr(8),
            ScratchBlock::OpSub((-1.0 / 0.0).into(), (1.0 / 0.0).into()).into(),
        ),
        ScratchBlock::VarSet(
            Ptr(9),
            ScratchBlock::OpSub((-1.0 / 0.0).into(), (-1.0 / 0.0).into()).into(),
        ),
        ScratchBlock::VarSet(
            Ptr(10),
            ScratchBlock::OpSub(1.0.into(), f64::NAN.into()).into(),
        ),
        ScratchBlock::VarSet(
            Ptr(11),
            ScratchBlock::OpSub(f64::NAN.into(), 1.0.into()).into(),
        ),
    ]
}

#[allow(unused)]
pub fn math_mul_test() -> Vec<ScratchBlock> {
    vec![
        ScratchBlock::WhenFlagClicked,
        ScratchBlock::VarSet(Ptr(0), ScratchBlock::OpMul(50.0.into(), 2.0.into()).into()),
        ScratchBlock::VarSet(
            Ptr(1),
            ScratchBlock::OpMul((-50.0).into(), 2.0.into()).into(),
        ),
        ScratchBlock::VarSet(
            Ptr(2),
            ScratchBlock::OpMul((-50.0).into(), (-2.0).into()).into(),
        ),
        ScratchBlock::VarSet(Ptr(3), ScratchBlock::OpMul(2.54.into(), 6.25.into()).into()),
        ScratchBlock::VarSet(
            Ptr(4),
            ScratchBlock::OpMul(2.54.into(), (-6.25).into()).into(),
        ),
        ScratchBlock::VarSet(Ptr(5), ScratchBlock::OpMul(true.into(), true.into()).into()),
        ScratchBlock::VarSet(
            Ptr(6),
            ScratchBlock::OpMul((1.0 / 0.0).into(), (1.0 / 0.0).into()).into(),
        ),
        ScratchBlock::VarSet(
            Ptr(7),
            ScratchBlock::OpMul((1.0 / 0.0).into(), 0.0.into()).into(),
        ),
        ScratchBlock::VarSet(
            Ptr(8),
            ScratchBlock::OpMul((1.0 / 0.0).into(), 2.0.into()).into(),
        ),
        ScratchBlock::VarSet(
            Ptr(9),
            ScratchBlock::OpMul((1.0 / 0.0).into(), (-2.0).into()).into(),
        ),
        ScratchBlock::VarSet(
            Ptr(10),
            ScratchBlock::OpMul((1.0 / 0.0).into(), (-1.0 / 0.0).into()).into(),
        ),
        ScratchBlock::VarSet(
            Ptr(11),
            ScratchBlock::OpMul((-1.0 / 0.0).into(), (1.0 / 0.0).into()).into(),
        ),
        ScratchBlock::VarSet(
            Ptr(12),
            ScratchBlock::OpMul((-1.0 / 0.0).into(), 0.0.into()).into(),
        ),
        ScratchBlock::VarSet(
            Ptr(13),
            ScratchBlock::OpMul((-1.0 / 0.0).into(), 2.0.into()).into(),
        ),
        ScratchBlock::VarSet(
            Ptr(14),
            ScratchBlock::OpMul((-1.0 / 0.0).into(), (-2.0).into()).into(),
        ),
        ScratchBlock::VarSet(
            Ptr(15),
            ScratchBlock::OpMul((-1.0 / 0.0).into(), (-1.0 / 0.0).into()).into(),
        ),
        ScratchBlock::VarSet(
            Ptr(16),
            ScratchBlock::OpMul(1.0.into(), f64::NAN.into()).into(),
        ),
        ScratchBlock::VarSet(
            Ptr(17),
            ScratchBlock::OpMul(f64::NAN.into(), 1.0.into()).into(),
        ),
    ]
}

#[allow(unused)]
pub fn math_div_test() -> Vec<ScratchBlock> {
    vec![
        ScratchBlock::WhenFlagClicked,
        ScratchBlock::VarSet(Ptr(0), ScratchBlock::OpDiv(50.0.into(), 2.0.into()).into()),
        ScratchBlock::VarSet(
            Ptr(1),
            ScratchBlock::OpDiv((-50.0).into(), 2.0.into()).into(),
        ),
        ScratchBlock::VarSet(
            Ptr(2),
            ScratchBlock::OpDiv((-50.0).into(), (-2.0).into()).into(),
        ),
        ScratchBlock::VarSet(Ptr(3), ScratchBlock::OpDiv(3.5.into(), 2.5.into()).into()),
        ScratchBlock::VarSet(
            Ptr(4),
            ScratchBlock::OpDiv(3.5.into(), (-2.5).into()).into(),
        ),
        ScratchBlock::VarSet(Ptr(5), ScratchBlock::OpDiv(true.into(), true.into()).into()),
        ScratchBlock::VarSet(
            Ptr(6),
            ScratchBlock::OpDiv((1.0 / 0.0).into(), (1.0 / 0.0).into()).into(),
        ),
        ScratchBlock::VarSet(
            Ptr(7),
            ScratchBlock::OpDiv((1.0 / 0.0).into(), 0.0.into()).into(),
        ),
        ScratchBlock::VarSet(
            Ptr(8),
            ScratchBlock::OpDiv((1.0 / 0.0).into(), 2.0.into()).into(),
        ),
        ScratchBlock::VarSet(
            Ptr(9),
            ScratchBlock::OpDiv((1.0 / 0.0).into(), (-2.0).into()).into(),
        ),
        ScratchBlock::VarSet(
            Ptr(10),
            ScratchBlock::OpDiv((1.0 / 0.0).into(), (-1.0 / 0.0).into()).into(),
        ),
        ScratchBlock::VarSet(
            Ptr(11),
            ScratchBlock::OpDiv((-1.0 / 0.0).into(), (1.0 / 0.0).into()).into(),
        ),
        ScratchBlock::VarSet(
            Ptr(12),
            ScratchBlock::OpDiv((-1.0 / 0.0).into(), 0.0.into()).into(),
        ),
        ScratchBlock::VarSet(
            Ptr(13),
            ScratchBlock::OpDiv((-1.0 / 0.0).into(), 2.0.into()).into(),
        ),
        ScratchBlock::VarSet(
            Ptr(14),
            ScratchBlock::OpDiv((-1.0 / 0.0).into(), (-2.0).into()).into(),
        ),
        ScratchBlock::VarSet(
            Ptr(15),
            ScratchBlock::OpDiv((-1.0 / 0.0).into(), (-1.0 / 0.0).into()).into(),
        ),
        ScratchBlock::VarSet(
            Ptr(16),
            ScratchBlock::OpDiv(1.0.into(), f64::NAN.into()).into(),
        ),
        ScratchBlock::VarSet(
            Ptr(17),
            ScratchBlock::OpDiv(f64::NAN.into(), 1.0.into()).into(),
        ),
    ]
}

#[allow(unused)]
fn run(program: &[ScratchBlock], memory: &[ScratchObject]) {
    let mut builder = settings::builder();
    builder.set("opt_level", "speed").unwrap();
    let flags = settings::Flags::new(builder);

    let isa = match isa::lookup(Triple::host()) {
        Err(err) => panic!("Error looking up target: {err}"),
        Ok(isa_builder) => isa_builder.finish(flags).unwrap(),
    };

    let sig = Signature::new(CallConv::SystemV);
    let mut func = Function::with_name_signature(UserFuncName::default(), sig);

    let mut func_ctx = FunctionBuilderContext::new();
    let mut builder = FunctionBuilder::new(&mut func, &mut func_ctx);

    let mut code_block = builder.create_block();

    builder.append_block_params_for_function_params(code_block);
    builder.switch_to_block(code_block);

    let mut variable_type_data: HashMap<Ptr, VarType> = HashMap::new();

    let mut compiler = Compiler::new(code_block);

    for block in program {
        compiler.compile_block(block, &mut builder, memory);
    }

    // builder.seal_block(compiler.code_block);
    builder.seal_all_blocks();

    let ins = builder.ins();
    ins.return_(&[]);

    builder.finalize();

    // println!("{}", func.display());

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
        code_fn();
    }
}

#[cfg(test)]
mod tests {
    use std::sync::MutexGuard;

    use crate::compiler::MEMORY;

    use super::*;

    fn run_code<'a>(code: Vec<ScratchBlock>) -> MutexGuard<'a, Box<[ScratchObject]>> {
        let mut memory = MEMORY.lock().unwrap();
        *memory = vec![ScratchObject::Number(0.0); 256].into_boxed_slice();
        run(&code, &memory);
        memory
    }

    #[test]
    pub fn b_str_ops() {
        let memory = run_code(str_ops());

        assert_eq!(memory[4].get_type(), VarType::Number);

        assert_eq!(memory[0].convert_to_string(), "helloworld");
        assert_eq!(memory[1].convert_to_string(), "hello1");
        assert_eq!(memory[2].convert_to_string(), "1world");
        assert_eq!(memory[3].convert_to_string(), "true2");
        assert_eq!(memory[4].convert_to_number(), 4.0);
    }

    #[test]
    pub fn b_pi() {
        let memory = run_code(pi());

        assert_eq!(memory[0].convert_to_number(), -3.1415916535897743);
        assert_eq!(memory[1].convert_to_number(), 2000001.0);
        assert_eq!(memory[2].convert_to_number(), 1000000.0);
    }

    #[test]
    pub fn b_nested_repeat() {
        let memory = run_code(nested_repeat());
        assert_eq!(memory[0].convert_to_string(), "0HHHHHHHHHHHHHHHHHHHHHHHHHHHHHHHHHHHHHHHHHHHHHHHHHHHHHHHHHHHHHHHHHHHHHHHHHHHHHHHHHHHHHHHHHHHHHHHHHHH")
    }

    #[test]
    pub fn b_repeat_until() {
        let memory = run_code(repeat_until());
        assert_eq!(memory[0].convert_to_number(), 11.0);
        assert_eq!(memory[1].convert_to_number(), 21.0);
    }

    #[test]
    pub fn b_if_else_test() {
        let memory = run_code(if_else_test());
        assert_eq!(memory[0].convert_to_number(), 1.0);
        assert_eq!(memory[1].convert_to_number(), 1.0);
        assert_eq!(memory[2].convert_to_number(), 1.0);
        assert_eq!(memory[3].convert_to_number(), 1.0);
        assert_eq!(memory[4].convert_to_number(), 1.0);
        assert_eq!(memory[5].convert_to_number(), 1.0);
    }

    #[test]
    pub fn b_if_test() {
        let memory = run_code(if_test());
        assert_eq!(memory[0].convert_to_number(), 1.0);
        assert_eq!(memory[1].convert_to_number(), 0.0);
        assert_eq!(memory[2].convert_to_number(), 1.0);
        assert_eq!(memory[3].convert_to_number(), 0.0);
        assert_eq!(memory[4].convert_to_number(), 1.0);
        assert_eq!(memory[5].convert_to_number(), 0.0);
        assert_eq!(memory[6].convert_to_number(), 1.0);
        assert_eq!(memory[7].convert_to_number(), 0.0);
        assert_eq!(memory[8].convert_to_number(), 1.0);
        assert_eq!(memory[9].convert_to_number(), 0.0);
        assert_eq!(memory[10].convert_to_number(), 0.0);
        assert_eq!(memory[11].convert_to_number(), 0.0);
    }

    #[test]
    pub fn b_repeated_sum() {
        let memory = run_code(repeated_sum());
        assert_eq!(memory[7].convert_to_number(), 200000.0);
    }

    #[test]
    pub fn b_repeated_join_string() {
        let memory = run_code(repeated_join_string());
        assert_eq!(
            memory[7].convert_to_string(),
            "hello world, world, world, world, world, world, world, world, world, world, world, world, world, world, world, world, world, world, world, world, world, world, world, world, world, world, world, world, world, world, world, world, world, world, world, world, world, world, world, world, world, world, world, world, world, world, world, world, world, world, world, world, world, world, world, world, world, world, world, world, world, world, world, world, world, world, world, world, world, world, world, world, world, world, world, world, world, world, world, world, world, world, world, world, world, world, world, world, world, world, world, world, world, world, world, world, world, world, world, world, "
        );
    }

    #[test]
    pub fn b_random() {
        let memory = run_code(random());

        assert!(memory[0].convert_to_number() >= 0.0);
        assert!(memory[0].convert_to_number() <= 100.0);
        assert_eq!(memory[0].convert_to_number().fract(), 0.0);

        assert!(memory[1].convert_to_number() >= 1.0);
        assert!(memory[1].convert_to_number() <= 2.5);
        assert_ne!(memory[1].convert_to_number().fract(), 0.0);

        assert!(memory[2].convert_to_number() >= 1.0);
        assert!(memory[2].convert_to_number() <= 2.0);
        assert_eq!(memory[2].convert_to_number().fract(), 0.0);

        assert!(memory[3].convert_to_number() >= 1.0);
        assert!(memory[3].convert_to_number() <= 2.0);
        assert_ne!(memory[3].convert_to_number().fract(), 0.0);

        assert!(memory[4].convert_to_number() >= 0.0);
        assert!(memory[4].convert_to_number() <= 100.0);
        assert_eq!(memory[4].convert_to_number().fract(), 0.0);
    }

    #[test]
    pub fn b_math_add_test() {
        let memory = run_code(math_add_test());
        assert_eq!(memory[0].convert_to_number(), 75.0);
        assert_eq!(memory[1].convert_to_number(), -475.0);
        assert_eq!(memory[2].convert_to_number(), -525.0);
        assert_eq!(memory[3].convert_to_number(), 8.79);
        assert_eq!(memory[4].convert_to_number(), -3.71);
        assert_eq!(memory[5].convert_to_number(), 2.0);

        assert!(memory[6].convert_to_number().is_infinite());
        assert!(memory[7].convert_to_number().is_nan());
        assert!(memory[8].convert_to_number().is_nan());
        assert!(memory[9].convert_to_number().is_infinite());
        assert!(memory[9].convert_to_number().is_sign_negative());

        assert_eq!(memory[10].convert_to_number(), 1.0);
        assert_eq!(memory[11].convert_to_number(), 1.0);
    }

    #[test]
    pub fn b_math_sub_test() {
        let memory = run_code(math_sub_test());
        assert_eq!(memory[0].convert_to_number(), 25.0);
        assert_eq!(memory[1].convert_to_number(), -525.0);
        assert_eq!(memory[2].convert_to_number(), -475.0);
        assert_eq!(memory[3].convert_to_number(), -3.71);
        assert_eq!(memory[4].convert_to_number(), 8.79);
        assert_eq!(memory[5].convert_to_number(), 0.0);

        assert!(memory[6].convert_to_number().is_nan());
        assert!(memory[7].convert_to_number().is_infinite());
        assert!(memory[7].convert_to_number().is_sign_positive());
        assert!(memory[8].convert_to_number().is_infinite());
        assert!(memory[8].convert_to_number().is_sign_negative());
        assert!(memory[9].convert_to_number().is_nan());

        assert_eq!(memory[10].convert_to_number(), 1.0);
        assert_eq!(memory[11].convert_to_number(), -1.0);
    }

    #[test]
    pub fn b_math_mul_test() {
        let memory = run_code(math_mul_test());
        assert_eq!(memory[0].convert_to_number(), 100.0);
        assert_eq!(memory[1].convert_to_number(), -100.0);
        assert_eq!(memory[2].convert_to_number(), 100.0);
        assert_eq!(memory[3].convert_to_number(), 15.875);
        assert_eq!(memory[4].convert_to_number(), -15.875);
        assert_eq!(memory[5].convert_to_number(), 1.0);

        assert!(memory[6].convert_to_number().is_infinite());
        assert!(memory[6].convert_to_number().is_sign_positive());

        assert!(memory[7].convert_to_number().is_nan());

        assert!(memory[8].convert_to_number().is_infinite());
        assert!(memory[8].convert_to_number().is_sign_positive());

        assert!(memory[9].convert_to_number().is_infinite());
        assert!(memory[9].convert_to_number().is_sign_negative());

        assert!(memory[10].convert_to_number().is_infinite());
        assert!(memory[10].convert_to_number().is_sign_negative());

        assert!(memory[11].convert_to_number().is_infinite());
        assert!(memory[11].convert_to_number().is_sign_negative());

        assert!(memory[12].convert_to_number().is_nan());

        assert!(memory[13].convert_to_number().is_infinite());
        assert!(memory[13].convert_to_number().is_sign_negative());

        assert!(memory[14].convert_to_number().is_infinite());
        assert!(memory[14].convert_to_number().is_sign_positive());

        assert!(memory[15].convert_to_number().is_infinite());
        assert!(memory[15].convert_to_number().is_sign_positive());

        assert_eq!(memory[16].convert_to_number(), 0.0);
        assert_eq!(memory[17].convert_to_number(), 0.0);
    }

    #[test]
    pub fn b_math_div_test() {
        let memory = run_code(math_div_test());
        assert_eq!(memory[0].convert_to_number(), 25.0);
        assert_eq!(memory[1].convert_to_number(), -25.0);
        assert_eq!(memory[2].convert_to_number(), 25.0);
        assert_eq!(memory[3].convert_to_number(), 1.4);
        assert_eq!(memory[4].convert_to_number(), -1.4);
        assert_eq!(memory[5].convert_to_number(), 1.0);

        assert!(memory[6].convert_to_number().is_nan());

        assert!(memory[7].convert_to_number().is_infinite());
        assert!(memory[7].convert_to_number().is_sign_positive());

        assert!(memory[8].convert_to_number().is_infinite());
        assert!(memory[8].convert_to_number().is_sign_positive());

        assert!(memory[9].convert_to_number().is_infinite());
        assert!(memory[9].convert_to_number().is_sign_negative());

        assert!(memory[10].convert_to_number().is_nan());
        assert!(memory[11].convert_to_number().is_nan());

        assert!(memory[12].convert_to_number().is_infinite());
        assert!(memory[12].convert_to_number().is_sign_negative());

        assert!(memory[13].convert_to_number().is_infinite());
        assert!(memory[13].convert_to_number().is_sign_negative());

        assert!(memory[14].convert_to_number().is_infinite());
        assert!(memory[14].convert_to_number().is_sign_positive());

        assert!(memory[15].convert_to_number().is_nan());

        assert!(memory[16].convert_to_number().is_infinite());
        assert!(memory[16].convert_to_number().is_sign_positive());

        assert_eq!(memory[17].convert_to_number(), 0.0);
    }
}
