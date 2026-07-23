use std::collections::HashMap;

use cranelift::prelude::{FunctionBuilder, InstBuilder, Value, types::I64};
use ordered_float::OrderedFloat;

#[derive(PartialEq, Eq, Hash, Clone, Debug)]
pub enum ConstantType {
    Float(OrderedFloat<f64>),
    Int(i64),
}

#[derive(Default, Clone)]
pub struct ConstantMap {
    map: HashMap<ConstantType, Value>,
}

impl std::fmt::Debug for ConstantMap {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.map.fmt(f)
    }
}

impl ConstantMap {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn get_int(&mut self, num: i64, builder: &mut FunctionBuilder) -> Value {
        if let Some(value) = self.map.get(&ConstantType::Int(num)) {
            *value
        } else {
            let value = builder.ins().iconst(I64, num);
            self.map.insert(ConstantType::Int(num), value);
            value
        }
    }

    pub fn get_float(&mut self, num: f64, builder: &mut FunctionBuilder) -> Value {
        if let Some(value) = self.map.get(&ConstantType::Float(OrderedFloat(num))) {
            *value
        } else {
            let value = builder.ins().f64const(num);
            self.map
                .insert(ConstantType::Float(OrderedFloat(num)), value);
            value
        }
    }

    pub fn clear(&mut self) {
        self.map.clear();
    }
}
