use std::collections::HashMap;

use crate::{
    compiler::{compile_block, ScratchBlock, VarType},
    data_types::ScratchObject,
    input_primitives::{Input, Ptr},
};

use codegen::{
    control::ControlPlane,
    ir::{Function, UserFuncName},
};
use cranelift::prelude::*;
use isa::CallConv;
use target_lexicon::Triple;

#[allow(unused)]
pub fn arithmetic() -> Vec<ScratchBlock> {
    vec![
        ScratchBlock::WhenFlagClicked,
        ScratchBlock::VarSet(Ptr(0), Input::new_num(2.0)),
        ScratchBlock::VarSet(Ptr(1), Input::Obj(ScratchObject::Bool(true))),
        ScratchBlock::VarSet(Ptr(2), Input::Obj(ScratchObject::Bool(false))),
        ScratchBlock::VarSet(
            Ptr(3),
            Input::Obj(ScratchObject::String("192.0".to_owned())),
        ),
        ScratchBlock::VarSet(
            Ptr(4),
            Input::Block(Box::new(ScratchBlock::OpAdd(
                Input::new_num(2.0),
                Input::Block(Box::new(ScratchBlock::OpMul(
                    Input::Obj(ScratchObject::String("3.0".to_owned())),
                    Input::new_num(4.0),
                ))),
            ))),
        ),
        ScratchBlock::VarSet(
            Ptr(5),
            Input::Block(Box::new(ScratchBlock::OpSub(
                Input::new_num(2.0),
                Input::Block(Box::new(ScratchBlock::OpDiv(
                    Input::new_num(3.0),
                    Input::new_num(4.0),
                ))),
            ))),
        ),
        ScratchBlock::VarSet(
            Ptr(6),
            Input::Block(Box::new(ScratchBlock::OpAdd(
                Input::Block(Box::new(ScratchBlock::OpAdd(
                    Input::Obj(ScratchObject::Bool(true)),
                    Input::Obj(ScratchObject::Bool(true)),
                ))),
                Input::Block(Box::new(ScratchBlock::VarRead(Ptr(3)))),
            ))),
        ),
        ScratchBlock::VarSet(
            Ptr(7),
            Input::Block(Box::new(ScratchBlock::OpAdd(
                Input::Block(Box::new(ScratchBlock::VarRead(Ptr(7)))),
                Input::Obj(ScratchObject::Bool(true)),
            ))),
        ),
    ]
}

#[allow(unused)]
pub fn repeated_sum() -> Vec<ScratchBlock> {
    vec![
        ScratchBlock::WhenFlagClicked,
        ScratchBlock::VarSet(
            Ptr(7),
            Input::Block(Box::new(ScratchBlock::OpAdd(
                Input::Block(Box::new(ScratchBlock::VarRead(Ptr(7)))),
                Input::Obj(ScratchObject::Bool(false)),
            ))),
        ),
        ScratchBlock::ControlRepeat(
            Input::new_num(100000.0),
            vec![
                ScratchBlock::VarSet(
                    Ptr(7),
                    Input::Block(Box::new(ScratchBlock::OpAdd(
                        Input::Block(Box::new(ScratchBlock::VarRead(Ptr(7)))),
                        Input::Obj(ScratchObject::Bool(true)),
                    ))),
                ),
                ScratchBlock::VarSet(
                    Ptr(7),
                    Input::Block(Box::new(ScratchBlock::OpAdd(
                        Input::Block(Box::new(ScratchBlock::VarRead(Ptr(7)))),
                        Input::Obj(ScratchObject::Bool(true)),
                    ))),
                ),
            ],
        ),
    ]
}

#[allow(unused)]
pub fn repeated_join_string() -> Vec<ScratchBlock> {
    vec![
        ScratchBlock::WhenFlagClicked,
        ScratchBlock::VarSet(
            Ptr(7),
            Input::Obj(ScratchObject::String("hello ".to_owned())),
        ),
        ScratchBlock::ControlRepeat(
            Input::new_num(100.0),
            vec![
                ScratchBlock::VarSet(
                    Ptr(7),
                    Input::Block(Box::new(ScratchBlock::OpStrJoin(
                        Input::new_block(ScratchBlock::VarRead(Ptr(7))),
                        Input::Obj(ScratchObject::String("world".to_owned())),
                    ))),
                ),
                ScratchBlock::VarSet(
                    Ptr(7),
                    Input::Block(Box::new(ScratchBlock::OpStrJoin(
                        Input::new_block(ScratchBlock::VarRead(Ptr(7))),
                        Input::Obj(ScratchObject::String(", ".to_owned())),
                    ))),
                ),
            ],
        ),
    ]
}

