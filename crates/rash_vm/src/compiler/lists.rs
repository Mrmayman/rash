use cranelift::{codegen::ir::Value, frontend::FunctionBuilder};

use crate::{ListId, constant_set::ConstantMap, effects::VarTypeChecked, lists::ScratchListDummy};

pub struct ListManager<'a> {
    base_ptr: *mut ScratchListDummy,
    lists_ty: &'a [VarTypeChecked],
}

impl<'a> ListManager<'a> {
    pub fn new(lists: &[ScratchListDummy], lists_ty: &'a [VarTypeChecked]) -> Self {
        Self {
            base_ptr: lists.as_ptr().cast_mut(),
            lists_ty,
        }
    }

    pub fn get(
        &self,
        id: ListId,
        builder: &mut FunctionBuilder,
        constants: &mut ConstantMap,
    ) -> (Value, VarTypeChecked) {
        let ptr = unsafe { self.base_ptr.add(id.0) };
        (constants.get_int(ptr as i64, builder), self.lists_ty[id.0])
    }
}
