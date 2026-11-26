use cranelift::{codegen::ir::types::I64, frontend::FunctionBuilder};

use crate::{callbacks::list, compiler::Compiler, effects::VarTypeChecked};

impl<'a> Compiler<'a> {
    pub fn list_clear(&mut self, builder: &mut FunctionBuilder<'_>, l: &crate::ListId) {
        let (ptr, ty) = self.lists.get(*l, builder, &mut self.constants);
        self.call_function(
            builder,
            match ty {
                VarTypeChecked::Number => list::CLEAR_NUM,
                VarTypeChecked::Bool => list::CLEAR_BOOL,
                VarTypeChecked::String => list::CLEAR_STR,
                VarTypeChecked::Object => list::CLEAR_OBJ,
            },
            &[I64],
            &[],
            &[ptr],
        );
    }
}
