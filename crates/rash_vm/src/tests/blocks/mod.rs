mod utils;
// pub use utils::run_code;

use std::cmp::Ordering;

use utils::run_code;

use crate::{
    Input, Ptr, ScratchBlock, ScratchObject,
    builder::{c_if, c_repeat, set},
    compiler::VarType,
};

mod control;
mod math;

fn set_vars(input: Vec<Input>) -> Vec<ScratchBlock> {
    input
        .into_iter()
        .enumerate()
        .map(|(i, input)| set(Ptr(i), input))
        .collect()
}

#[test]
pub fn str_ops() {
    let memory = run_code(set_vars(vec![
        ScratchBlock::OpStrJoin("hello".into(), "world".into()).into(),
        ScratchBlock::OpStrJoin("hello".into(), 1.0.into()).into(),
        ScratchBlock::OpStrJoin(1.0.into(), "world".into()).into(),
        ScratchBlock::OpStrJoin(true.into(), 2.0.into()).into(),
        ScratchBlock::OpStrLen(true.into()).into(),
        ScratchBlock::OpStrLen("💀".into()).into(),
        ScratchBlock::OpStrLetterOf(2.0.into(), "hello".into()).into(),
        ScratchBlock::OpStrLetterOf(0.0.into(), "hello".into()).into(),
        ScratchBlock::OpStrLetterOf(1.0.into(), "💀".into()).into(),
        ScratchBlock::OpStrContains("Hello World".into(), "World".into()).into(),
        ScratchBlock::OpStrContains("Hello World".into(), "world".into()).into(),
        ScratchBlock::OpStrContains("Hello World".into(), "Hi".into()).into(),
        // real bug btw
        ScratchBlock::OpStrJoin("".into(), true.into()).into(),
    ]));

    assert_eq!(memory[4].get_type(), VarType::Number);

    assert_eq!(memory[0].convert_to_string(), "helloworld");
    assert_eq!(memory[1].convert_to_string(), "hello1");
    assert_eq!(memory[2].convert_to_string(), "1world");
    assert_eq!(memory[3].convert_to_string(), "true2");
    assert_eq!(memory[4].convert_to_number(), 4.0);

    // Skull emoji takes 2 chars in Scratch.
    assert_eq!(memory[5].convert_to_number(), 2.0);

    assert_eq!(memory[6].convert_to_string(), "e");
    assert_eq!(memory[7].convert_to_string(), "");

    // memory[8] isn't valid unicode.
    // There is no way to test this.
    assert_eq!(memory[8].convert_to_string().chars().count(), 1);

    assert!(memory[9].convert_to_bool());
    assert!(memory[10].convert_to_bool());
    assert!(!memory[11].convert_to_bool());
    assert_eq!(memory[12].convert_to_string(), "true");
}

#[test]
pub fn pi() {
    let memory = run_code(crate::builder::program_pi());

    assert_eq!(memory[0].convert_to_number(), -3.1415916535897743);
    assert_eq!(memory[1].convert_to_number(), 2000001.0);
    assert_eq!(memory[2].convert_to_number(), 1000000.0);
}

