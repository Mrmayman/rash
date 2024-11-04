use crate::{
    compiler::ScratchBlock,
    data_types::ScratchObject,
    input_primitives::{Input, Ptr},
};

pub fn test_math() -> Vec<ScratchBlock> {
    vec![
        ScratchBlock::WhenFlagClicked,
        ScratchBlock::VarSet(Ptr(0), Input::Obj(ScratchObject::Number(2.0))),
        ScratchBlock::VarSet(Ptr(1), Input::Obj(ScratchObject::Bool(true))),
        ScratchBlock::VarSet(Ptr(2), Input::Obj(ScratchObject::Bool(false))),
        ScratchBlock::VarSet(
            Ptr(3),
            Input::Obj(ScratchObject::String("192.0".to_owned())),
        ),
        ScratchBlock::VarSet(
            Ptr(4),
            Input::Block(Box::new(ScratchBlock::OpAdd(
                Input::Obj(ScratchObject::Number(2.0)),
                Input::Block(Box::new(ScratchBlock::OpMul(
                    Input::Obj(ScratchObject::String("3.0".to_owned())),
                    Input::Obj(ScratchObject::Number(4.0)),
                ))),
            ))),
        ),
        ScratchBlock::VarSet(
            Ptr(5),
            Input::Block(Box::new(ScratchBlock::OpSub(
                Input::Obj(ScratchObject::Number(2.0)),
                Input::Block(Box::new(ScratchBlock::OpDiv(
                    Input::Obj(ScratchObject::Number(3.0)),
                    Input::Obj(ScratchObject::Number(4.0)),
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
            // Input::Block(Box::new(ScratchBlock::OpJoin(
            //     Input::Block(Box::new(ScratchBlock::VarRead(Ptr(7)))),
            //     Input::Obj(ScratchObject::String("world".to_owned())),
            // ))),
            Input::Block(Box::new(ScratchBlock::OpAdd(
                Input::Block(Box::new(ScratchBlock::VarRead(Ptr(7)))),
                Input::Obj(ScratchObject::Bool(true)),
            ))),
        ),
    ]
}

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
            Input::Obj(ScratchObject::Number(100000.0)),
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

pub fn pi() -> Vec<ScratchBlock> {
    const PI: Ptr = Ptr(0);
    const D: Ptr = Ptr(1);
    const I: Ptr = Ptr(2);
    vec![
        ScratchBlock::WhenFlagClicked,
        ScratchBlock::VarSet(PI, Input::new_num(0.0)),
        ScratchBlock::VarSet(D, Input::new_num(1.0)),
        ScratchBlock::VarSet(I, Input::new_num(0.0)),
        ScratchBlock::ControlRepeat(
            Input::new_num(1000000.0),
            vec![
                ScratchBlock::VarSet(
                    PI,
                    Input::new_block(ScratchBlock::OpAdd(
                        Input::new_block(ScratchBlock::VarRead(PI)),
                        Input::new_block(ScratchBlock::OpDiv(
                            Input::new_block(ScratchBlock::OpSub(
                                Input::new_block(ScratchBlock::OpMul(
                                    Input::new_num(8.0),
                                    Input::new_block(ScratchBlock::OpMod(
                                        // Input::new_num(2.0),
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
                ScratchBlock::VarSet(
                    D,
                    Input::new_block(ScratchBlock::OpAdd(
                        Input::new_block(ScratchBlock::VarRead(D)),
                        Input::new_num(2.0),
                    )),
                ),
                ScratchBlock::VarSet(
                    I,
                    // Input::new_num(2.0),
                    Input::new_block(ScratchBlock::OpAdd(
                        Input::new_block(ScratchBlock::VarRead(I)),
                        Input::new_num(1.0),
                    )),
                ),
            ],
        ),
    ]
}
