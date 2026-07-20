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
mod variable_storage;

#[cfg(test)]
mod tests;

pub use callbacks::print_function_addresses;
pub use compiler::{MEMORY, ScratchBlock};
pub use data_types::ScratchObject;
pub use graphics::{
    CostumeData, CostumeId, GraphicsState, RunState, SpriteData, SpriteId, SpriteLoadData,
};
pub use input_primitives::{Input, Ptr};
pub use runtime::{ProjectBuilder, Runtime, SpriteBuilder};
use smol_str::SmolStr;

mod config {
    /// Scratch has a special edge case for math with NaN.
    /// Any operation with NaN will be treated as
    /// an operation with 0.
    ///
    /// For example, `NaN + 1` will be `0 + 1`.
    ///
    /// This is a special case for Scratch, and is not
    /// a standard behavior for most programming languages.
    /// Enabling this check adds special behavior for NaN in the
    /// compiled code, making it more correct but slower.
    ///
    /// # Performance
    /// Pi benchmark:
    /// - Without NaN check: `4.6 ms`
    /// - With NaN check: `6.5 ms`
    pub const ARITHMETIC_NAN_CHECK: bool = true;

    /// Sacrifices some floating point precision for
    /// slightly faster division in certain scenarios
    pub const IMPRECISE_DIVISION: bool = false;
}

pub mod builder {
    use crate::{Input, Ptr, ScratchBlock};

    #[must_use]
    pub fn program_pi() -> Vec<ScratchBlock> {
        const PI: Ptr = Ptr(0);
        const D: Ptr = Ptr(1);
        const I: Ptr = Ptr(2);

        vec![
            set(PI, 0.0),
            set(D, 1.0),
            set(I, 0.0),
            c_repeat(
                1_000_000.0,
                vec![
                    // PI += ((8 * (I % 2)) - 4) / D
                    change(PI, fdiv(fsub(fmul(8.0, fmod(I, 2.0)), 4.0), D)),
                    change(D, 2.0),
                    change(I, 1.0),
                ],
            ),
        ]
    }

    #[must_use]
    pub fn c_if(input: impl Into<Input>, then: Vec<ScratchBlock>) -> ScratchBlock {
        ScratchBlock::ControlIf(input.into(), then)
    }

    #[must_use]
    pub fn c_if_else(
        input: impl Into<Input>,
        then: Vec<ScratchBlock>,
        else_: Vec<ScratchBlock>,
    ) -> ScratchBlock {
        ScratchBlock::ControlIfElse(input.into(), then, else_)
    }

    #[must_use]
    pub fn c_repeat(times: impl Into<Input>, blocks: Vec<ScratchBlock>) -> ScratchBlock {
        ScratchBlock::ControlRepeat(times.into(), blocks)
    }

    #[must_use]
    pub fn set(ptr: Ptr, input: impl Into<Input>) -> ScratchBlock {
        ScratchBlock::VarSet(ptr, input.into())
    }

    #[must_use]
    pub fn change(ptr: Ptr, value: impl Into<Input>) -> ScratchBlock {
        ScratchBlock::VarChange(ptr, value.into())
    }

    #[must_use]
    pub fn fadd(a: impl Into<Input>, b: impl Into<Input>) -> ScratchBlock {
        ScratchBlock::OpAdd(a.into(), b.into())
    }

    #[must_use]
    pub fn fsub(a: impl Into<Input>, b: impl Into<Input>) -> ScratchBlock {
        ScratchBlock::OpSub(a.into(), b.into())
    }

    #[must_use]
    pub fn fmul(a: impl Into<Input>, b: impl Into<Input>) -> ScratchBlock {
        ScratchBlock::OpMul(a.into(), b.into())
    }

    #[must_use]
    pub fn fdiv(a: impl Into<Input>, b: impl Into<Input>) -> ScratchBlock {
        ScratchBlock::OpDiv(a.into(), b.into())
    }

    #[must_use]
    pub fn fmod(a: impl Into<Input>, b: impl Into<Input>) -> ScratchBlock {
        ScratchBlock::OpMod(a.into(), b.into())
    }
}

fn stat(s: &'static str) -> SmolStr {
    SmolStr::new_static(s)
}
