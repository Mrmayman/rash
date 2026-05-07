use std::collections::BTreeMap;

use cranelift::prelude::{types::I64, FunctionBuilder, InstBuilder, Value};
use ordered_float::OrderedFloat;

#[derive(PartialEq, Eq, PartialOrd, Ord)]
pub enum ConstantType {
    Float(OrderedFloat<f64>),
    Int(i64),
}

pub struct ConstantMap {
    map: BTreeMap<ConstantType, Value>,
}

impl ConstantMap {
    pub fn new() -> Self {
        Self {
            map: BTreeMap::new(),
        }
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
