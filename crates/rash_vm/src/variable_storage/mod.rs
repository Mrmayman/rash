use cranelift::prelude::FunctionBuilder;
use smol_str::SmolStr;

use crate::{
    ScratchObject,
    constant_set::ConstantMap,
    data_types::ID_STRING,
    effects::{Effects, VariableWrite},
    input_primitives::{Ptr, ScratchValue},
};

mod ssa;
pub use ssa::SsaVarStore;
mod generic;
pub use generic::GenericVarStore;

pub trait VarStore {
    fn clone_box(&self) -> Box<dyn VarStore>;

    fn get(
        &self,
        ptr: Ptr,
        builder: &mut FunctionBuilder<'_>,
        constants: &mut ConstantMap,
    ) -> VariableSlot;
    fn store(
        &mut self,
        ptr: Ptr,
        value: VariableSlot,
        builder: &mut FunctionBuilder<'_>,
        constants: &mut ConstantMap,
    );

    fn get_type(&self, ptr: Ptr) -> VariableWrite;
    fn extend(&mut self, info: Vec<(Ptr, VariableSlot)>);

    fn uses_block_params(&self) -> bool;

    fn save(
        &self,
        builder: &mut FunctionBuilder,
        constants: &mut ConstantMap,
        memory: &[ScratchObject],
        effects: &Effects,
    );

    fn reinit(
        &mut self,
        builder: &mut FunctionBuilder,
        constants: &mut ConstantMap,
        memory: &[ScratchObject],
        effects: &Effects,
    );

    fn store_f64(
        &mut self,
        ptr: Ptr,
        builder: &mut FunctionBuilder<'_>,
        num: f64,
        constants: &mut ConstantMap,
    ) {
        let v = constants.get_float(num, builder);
        self.store(
            ptr,
            VariableSlot {
                val: ScratchValue::Num(v),
                skip_nan: !num.is_nan(),
            },
            builder,
            constants,
        );
    }

    fn store_bool(
        &mut self,
        ptr: Ptr,
        builder: &mut FunctionBuilder<'_>,
        num: bool,
        constants: &mut ConstantMap,
    ) {
        let v = constants.get_int(i64::from(num), builder);
        self.store(
            ptr,
            VariableSlot::skip_nan(ScratchValue::Bool(v)),
            builder,
            constants,
        );
    }

    #[must_use]
    fn store_string(
        &mut self,
        ptr: Ptr,
        builder: &mut FunctionBuilder<'_>,
        string: &str,
        constants: &mut ConstantMap,
    ) -> SmolStr {
        // Transmute the SmolStr into a [i64; 3] array
        let arr: [i64; 3] = unsafe { std::mem::transmute(SmolStr::from(string)) };

        let id = constants.get_int(ID_STRING, builder);
        let i1 = constants.get_int(arr[0], builder);
        let i2 = constants.get_int(arr[1], builder);
        let i3 = constants.get_int(arr[2], builder);

        self.store(
            ptr,
            VariableSlot::normal(ScratchValue::Object([id, i1, i2, i3])),
            builder,
            constants,
        );

        // # Safety
        //
        // The string is intentionally owned in two places:
        //
        // - The generated machine code.
        // - The compiler's `static_strings`.
        //
        // Neither mutates the string, and both the machine code AND the strings
        // are dropped with the `Runtime`, so there's no use-after-free.
        unsafe { std::mem::transmute(arr) }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct VariableSlot {
    pub val: ScratchValue,
    pub skip_nan: bool,
}

impl VariableSlot {
    pub fn normal(val: ScratchValue) -> VariableSlot {
        VariableSlot {
            val,
            skip_nan: false,
        }
    }

    pub fn skip_nan(val: ScratchValue) -> VariableSlot {
        VariableSlot {
            val,
            skip_nan: true,
        }
    }
}
