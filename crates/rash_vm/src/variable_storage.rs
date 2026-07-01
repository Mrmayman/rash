use std::collections::{HashMap, HashSet};

use cranelift::prelude::{FunctionBuilder, InstBuilder, MemFlags, types::I64};

use crate::{
    compiler::VarType,
    constant_set::ConstantMap,
    data_types::{ID_BOOL, ID_NUMBER, ID_STRING, ScratchObject},
    effects::Effects,
    input_primitives::{Ptr, ReturnValue, STRINGS_TO_DROP},
};

/// Stores the current value of every variable used by a compiled function.
///
/// Only variables listed in the provided [`Effects`] are tracked. Their values
/// are loaded into this cache before code generation and can later be written
/// back to the backing scratch memory with [`VariableStorage::save`].
///
/// This avoids repeatedly loading and storing variable objects from memory
/// while generating code.
#[derive(Clone)]
pub struct VariableStorage<'a> {
    pub variable_vals: HashMap<Ptr, ReturnValue>,
    memory: &'a [ScratchObject],
}

impl<'a> VariableStorage<'a> {
    /// Creates a [`VariableStorage`] for all variables referenced by `effects`.
    ///
    /// Every variable that is either read or written is loaded from `memory` into
    /// the local cache as a raw [`ReturnValue::Object`]. Later compilation passes
    /// may replace these cached values with more specific variants such as
    /// [`ReturnValue::Num`] or [`ReturnValue::Bool`].
    pub fn new(
        builder: &mut FunctionBuilder,
        effects: &Effects,
        memory: &'a [ScratchObject],
        constants: &mut ConstantMap,
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

            variable_vals.insert(var, ReturnValue::Object([i1, i2, i3, i4]));
        }

        Self {
            variable_vals,
            memory,
        }
    }

    /// Reloads every cached variable from the backing scratch memory.
    ///
    /// This discards any cached values and replaces them with the current contents
    /// of memory.
    pub fn init(
        &mut self,
        builder: &mut FunctionBuilder,
        constants: &mut ConstantMap,
        memory: &[ScratchObject],
    ) {
        for (var, val) in self.variable_vals.iter_mut() {
            let ptr = var.constant(constants, builder, memory);
            let i1 = builder.ins().load(I64, MemFlags::new(), ptr, 0);
            let i2 = builder.ins().load(I64, MemFlags::new(), ptr, 8);
            let i3 = builder.ins().load(I64, MemFlags::new(), ptr, 16);
            let i4 = builder.ins().load(I64, MemFlags::new(), ptr, 24);

            *val = ReturnValue::Object([i1, i2, i3, i4]);
        }
    }

    /// Writes all cached variables back to the backing scratch memory.
    ///
    /// Values are serialized according to their current [`ReturnValue`] variant,
    /// preserving both their type tag and payload.
    pub fn save(&self, builder: &mut FunctionBuilder, constants: &mut ConstantMap) {
        for (ptr, val) in &self.variable_vals {
            let ptr = ptr.constant(constants, builder, self.memory);

            match val {
                ReturnValue::Num(value) => {
                    let id = constants.get_int(ID_NUMBER, builder);
                    builder.ins().store(MemFlags::new(), id, ptr, 0);
                    builder.ins().store(MemFlags::new(), *value, ptr, 8);
                }
                ReturnValue::Bool(value) => {
                    let id = constants.get_int(ID_BOOL, builder);
                    builder.ins().store(MemFlags::new(), id, ptr, 0);
                    builder.ins().store(MemFlags::new(), *value, ptr, 8);
                }
                ReturnValue::String(vals) => {
                    let id = constants.get_int(ID_STRING, builder);
                    builder.ins().store(MemFlags::new(), id, ptr, 0);

                    for (i, val) in vals.iter().enumerate() {
                        builder
                            .ins()
                            .store(MemFlags::new(), *val, ptr, ((i + 1) * 8) as i32);
                    }
                }
                ReturnValue::Object(vals) => {
                    for (i, val) in vals.iter().enumerate() {
                        builder
                            .ins()
                            .store(MemFlags::new(), *val, ptr, (i * 8) as i32);
                    }
                }
            }
        }
    }

    /*pub fn get(&self, ptr: Ptr, builder: &mut FunctionBuilder) -> (Value, Value, Value, Value) {
        let offset = self.variable_offsets[&ptr] as i32;
        (
            builder.ins().stack_load(I64, self.slot, offset),
            builder.ins().stack_load(I64, self.slot, offset + 8),
            builder.ins().stack_load(I64, self.slot, offset + 16),
            builder.ins().stack_load(I64, self.slot, offset + 24),
        )
    }*/

    /// Returns the known type of a cached variable.
    ///
    /// If the value is still stored as a raw object loaded from memory,
    /// `None` is returned because its type has not yet been inferred.
    pub fn get_type(&self, ptr: Ptr) -> Option<VarType> {
        self.variable_vals.get(&ptr).and_then(|val| match val {
            ReturnValue::Num(_) => Some(VarType::Number),
            ReturnValue::Bool(_) => Some(VarType::Bool),
            ReturnValue::String(_) => Some(VarType::String),
            ReturnValue::Object(_) => None,
        })
    }

    /// Stores a constant number in the local cache.
    pub fn store_f64(
        &mut self,
        ptr: Ptr,
        builder: &mut FunctionBuilder<'_>,
        num: f64,
        constants: &mut ConstantMap,
    ) {
        let v = constants.get_float(num, builder);
        self.set_retval(ptr, ReturnValue::Num(v));
    }

    /// Stores a constant boolean in the local cache.
    pub fn store_bool(
        &mut self,
        ptr: Ptr,
        builder: &mut FunctionBuilder<'_>,
        num: bool,
        constants: &mut ConstantMap,
    ) {
        let v = constants.get_int(i64::from(num), builder);
        self.set_retval(ptr, ReturnValue::Bool(v));
    }

    /// Stores a constant string in the local cache.
    ///
    /// The string is heap-allocated and registered for later cleanup before its
    /// raw representation is embedded into the cached value.
    pub fn store_string(
        &mut self,
        ptr: Ptr,
        builder: &mut FunctionBuilder<'_>,
        string: &str,
        constants: &mut ConstantMap,
    ) {
        // Transmute the String into a [i64; 4] array
        let arr: [i64; 3] = unsafe { std::mem::transmute(string.to_owned()) };
        STRINGS_TO_DROP.lock().unwrap().insert(arr);
        let id = constants.get_int(ID_STRING, builder);
        let i1 = constants.get_int(arr[0], builder);
        let i2 = constants.get_int(arr[1], builder);
        let i3 = constants.get_int(arr[2], builder);

        self.set_retval(ptr, ReturnValue::Object([id, i1, i2, i3]));
    }

    /// Replaces the cached value of a variable.
    pub fn set_retval(&mut self, ptr: Ptr, value: ReturnValue) {
        let val = self.get_retval(ptr);
        *val = value;
    }

    /// Returns a mutable reference to a cached variable.
    ///
    /// Panics if the variable was not included in the original [`Effects`].
    fn get_retval(&mut self, ptr: Ptr) -> &mut ReturnValue {
        if !self.variable_vals.contains_key(&ptr) {
            panic!(
                "Stack Cache: Variable {ptr:?} not found!\nOffsets: {:?}",
                self.variable_vals
            );
        }

        self.variable_vals.get_mut(&ptr).unwrap()
    }
}
