mod blocks;
mod callbacks;
mod compile_fn;
mod compiler;
mod constant_set;
pub mod data_types;
mod effects;
pub mod error;
pub mod graphics;
mod input_primitives;
mod ins_shortcuts;
pub mod runtime;
mod tests;
mod variable_storage;

pub use callbacks::print_function_addresses;
pub use compiler::{MEMORY, ScratchBlock};
pub use data_types::ScratchObject;
pub use graphics::{
    CostumeData, CostumeId, GraphicsState, RunState, SpriteData, SpriteId, SpriteLoadData,
};
pub use input_primitives::{Input, Ptr};
pub use runtime::{ProjectBuilder, Runtime, SpriteBuilder};

pub fn program_pi() -> Vec<ScratchBlock> {
    fn set_var(ptr: Ptr, input: impl Into<Input>) -> ScratchBlock {
        ScratchBlock::VarSet(ptr, input.into())
    }

    const PI: Ptr = Ptr(0);
    const D: Ptr = Ptr(1);
    const I: Ptr = Ptr(2);

    vec![
        set_var(PI, 0.0),
        set_var(D, 1.0),
        set_var(I, 0.0),
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
