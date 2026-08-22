use std::collections::HashMap;

use cranelift::{
    codegen::ir::{
        InstBuilder, MemFlags,
        types::{F64, I64},
    },
    frontend::FunctionBuilder,
};

use crate::{
    Ptr, ScratchObject,
    compiler::VarTypeChecked,
    constant_set::ConstantMap,
    effects::{Effects, VariableWrite},
    input_primitives::ScratchValue,
    variable_storage::{VarStore, VariableSlot},
};

#[derive(Clone)]
pub struct GenericVarStore {
    variable_types: HashMap<Ptr, VariableWrite>,
    memory: *const ScratchObject,
}

impl GenericVarStore {
    pub fn new(memory: &[ScratchObject]) -> Self {
        GenericVarStore {
            variable_types: HashMap::new(),
            memory: memory.as_ptr(),
        }
    }
}

impl VarStore for GenericVarStore {
    fn clone_box(&self) -> Box<dyn VarStore> {
        Box::new(self.clone())
    }

    fn get(
        &self,
        ptr: Ptr,
        builder: &mut FunctionBuilder<'_>,
        constants: &mut ConstantMap,
    ) -> super::VariableSlot {
        let load_ptr = constants.get_int(self.memory as usize as i64, builder);
        let offset = (ptr.0 * std::mem::size_of::<ScratchObject>()) as i32;
        let variable = self.variable_types.get(&ptr).copied().unwrap_or_default();

        let mut load_val = |t, off| {
            builder
                .ins()
                .load(t, MemFlags::new(), load_ptr, offset + off)
        };

        let val = match variable.ty {
            VarTypeChecked::Number => ScratchValue::Num(load_val(F64, 8)),
            VarTypeChecked::Bool => ScratchValue::Bool(load_val(I64, 8)),
            VarTypeChecked::String => {
                ScratchValue::String([load_val(I64, 8), load_val(I64, 16), load_val(I64, 24)])
            }
            VarTypeChecked::Object => ScratchValue::Object([
                load_val(I64, 0),
                load_val(I64, 8),
                load_val(I64, 16),
                load_val(I64, 24),
            ]),
        };
        VariableSlot {
            val,
            skip_nan: variable.skip_nan,
        }
    }

    fn store(
        &mut self,
        ptr: Ptr,
        value: VariableSlot,
        builder: &mut FunctionBuilder<'_>,
        constants: &mut ConstantMap,
    ) {
        let load_ptr = constants.get_int(self.memory as usize as i64, builder);
        let offset = (ptr.0 * std::mem::size_of::<ScratchObject>()) as i32;

        let old_var = self.variable_types.get(&ptr).copied().unwrap_or_default();
        let new_ty = value.val.into();

        if old_var.ty != new_ty
            && let Some(id) = new_ty.get_id()
        {
            let id = constants.get_int(id, builder);
            builder.ins().store(MemFlags::new(), id, load_ptr, offset);
        }

        let mut store_val = |v, off| {
            builder
                .ins()
                .store(MemFlags::new(), v, load_ptr, offset + off)
        };

        match value.val {
            ScratchValue::Num(value) | ScratchValue::Bool(value) => {
                store_val(value, 8);
            }
            ScratchValue::String([i1, i2, i3]) => {
                store_val(i1, 8);
                store_val(i2, 16);
                store_val(i3, 24);
            }
            ScratchValue::Object([i1, i2, i3, i4]) => {
                store_val(i1, 0);
                store_val(i2, 8);
                store_val(i3, 16);
                store_val(i4, 24);
            }
        }

        self.variable_types.insert(ptr, value.into());
    }

    fn get_type(&self, ptr: Ptr) -> VariableWrite {
        self.variable_types.get(&ptr).copied().unwrap_or_default()
    }

    fn extend(&mut self, info: Vec<(Ptr, VariableSlot)>) {
        for (ptr, value) in info {
            self.variable_types.insert(ptr, value.into());
        }
    }

    fn uses_block_params(&self) -> bool {
        false
    }

    // `GenericVarStore` doesn't have persistent state
    fn save(&self, _: &mut FunctionBuilder, _: &mut ConstantMap, _: &[ScratchObject], _: &Effects) {
    }
    fn reinit(
        &mut self,
        _: &mut FunctionBuilder,
        _: &mut ConstantMap,
        _: &[ScratchObject],
        _: &Effects,
    ) {
    }
}