#[test]
pub fn random() {
    let memory = run_code(vec![
        set(Ptr(0), ScratchBlock::OpRandom(0.0.into(), 100.0.into())),
        set(Ptr(1), ScratchBlock::OpRandom(1.0.into(), 2.5.into())),
        set(Ptr(2), ScratchBlock::OpRandom("1".into(), "2".into())),
        set(Ptr(3), ScratchBlock::OpRandom("1.0".into(), "2".into())),
        c_repeat(
            100_000.0,
            vec![set(
                Ptr(4),
                ScratchBlock::OpRandom(0.0.into(), 100.0.into()),
            )],
        ),
    ]);

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
pub fn negative_zero_is_truthy() {
    // Real bug btw
    let memory = run_code(vec![
        set(Ptr(0), -0.0),
        c_if(Ptr(0), vec![set(Ptr(1), 1.0)]),
        // Also verify the literal path
        c_if(-0.0, vec![set(Ptr(2), 1.0)]),
    ]);
    assert_eq!(memory[1].convert_to_number(), 1.0);
    assert_eq!(memory[2].convert_to_number(), 1.0);
}

#[test]
pub fn bool_ops() {
    fn check_and(memory: &[ScratchObject], offset: usize) {
        assert!(memory[offset + 0].convert_to_bool());
        assert!(!memory[offset + 1].convert_to_bool());
        assert!(!memory[offset + 2].convert_to_bool());
        assert!(!memory[offset + 3].convert_to_bool());
    }

    fn check_or(memory: &[ScratchObject], offset: usize) {
        assert!(memory[offset + 0].convert_to_bool());
        assert!(memory[offset + 1].convert_to_bool());
        assert!(memory[offset + 2].convert_to_bool());
        assert!(!memory[offset + 3].convert_to_bool());
    }

    // Testing multiple data types as this caused a real bug earlier
    let memory = run_code(set_vars(vec![
        // Bools
        ScratchBlock::OpBAnd(true.into(), true.into()).into(),
        ScratchBlock::OpBAnd(true.into(), false.into()).into(),
        ScratchBlock::OpBAnd(false.into(), true.into()).into(),
        ScratchBlock::OpBAnd(false.into(), false.into()).into(),
        ScratchBlock::OpBOr(true.into(), true.into()).into(),
        ScratchBlock::OpBOr(true.into(), false.into()).into(),
        ScratchBlock::OpBOr(false.into(), true.into()).into(),
        ScratchBlock::OpBOr(false.into(), false.into()).into(),
        ScratchBlock::OpBNot(true.into()).into(),
        ScratchBlock::OpBNot(false.into()).into(),
        // Numbers
        ScratchBlock::OpBNot(1.0.into()).into(),
        ScratchBlock::OpBNot(0.0.into()).into(),
        ScratchBlock::OpBAnd(1.0.into(), 1.0.into()).into(),
        ScratchBlock::OpBAnd(1.0.into(), 0.0.into()).into(),
        ScratchBlock::OpBAnd(0.0.into(), 1.0.into()).into(),
        ScratchBlock::OpBAnd(0.0.into(), 0.0.into()).into(),
        ScratchBlock::OpBOr(1.0.into(), 1.0.into()).into(),
        ScratchBlock::OpBOr(1.0.into(), 0.0.into()).into(),
        ScratchBlock::OpBOr(0.0.into(), 1.0.into()).into(),
        ScratchBlock::OpBOr(0.0.into(), 0.0.into()).into(),
        // Strings
        ScratchBlock::OpBAnd("true".into(), "true".into()).into(),
        ScratchBlock::OpBAnd("true".into(), "false".into()).into(),
        ScratchBlock::OpBAnd("false".into(), "true".into()).into(),
        ScratchBlock::OpBAnd("false".into(), "false".into()).into(),
        ScratchBlock::OpBOr("true".into(), "true".into()).into(),
        ScratchBlock::OpBOr("true".into(), "false".into()).into(),
        ScratchBlock::OpBOr("false".into(), "true".into()).into(),
        ScratchBlock::OpBOr("false".into(), "false".into()).into(),
        ScratchBlock::OpBNot("true".into()).into(),
        ScratchBlock::OpBNot("false".into()).into(),
    ]));

    check_and(&memory, 0);
    check_or(&memory, 4);

    assert!(!memory[8].convert_to_bool());
    assert!(memory[9].convert_to_bool());
    assert!(!memory[10].convert_to_bool());
    assert!(memory[11].convert_to_bool());

    check_and(&memory, 12);
    check_or(&memory, 16);
    check_and(&memory, 20);
    check_or(&memory, 24);

    assert!(!memory[28].convert_to_bool());
    assert!(memory[29].convert_to_bool());
}

#[test]
fn bool_return() {
    let memory = run_code(vec![set(
        Ptr(0),
        ScratchBlock::OpBAnd(
            ScratchBlock::OpCmp(3.0.into(), 2.0.into(), Ordering::Greater).into(),
            false.into(),
        ),
    )]);
    assert_eq!(memory[0].convert_to_number(), 0.0);
}

#[test]
fn comparison() {
    let memory = run_code(set_vars(vec![
        ScratchBlock::OpCmp(3.0.into(), 2.0.into(), Ordering::Greater).into(),
        ScratchBlock::OpCmp(2.0.into(), 3.0.into(), Ordering::Greater).into(),
        ScratchBlock::OpCmp(3.0.into(), 3.0.into(), Ordering::Greater).into(),
        ScratchBlock::OpCmp(3.0.into(), 2.0.into(), Ordering::Less).into(),
        ScratchBlock::OpCmp(2.0.into(), 3.0.into(), Ordering::Less).into(),
        ScratchBlock::OpCmp(3.0.into(), 3.0.into(), Ordering::Less).into(),
    ]));
    assert_eq!(memory[0].convert_to_number(), 1.0);
    assert_eq!(memory[1].convert_to_number(), 0.0);
    assert_eq!(memory[2].convert_to_number(), 0.0);
    assert_eq!(memory[3].convert_to_number(), 0.0);
    assert_eq!(memory[4].convert_to_number(), 1.0);
    assert_eq!(memory[5].convert_to_number(), 0.0);
}
