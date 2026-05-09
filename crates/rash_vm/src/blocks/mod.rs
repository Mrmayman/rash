use cranelift::{
    codegen::ir::Inst,
    prelude::{AbiParam, FunctionBuilder, InstBuilder, Signature, Type, Value, isa::CallConv},
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
        func: *const (),
        params: &[Type],
        returns: &[Type],
        arguments: &[Value],
    ) -> Inst {
        let func = self.constants.get_int(func as i64, builder);
        let mut sig = Signature::new(CallConv::SystemV);
        for param in params {
            sig.params.push(AbiParam::new(*param));
        }
        for ret in returns {
            sig.returns.push(AbiParam::new(*ret));
        }

        let sig = if let Some(sigref) = self.func_signatures.get(&sig) {
            *sigref
        } else {
            let r = builder.import_signature(sig.clone());
            self.func_signatures.insert(sig.clone(), r);
            r
        };
        builder.ins().call_indirect(sig, func, arguments)
    }
}
