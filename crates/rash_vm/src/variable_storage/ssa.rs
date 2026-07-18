use std::collections::{HashMap, HashSet};

use cranelift::{
    codegen::ir::{InstBuilder, MemFlags, types::I64},
    frontend::FunctionBuilder,
};

use crate::{
    Ptr, ScratchObject,
    constant_set::ConstantMap,
    data_types::{ID_BOOL, ID_NUMBER, ID_STRING},
    effects::{Effects, VariableWrite},
    input_primitives::ScratchValue,
    variable_storage::VariableSlot,
};

/// Stores variables as SSA values.
///
/// This avoids repeatedly loading and storing variable objects from memory
/// while generating code.
#[derive(Clone)]
pub struct SsaVarStore {
    variable_vals: HashMap<Ptr, VariableSlot>,
}

impl SsaVarStore {
    /// Creates a [`VariableStorage`] for all variables referenced by `effects`.
    ///
    /// Every variable that is either read or written is loaded from `memory` into
    /// the local cache as a raw [`ReturnValue::Object`]. Later compilation passes
    /// may replace these cached values with more specific variants such as
    /// [`ReturnValue::Num`] or [`ReturnValue::Bool`].
    pub fn new(
        builder: &mut FunctionBuilder,
        effects: &Effects,
        constants: &mut ConstantMap,
        memory: &[ScratchObject],
    ) -> Self {
        let mut variable_vals = HashMap::new();

        let vars: HashSet<Ptr> = effects
            .reads
            .iter()
            .chain(effects.writes.keys())
            .copied()
            .collect();

        for var in vars {
            let ptr = var.constant(constants, builder, memory);
            let i1 = builder.ins().load(I64, MemFlags::new(), ptr, 0);
            let i2 = builder.ins().load(I64, MemFlags::new(), ptr, 8);
            let i3 = builder.ins().load(I64, MemFlags::new(), ptr, 16);
            let i4 = builder.ins().load(I64, MemFlags::new(), ptr, 24);

            variable_vals.insert(
                var,
                VariableSlot::normal(ScratchValue::Object([i1, i2, i3, i4])),
            );
        }

        Self { variable_vals }
    }

    fn get_inner(&self, ptr: Ptr) -> VariableSlot {
        let val = self.variable_vals.get(&ptr);
        assert!(
            val.is_some(),
            "variable {ptr:?} should be valid!\nvariables: {:?}",
            self.variable_vals
        );
        *val.unwrap()
    }
}

impl super::VarStore for SsaVarStore {
    fn clone_box(&self) -> Box<dyn super::VarStore> {
        Box::new(self.clone())
    }

    fn uses_block_params(&self) -> bool {
        true
    }

    fn store(
        &mut self,
        ptr: Ptr,
        value: VariableSlot,
        _: &mut FunctionBuilder<'_>,
        _: &mut ConstantMap,
    ) {
        assert!(
            self.variable_vals.contains_key(&ptr),
            "variable {ptr:?} should be valid!\nvariables: {:?}",
            self.variable_vals
        );

        *self.variable_vals.get_mut(&ptr).unwrap() = value;
    }

    fn get_type(&self, ptr: Ptr) -> VariableWrite {
        self.get_inner(ptr).into()
    }

    fn extend(&mut self, info: Vec<(Ptr, VariableSlot)>) {
        self.variable_vals.extend(info);
    }

    fn get(&self, ptr: Ptr, _: &mut FunctionBuilder<'_>, _: &mut ConstantMap) -> VariableSlot {
        self.get_inner(ptr)
    }

    fn save(
        &self,
        builder: &mut FunctionBuilder,
        constants: &mut ConstantMap,
        memory: &[ScratchObject],
    ) {
        for (ptr, val) in &self.variable_vals {
            let ptr = ptr.constant(constants, builder, memory);

            match &val.val {
                ScratchValue::Num(value) => {
                    let id = constants.get_int(ID_NUMBER, builder);
                    builder.ins().store(MemFlags::new(), id, ptr, 0);
                    builder.ins().store(MemFlags::new(), *value, ptr, 8);
                }
                ScratchValue::Bool(value) => {
                    let id = constants.get_int(ID_BOOL, builder);
                    builder.ins().store(MemFlags::new(), id, ptr, 0);
                    builder.ins().store(MemFlags::new(), *value, ptr, 8);
                }
                ScratchValue::String(vals) => {
                    let id = constants.get_int(ID_STRING, builder);
                    builder.ins().store(MemFlags::new(), id, ptr, 0);

                    for (i, val) in vals.iter().enumerate() {
                        builder
                            .ins()
                            .store(MemFlags::new(), *val, ptr, ((i + 1) * 8) as i32);
                    }
                }
                ScratchValue::Object(vals) => {
                    for (i, val) in vals.iter().enumerate() {
                        builder
                            .ins()
                            .store(MemFlags::new(), *val, ptr, (i * 8) as i32);
                    }
                }
            }
        }
    }

    fn reinit(
        &mut self,
        builder: &mut FunctionBuilder,
        constants: &mut ConstantMap,
        memory: &[ScratchObject],
    ) {
        for (var, val) in &mut self.variable_vals {
            let ptr = var.constant(constants, builder, memory);
            let i1 = builder.ins().load(I64, MemFlags::new(), ptr, 0);
            let i2 = builder.ins().load(I64, MemFlags::new(), ptr, 8);
            let i3 = builder.ins().load(I64, MemFlags::new(), ptr, 16);
            let i4 = builder.ins().load(I64, MemFlags::new(), ptr, 24);

            *val = VariableSlot::normal(ScratchValue::Object([i1, i2, i3, i4]));
        }
    }
}