// Pass: 1,0,1,0 pattern until *9
#[allow(unused)]
pub fn if_test() -> Vec<ScratchBlock> {
    vec![
        ScratchBlock::WhenFlagClicked,
        ScratchBlock::ControlIf(
            Input::new_num(1.0),
            vec![ScratchBlock::VarSet(Ptr(0), Input::new_num(1.0))],
        ),
        ScratchBlock::ControlIf(
            Input::new_num(0.0),
            vec![ScratchBlock::VarSet(Ptr(1), Input::new_num(1.0))],
        ),
        ScratchBlock::ControlIf(
            Input::Obj(ScratchObject::Bool(true)),
            vec![ScratchBlock::VarSet(Ptr(2), Input::new_num(1.0))],
        ),
        ScratchBlock::ControlIf(
            Input::Obj(ScratchObject::Bool(false)),
            vec![ScratchBlock::VarSet(Ptr(3), Input::new_num(1.0))],
        ),
        ScratchBlock::ControlIf(
            Input::Obj(ScratchObject::String("hello".to_owned())),
            vec![ScratchBlock::VarSet(Ptr(4), Input::new_num(1.0))],
        ),
        ScratchBlock::ControlIf(
            Input::Obj(ScratchObject::String(String::new())),
            vec![ScratchBlock::VarSet(Ptr(5), Input::new_num(1.0))],
        ),
        ScratchBlock::ControlIf(
            Input::Obj(ScratchObject::String("true".to_owned())),
            vec![ScratchBlock::VarSet(Ptr(6), Input::new_num(1.0))],
        ),
        ScratchBlock::ControlIf(
            Input::Obj(ScratchObject::String("false".to_owned())),
            vec![ScratchBlock::VarSet(Ptr(7), Input::new_num(1.0))],
        ),
        // nested statements
        ScratchBlock::ControlIf(
            Input::Obj(ScratchObject::Bool(true)),
            vec![
                ScratchBlock::ControlIf(
                    Input::Obj(ScratchObject::Bool(true)),
                    vec![ScratchBlock::VarSet(Ptr(8), Input::new_num(1.0))],
                ),
                ScratchBlock::ControlIf(
                    Input::Obj(ScratchObject::Bool(false)),
                    vec![ScratchBlock::VarSet(Ptr(9), Input::new_num(1.0))],
                ),
            ],
        ),
    ]
}

