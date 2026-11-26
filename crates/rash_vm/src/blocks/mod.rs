use cranelift::{
    codegen::ir::{Inst, UserExternalName},
    prelude::{FunctionBuilder, InstBuilder, Type, Value},
};

use crate::compiler::Compiler;

mod control;
mod custom_block;
mod list;
mod op;
mod var;

impl Compiler<'_> {
    pub fn call_function(
        &mut self,
        builder: &mut FunctionBuilder<'_>,
        name: UserExternalName,
        params: &[Type],
        returns: &[Type],
        arguments: &[Value],
    ) -> Inst {
        let func = self
            .func_store
            .get_function(self.call_conv, builder, name, params, returns);
        builder.ins().call(func, arguments)
    }
}
