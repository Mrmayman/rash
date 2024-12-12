use cranelift::{
    codegen::ir::Inst,
    prelude::{isa::CallConv, AbiParam, FunctionBuilder, InstBuilder, Signature, Type, Value},
};

use crate::compiler::Compiler;

pub mod control;
pub mod custom_block;
pub mod op;
pub mod var;

impl Compiler<'_> {
    pub fn call_function(
        &mut self,
        builder: &mut FunctionBuilder<'_>,
        func: usize,
        params: &[Type],
        returns: &[Type],
        arguments: &[Value],
    ) -> Inst {
        let func = self.constants.get_int(func as i64, builder);
        let sig = builder.import_signature({
            let mut sig = Signature::new(CallConv::SystemV);
            for param in params {
                sig.params.push(AbiParam::new(*param));
            }
            for ret in returns {
                sig.returns.push(AbiParam::new(*ret));
            }
            sig
        });
        builder.ins().call_indirect(sig, func, arguments)
    }
}
