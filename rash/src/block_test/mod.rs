use crate::{compiler::ScratchBlock, input_primitives::Ptr};

mod utils;

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
            1_000_000.0.into(),
            // vec![ScratchBlock::ControlRepeat(
            // 1000.0.into(),
            vec![
                // PI += ((8 * (I % 2)) - 4) / D
                ScratchBlock::VarChange(
                    PI,
                    ScratchBlock::OpDiv(
                        ScratchBlock::OpSub(
                            ScratchBlock::OpMul(
                                8.0.into(),
                                ScratchBlock::OpMod(ScratchBlock::VarRead(I).into(), 2.0.into())
                                    .into(),
                            )
                            .into(),
                            4.0.into(),
                        )
                        .into(),
                        ScratchBlock::VarRead(D).into(),
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
        ScratchBlock::VarSet(Ptr(5), ScratchBlock::OpStrLen("💀".into()).into()),
        ScratchBlock::VarSet(
            Ptr(6),
            ScratchBlock::OpStrLetterOf(2.0.into(), "hello".into()).into(),
        ),
        ScratchBlock::VarSet(
            Ptr(7),
            ScratchBlock::OpStrLetterOf(0.0.into(), "hello".into()).into(),
        ),
        ScratchBlock::VarSet(
            Ptr(8),
            ScratchBlock::OpStrLetterOf(1.0.into(), "💀".into()).into(),
        ),
        ScratchBlock::VarSet(
            Ptr(9),
            ScratchBlock::OpStrContains("Hello World".into(), "World".into()).into(),
        ),
        ScratchBlock::VarSet(
            Ptr(10),
            ScratchBlock::OpStrContains("Hello World".into(), "world".into()).into(),
        ),
        ScratchBlock::VarSet(
            Ptr(11),
            ScratchBlock::OpStrContains("Hello World".into(), "Hi".into()).into(),
        ),
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
pub fn bool_ops() -> Vec<ScratchBlock> {
    vec![
        ScratchBlock::WhenFlagClicked,
        ScratchBlock::VarSet(
            Ptr(0),
            ScratchBlock::OpBAnd(true.into(), true.into()).into(),
        ),
        ScratchBlock::VarSet(
            Ptr(1),
            ScratchBlock::OpBAnd(true.into(), false.into()).into(),
        ),
        ScratchBlock::VarSet(
            Ptr(2),
            ScratchBlock::OpBAnd(false.into(), true.into()).into(),
        ),
        ScratchBlock::VarSet(
            Ptr(3),
            ScratchBlock::OpBAnd(false.into(), false.into()).into(),
        ),
        ScratchBlock::VarSet(Ptr(4), ScratchBlock::OpBOr(true.into(), true.into()).into()),
        ScratchBlock::VarSet(
            Ptr(5),
            ScratchBlock::OpBOr(true.into(), false.into()).into(),
        ),
        ScratchBlock::VarSet(
            Ptr(6),
            ScratchBlock::OpBOr(false.into(), true.into()).into(),
        ),
        ScratchBlock::VarSet(
            Ptr(7),
            ScratchBlock::OpBOr(false.into(), false.into()).into(),
        ),
        ScratchBlock::VarSet(Ptr(8), ScratchBlock::OpBNot(true.into()).into()),
        ScratchBlock::VarSet(Ptr(9), ScratchBlock::OpBNot(false.into()).into()),
        ScratchBlock::VarSet(Ptr(10), ScratchBlock::OpBNot(1.0.into()).into()),
        ScratchBlock::VarSet(Ptr(11), ScratchBlock::OpBNot(0.0.into()).into()),
    ]
}

#[allow(unused)]
pub fn math_misc() -> Vec<ScratchBlock> {
    vec![
        ScratchBlock::WhenFlagClicked,
        ScratchBlock::VarSet(Ptr(0), ScratchBlock::OpRound(2.3.into()).into()),
        ScratchBlock::VarSet(Ptr(1), ScratchBlock::OpRound(2.5.into()).into()),
        ScratchBlock::VarSet(Ptr(2), ScratchBlock::OpRound(2.7.into()).into()),
        ScratchBlock::VarSet(Ptr(3), ScratchBlock::OpRound(3.0.into()).into()),
        ScratchBlock::VarSet(Ptr(4), ScratchBlock::OpRound((-2.3).into()).into()),
        ScratchBlock::VarSet(Ptr(5), ScratchBlock::OpRound((-2.5).into()).into()),
        ScratchBlock::VarSet(Ptr(6), ScratchBlock::OpRound((-2.7).into()).into()),
        ScratchBlock::VarSet(Ptr(7), ScratchBlock::OpRound((-3.0).into()).into()),
    ]
}

#[allow(unused)]
pub fn math_modulo() -> Vec<ScratchBlock> {
    vec![
        ScratchBlock::WhenFlagClicked,
        ScratchBlock::VarSet(Ptr(0), ScratchBlock::OpMod(5.5.into(), 3.0.into()).into()), // 5.5 % 3.0
        ScratchBlock::VarSet(
            Ptr(1),
            ScratchBlock::OpMod((-5.5).into(), 3.0.into()).into(),
        ), // -5.5 % 3.0
        ScratchBlock::VarSet(
            Ptr(2),
            ScratchBlock::OpMod(5.5.into(), (-3.0).into()).into(),
        ), // 5.5 % -3.0
        ScratchBlock::VarSet(
            Ptr(3),
            ScratchBlock::OpMod((-5.5).into(), (-3.0).into()).into(),
        ), // -5.5 % -3.0
        ScratchBlock::VarSet(Ptr(4), ScratchBlock::OpMod(10.0.into(), 3.0.into()).into()), // 10.0 % 3.0
        ScratchBlock::VarSet(
            Ptr(5),
            ScratchBlock::OpMod((-10.0).into(), 3.0.into()).into(),
        ), // -10.0 % 3.0
        ScratchBlock::VarSet(
            Ptr(6),
            ScratchBlock::OpMod(10.0.into(), (-3.0).into()).into(),
        ), // 10.0 % -3.0
        ScratchBlock::VarSet(
            Ptr(7),
            ScratchBlock::OpMod((-10.0).into(), (-3.0).into()).into(),
        ), // -10.0 % -3.0
        ScratchBlock::VarSet(Ptr(8), ScratchBlock::OpMod(0.0.into(), 1.0.into()).into()), // 0.0 % 1.0
        ScratchBlock::VarSet(
            Ptr(9),
            ScratchBlock::OpMod((-1.0).into(), 1.0.into()).into(),
        ), // -1.0 % 1.0
        ScratchBlock::VarSet(
            Ptr(10),
            ScratchBlock::OpMod(1.0.into(), (-1.0).into()).into(),
        ), // 1.0 % -1.0
        ScratchBlock::VarSet(Ptr(11), ScratchBlock::OpMod(1.0.into(), 2.5.into()).into()), // 1.0 % 2.5
        ScratchBlock::VarSet(
            Ptr(12),
            ScratchBlock::OpMod((-1.0).into(), 2.5.into()).into(),
        ), // -1.0 % 2.5
        ScratchBlock::VarSet(Ptr(13), ScratchBlock::OpMod(1e10.into(), 3.0.into()).into()), // Large numbers
        ScratchBlock::VarSet(
            Ptr(14),
            ScratchBlock::OpMod((-1e10).into(), 3.0.into()).into(),
        ),
        ScratchBlock::VarSet(
            Ptr(15),
            ScratchBlock::OpMod(0.0001.into(), 0.003.into()).into(),
        ), // Small remainders
        ScratchBlock::VarSet(
            Ptr(16),
            ScratchBlock::OpMod((-0.0001).into(), 0.003.into()).into(),
        ),
    ]
}

#[allow(unused)]
pub fn math_floor() -> Vec<ScratchBlock> {
    vec![
        ScratchBlock::WhenFlagClicked,
        ScratchBlock::VarSet(Ptr(0), ScratchBlock::OpMFloor(5.5.into()).into()),
        ScratchBlock::VarSet(Ptr(1), ScratchBlock::OpMFloor((-3.2).into()).into()),
        ScratchBlock::VarSet(Ptr(2), ScratchBlock::OpMFloor(0.0.into()).into()),
        ScratchBlock::VarSet(Ptr(3), ScratchBlock::OpMFloor((-0.8).into()).into()),
        ScratchBlock::VarSet(Ptr(4), ScratchBlock::OpMFloor(2.999.into()).into()),
        ScratchBlock::VarSet(Ptr(5), ScratchBlock::OpMFloor((-1.1).into()).into()),
        ScratchBlock::VarSet(Ptr(6), ScratchBlock::OpMFloor(10.0.into()).into()),
        ScratchBlock::VarSet(Ptr(7), ScratchBlock::OpMFloor((-10.999).into()).into()),
        ScratchBlock::VarSet(Ptr(8), ScratchBlock::OpMFloor(123456.789.into()).into()),
        ScratchBlock::VarSet(Ptr(9), ScratchBlock::OpMFloor((-123456.789).into()).into()),
        ScratchBlock::VarSet(Ptr(10), ScratchBlock::OpMFloor(1e-9.into()).into()),
        ScratchBlock::VarSet(Ptr(11), ScratchBlock::OpMFloor((-1e-9).into()).into()),
        ScratchBlock::VarSet(Ptr(12), ScratchBlock::OpMFloor(1e10.into()).into()),
        ScratchBlock::VarSet(Ptr(13), ScratchBlock::OpMFloor((-1e10).into()).into()),
    ]
}

#[cfg(test)]
mod tests {
    use utils::run_code;

    use crate::compiler::VarType;

    use super::*;

    #[test]
    pub fn b_str_ops() {
        let memory = run_code(str_ops());

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

    #[test]
    pub fn b_bool_ops() {
        let memory = run_code(bool_ops());
        assert_eq!(memory[0].convert_to_number(), 1.0);
        assert_eq!(memory[1].convert_to_number(), 0.0);
        assert_eq!(memory[2].convert_to_number(), 0.0);
        assert_eq!(memory[3].convert_to_number(), 0.0);
        assert_eq!(memory[4].convert_to_number(), 1.0);
        assert_eq!(memory[5].convert_to_number(), 1.0);
        assert_eq!(memory[6].convert_to_number(), 1.0);
        assert_eq!(memory[7].convert_to_number(), 0.0);
        assert_eq!(memory[8].convert_to_number(), 0.0);
        assert_eq!(memory[9].convert_to_number(), 1.0);
        assert_eq!(memory[10].convert_to_number(), 0.0);
        assert_eq!(memory[11].convert_to_number(), 1.0);
    }

    #[test]
    pub fn b_math_modulo() {
        let memory = run_code(math_modulo());

        assert_eq!(memory[0].convert_to_number(), 2.5);
        assert!((memory[1].convert_to_number() - 0.5) <= f64::EPSILON);
        assert!((memory[2].convert_to_number() + 0.5).abs() <= f64::EPSILON);
        assert_eq!(memory[3].convert_to_number(), -2.5);

        assert!((memory[4].convert_to_number() - 1.0) <= 2.0 * f64::EPSILON);
        assert!((memory[5].convert_to_number() - 2.0).abs() <= 2.0 * f64::EPSILON);
        assert!((memory[6].convert_to_number() + 2.0).abs() <= 2.0 * f64::EPSILON);
        assert!((memory[7].convert_to_number() + 1.0).abs() <= 2.0 * f64::EPSILON);

        assert_eq!(memory[8].convert_to_number(), 0.0);
        assert_eq!(memory[9].convert_to_number(), 0.0);
        assert_eq!(memory[10].convert_to_number(), 0.0);

        assert_eq!(memory[11].convert_to_number(), 1.0);
        assert_eq!(memory[12].convert_to_number(), 1.5);

        assert!(
            (memory[13].convert_to_number() - 1.0).abs() <= (i32::MAX as f64 + 1.0) * f64::EPSILON
        );
        assert!(
            (memory[14].convert_to_number() - 2.0).abs() <= (i32::MAX as f64 + 1.0) * f64::EPSILON
        );

        assert_eq!(memory[15].convert_to_number(), 0.0001);
        assert!(memory[16].convert_to_number() - 0.0029 <= f64::EPSILON);
    }

    #[test]
    pub fn b_math_floor() {
        let memory = run_code(math_floor());
        assert_eq!(memory[0].convert_to_number(), 5.0);
        assert_eq!(memory[1].convert_to_number(), -4.0);
        assert_eq!(memory[2].convert_to_number(), 0.0);
        assert_eq!(memory[3].convert_to_number(), -1.0);
        assert_eq!(memory[4].convert_to_number(), 2.0);
        assert_eq!(memory[5].convert_to_number(), -2.0);
        assert_eq!(memory[6].convert_to_number(), 10.0);
        assert_eq!(memory[7].convert_to_number(), -11.0);
        assert_eq!(memory[8].convert_to_number(), 123456.0);
        assert_eq!(memory[9].convert_to_number(), -123457.0);
        assert_eq!(memory[10].convert_to_number(), 0.0);
        assert_eq!(memory[11].convert_to_number(), -1.0);
        assert_eq!(memory[12].convert_to_number(), 1e10);
        assert_eq!(memory[13].convert_to_number(), -1e10);
    }

    #[test]
    pub fn b_math_misc() {
        let memory = run_code(math_misc());
        assert_eq!(memory[0].convert_to_number(), 2.0);
        assert_eq!(memory[1].convert_to_number(), 3.0);
        assert_eq!(memory[2].convert_to_number(), 3.0);
        assert_eq!(memory[3].convert_to_number(), 3.0);
        assert_eq!(memory[4].convert_to_number(), -2.0);
        assert_eq!(memory[5].convert_to_number(), -2.0);
        assert_eq!(memory[6].convert_to_number(), -3.0);
        assert_eq!(memory[7].convert_to_number(), -3.0);

        /*
        ScratchBlock::VarSet(Ptr(0), ScratchBlock::OpRound(2.3.into()).into()),
        ScratchBlock::VarSet(Ptr(1), ScratchBlock::OpRound(2.5.into()).into()),
        ScratchBlock::VarSet(Ptr(2), ScratchBlock::OpRound(2.7.into()).into()),
        ScratchBlock::VarSet(Ptr(3), ScratchBlock::OpRound(3.0.into()).into()),
        ScratchBlock::VarSet(Ptr(4), ScratchBlock::OpRound((-2.3).into()).into()),
        ScratchBlock::VarSet(Ptr(5), ScratchBlock::OpRound((-2.5).into()).into()),
        ScratchBlock::VarSet(Ptr(6), ScratchBlock::OpRound((-2.7).into()).into()),
        ScratchBlock::VarSet(Ptr(7), ScratchBlock::OpRound((-3.0).into()).into()),
        */
    }
}