// Pass: *1-*5 true
#[allow(unused)]
pub fn if_else_test() -> Vec<ScratchBlock> {
    vec![
        ScratchBlock::WhenFlagClicked,
        ScratchBlock::ControlIfElse(
            Input::Obj(ScratchObject::Bool(true)),
            vec![ScratchBlock::VarSet(Ptr(0), Input::new_num(1.0))],
            vec![ScratchBlock::VarSet(Ptr(0), Input::new_num(0.0))],
        ),
        ScratchBlock::ControlIfElse(
            Input::Obj(ScratchObject::Bool(false)),
            vec![ScratchBlock::VarSet(Ptr(1), Input::new_num(0.0))],
            vec![ScratchBlock::VarSet(Ptr(1), Input::new_num(1.0))],
        ),
        ScratchBlock::ControlIfElse(
            Input::Obj(ScratchObject::String("hello".to_owned())),
            vec![ScratchBlock::VarSet(Ptr(2), Input::new_num(1.0))],
            vec![ScratchBlock::VarSet(Ptr(2), Input::new_num(0.0))],
        ),
        ScratchBlock::ControlIfElse(
            Input::Obj(ScratchObject::String(String::new())),
            vec![ScratchBlock::VarSet(Ptr(3), Input::new_num(0.0))],
            vec![ScratchBlock::VarSet(Ptr(3), Input::new_num(1.0))],
        ),
        ScratchBlock::ControlIfElse(
            Input::Obj(ScratchObject::String("true".to_owned())),
            vec![ScratchBlock::VarSet(Ptr(4), Input::new_num(1.0))],
            vec![ScratchBlock::VarSet(Ptr(4), Input::new_num(0.0))],
        ),
        ScratchBlock::ControlIfElse(
            Input::Obj(ScratchObject::String("false".to_owned())),
            vec![ScratchBlock::VarSet(Ptr(5), Input::new_num(0.0))],
            vec![ScratchBlock::VarSet(Ptr(5), Input::new_num(1.0))],
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
        ScratchBlock::VarSet(PI, Input::new_num(0.0)),
        ScratchBlock::VarSet(D, Input::new_num(1.0)),
        ScratchBlock::VarSet(I, Input::new_num(0.0)),
        // A test of nested repeat loops
        ScratchBlock::ControlRepeat(
            Input::new_num(1000.0),
            vec![ScratchBlock::ControlRepeat(
                Input::new_num(1000.0),
                vec![
                    // PI += ((8 * (I % 2)) - 4) / D
                    ScratchBlock::VarSet(
                        PI,
                        Input::new_block(ScratchBlock::OpAdd(
                            Input::new_block(ScratchBlock::VarRead(PI)),
                            Input::new_block(ScratchBlock::OpDiv(
                                Input::new_block(ScratchBlock::OpSub(
                                    Input::new_block(ScratchBlock::OpMul(
                                        Input::new_num(8.0),
                                        Input::new_block(ScratchBlock::OpMod(
                                            Input::new_block(ScratchBlock::VarRead(I)),
                                            Input::new_num(2.0),
                                        )),
                                    )),
                                    Input::new_num(4.0),
                                )),
                                Input::new_block(ScratchBlock::VarRead(D)),
                            )),
                        )),
                    ),
                    ScratchBlock::VarChange(D, Input::new_num(2.0)),
                    ScratchBlock::VarChange(I, Input::new_num(1.0)),
                ],
            )],
        ),
    ]
}

#[allow(unused)]
pub fn nested_repeat() -> Vec<ScratchBlock> {
    vec![
        ScratchBlock::WhenFlagClicked,
        ScratchBlock::ControlRepeat(
            Input::new_num(9.0),
            vec![ScratchBlock::ControlRepeat(
                Input::new_num(11.0),
                vec![ScratchBlock::VarSet(
                    Ptr(0),
                    Input::new_block(ScratchBlock::OpStrJoin(
                        Input::new_block(ScratchBlock::VarRead(Ptr(0))),
                        Input::Obj(ScratchObject::String("H".to_owned())),
                    )),
                )],
            )],
        ),
    ]
}

#[allow(unused)]
pub fn repeat_until() -> Vec<ScratchBlock> {
    vec![
        ScratchBlock::WhenFlagClicked,
        ScratchBlock::VarSet(Ptr(0), Input::new_num(0.0)),
        ScratchBlock::ControlRepeatUntil(
            Input::new_block(ScratchBlock::OpCmpGreater(
                Input::new_block(ScratchBlock::VarRead(Ptr(0))),
                Input::new_num(10.0),
            )),
            vec![
                ScratchBlock::VarSet(Ptr(1), Input::new_num(0.0)),
                ScratchBlock::ControlRepeatUntil(
                    Input::new_block(ScratchBlock::OpCmpGreater(
                        Input::new_block(ScratchBlock::VarRead(Ptr(1))),
                        Input::new_num(20.0),
                    )),
                    vec![ScratchBlock::VarChange(Ptr(1), Input::new_num(1.0))],
                ),
                ScratchBlock::VarChange(Ptr(0), Input::new_num(1.0)),
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
            Input::Block(Box::new(ScratchBlock::OpStrJoin(
                Input::Obj(ScratchObject::String("hello".to_owned())),
                Input::Obj(ScratchObject::String("world".to_owned())),
            ))),
        ),
        ScratchBlock::VarSet(
            Ptr(1),
            Input::Block(Box::new(ScratchBlock::OpStrJoin(
                Input::Obj(ScratchObject::String("hello".to_owned())),
                Input::new_num(1.0),
            ))),
        ),
        ScratchBlock::VarSet(
            Ptr(2),
            Input::Block(Box::new(ScratchBlock::OpStrJoin(
                Input::new_num(1.0),
                Input::Obj(ScratchObject::String("world".to_owned())),
            ))),
        ),
        ScratchBlock::VarSet(
            Ptr(3),
            Input::Block(Box::new(ScratchBlock::OpStrJoin(
                Input::Obj(ScratchObject::Bool(true)),
                Input::new_num(2.0),
            ))),
        ),
        ScratchBlock::VarSet(
            Ptr(4),
            Input::Block(Box::new(ScratchBlock::OpStrLen(Input::Obj(
                ScratchObject::Bool(true),
            )))),
        ),
    ]
}

#[allow(unused)]
pub fn random() -> Vec<ScratchBlock> {
    vec![
        ScratchBlock::WhenFlagClicked,
        ScratchBlock::VarSet(
            Ptr(0),
            Input::new_block(ScratchBlock::OpRandom(
                Input::new_num(0.0),
                Input::new_num(100.0),
            )),
        ),
        ScratchBlock::VarSet(
            Ptr(1),
            Input::new_block(ScratchBlock::OpRandom(
                Input::new_num(1.0),
                Input::new_num(2.5),
            )),
        ),
        ScratchBlock::VarSet(
            Ptr(2),
            Input::new_block(ScratchBlock::OpRandom(
                Input::Obj(ScratchObject::String("1".to_owned())),
                Input::Obj(ScratchObject::String("2".to_owned())),
            )),
        ),
        ScratchBlock::VarSet(
            Ptr(3),
            Input::new_block(ScratchBlock::OpRandom(
                Input::Obj(ScratchObject::String("1.0".to_owned())),
                Input::Obj(ScratchObject::String("2".to_owned())),
            )),
        ),
        ScratchBlock::ControlRepeat(
            Input::new_num(100_000.0),
            vec![ScratchBlock::VarSet(
                Ptr(4),
                Input::new_block(ScratchBlock::OpRandom(
                    Input::new_num(0.0),
                    Input::new_num(100.0),
                )),
            )],
        ),
    ]
}

#[allow(unused)]
pub fn math_add_test() -> Vec<ScratchBlock> {
    vec![
        ScratchBlock::WhenFlagClicked,
        ScratchBlock::VarSet(
            Ptr(0),
            Input::new_block(ScratchBlock::OpAdd(
                Input::new_num(50.0),
                Input::new_num(25.0),
            )),
        ),
        ScratchBlock::VarSet(
            Ptr(1),
            Input::new_block(ScratchBlock::OpAdd(
                Input::new_num(-500.0),
                Input::new_num(25.0),
            )),
        ),
        ScratchBlock::VarSet(
            Ptr(2),
            Input::new_block(ScratchBlock::OpAdd(
                Input::new_num(-500.0),
                Input::new_num(-25.0),
            )),
        ),
        ScratchBlock::VarSet(
            Ptr(3),
            Input::new_block(ScratchBlock::OpAdd(
                Input::new_num(2.54),
                Input::new_num(6.28),
            )),
        ),
        ScratchBlock::VarSet(
            Ptr(4),
            ScratchBlock::OpAdd(2.54.into(), (-6.28).into()).into(),
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
            ScratchBlock::OpAdd(1.0.into(), (0.0 / 0.0).into()).into(),
        ),
        ScratchBlock::VarSet(
            Ptr(11),
            ScratchBlock::OpAdd((0.0 / 0.0).into(), 1.0.into()).into(),
        ),
    ]
}

#[allow(unused)]
pub fn math_sub_test() -> Vec<ScratchBlock> {
    vec![
        ScratchBlock::WhenFlagClicked,
        ScratchBlock::VarSet(
            Ptr(0),
            Input::new_block(ScratchBlock::OpSub(
                Input::new_num(50.0),
                Input::new_num(25.0),
            )),
        ),
        ScratchBlock::VarSet(
            Ptr(1),
            Input::new_block(ScratchBlock::OpSub(
                Input::new_num(-500.0),
                Input::new_num(25.0),
            )),
        ),
        ScratchBlock::VarSet(
            Ptr(2),
            Input::new_block(ScratchBlock::OpSub(
                Input::new_num(-500.0),
                Input::new_num(-25.0),
            )),
        ),
        ScratchBlock::VarSet(Ptr(3), ScratchBlock::OpSub(2.54.into(), 6.28.into()).into()),
        ScratchBlock::VarSet(
            Ptr(4),
            ScratchBlock::OpSub(2.54.into(), (-6.28).into()).into(),
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
            ScratchBlock::OpSub(1.0.into(), (0.0 / 0.0).into()).into(),
        ),
        ScratchBlock::VarSet(
            Ptr(11),
            ScratchBlock::OpSub((0.0 / 0.0).into(), 1.0.into()).into(),
        ),
    ]
}

#[allow(unused)]
pub fn math_mul_test() -> Vec<ScratchBlock> {
    vec![
        ScratchBlock::WhenFlagClicked,
        ScratchBlock::VarSet(
            Ptr(0),
            Input::new_block(ScratchBlock::OpMul(
                Input::new_num(50.0),
                Input::new_num(2.0),
            )),
        ),
        ScratchBlock::VarSet(
            Ptr(1),
            Input::new_block(ScratchBlock::OpMul(
                Input::new_num(-50.0),
                Input::new_num(2.0),
            )),
        ),
        ScratchBlock::VarSet(
            Ptr(2),
            Input::new_block(ScratchBlock::OpMul(
                Input::new_num(-50.0),
                Input::new_num(-2.0),
            )),
        ),
        ScratchBlock::VarSet(
            Ptr(3),
            Input::new_block(ScratchBlock::OpMul(
                Input::new_num(2.54),
                Input::new_num(6.28),
            )),
        ),
        ScratchBlock::VarSet(
            Ptr(4),
            ScratchBlock::OpMul(2.54.into(), (-6.28).into()).into(),
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
            ScratchBlock::OpMul(1.0.into(), (0.0 / 0.0).into()).into(),
        ),
        ScratchBlock::VarSet(
            Ptr(17),
            ScratchBlock::OpMul((0.0 / 0.0).into(), 1.0.into()).into(),
        ),
    ]
}

#[allow(unused)]
pub fn math_div_test() -> Vec<ScratchBlock> {
    vec![
        ScratchBlock::WhenFlagClicked,
        ScratchBlock::VarSet(
            Ptr(0),
            Input::new_block(ScratchBlock::OpDiv(
                Input::new_num(50.0),
                Input::new_num(2.0),
            )),
        ),
        ScratchBlock::VarSet(
            Ptr(1),
            Input::new_block(ScratchBlock::OpDiv(
                Input::new_num(-50.0),
                Input::new_num(2.0),
            )),
        ),
        ScratchBlock::VarSet(
            Ptr(2),
            Input::new_block(ScratchBlock::OpDiv(
                Input::new_num(-50.0),
                Input::new_num(-2.0),
            )),
        ),
        ScratchBlock::VarSet(
            Ptr(3),
            Input::new_block(ScratchBlock::OpDiv(
                Input::new_num(3.5),
                Input::new_num(2.5),
            )),
        ),
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
            ScratchBlock::OpDiv(1.0.into(), (0.0 / 0.0).into()).into(),
        ),
        ScratchBlock::VarSet(
            Ptr(17),
            ScratchBlock::OpDiv((0.0 / 0.0).into(), 1.0.into()).into(),
        ),
    ]
}

#[allow(unused)]
fn run(program: Vec<ScratchBlock>, memory: &[ScratchObject]) {
    let mut builder = settings::builder();
    builder.set("opt_level", "speed").unwrap();
    let flags = settings::Flags::new(builder);

    let isa = match isa::lookup(Triple::host()) {
        Err(err) => panic!("Error looking up target: {}", err),
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

    for block in &program {
        compile_block(
            block,
            &mut builder,
            &mut code_block,
            &mut variable_type_data,
            memory,
        );
    }

    builder.seal_block(code_block);

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
        run(code, &memory);
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
    pub fn b_arithmetic() {
        let memory = run_code(arithmetic());
        assert_eq!(memory[4].convert_to_number(), 14.0);
        assert_eq!(memory[5].convert_to_number(), 1.25);
        assert_eq!(memory[6].convert_to_number(), 194.0);
        assert_eq!(memory[7].convert_to_number(), 1.0);
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
        assert_eq!(memory[3].convert_to_number(), 8.82);
        assert_eq!(memory[4].convert_to_number(), -3.74);
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
        assert_eq!(memory[3].convert_to_number(), -3.74);
        assert_eq!(memory[4].convert_to_number(), 8.82);
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
        assert_eq!(memory[3].convert_to_number(), 15.9512);
        assert_eq!(memory[4].convert_to_number(), -15.9512);
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
