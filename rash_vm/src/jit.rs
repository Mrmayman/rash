use crate::{bytecode::instructions::Instruction, data_types::ScratchObject, vm_thread::Thread};

use cranelift::{
    codegen::{
        ir::{Function, UserFuncName},
        isa::CallConv,
        CodegenError,
    },
    prelude::*,
};

impl Thread {
    pub fn jit(&mut self, memory: *const ScratchObject) -> Result<(), JitError> {
        let mut builder = settings::builder();
        builder.set("opt_level", "speed").unwrap();
        let flags = settings::Flags::new(builder);

        let isa = isa::lookup(target_lexicon::Triple::host())
            .map_err(|_| JitError::ArchitectureNotSupported)?
            .finish(flags)?;

        let pointer_type = isa.pointer_type();

        let mut sig = Signature::new(CallConv::triple_default(isa.triple()));
        // The JITted function will take in a pointer to the VM memory.
        sig.params.push(AbiParam::new(pointer_type));

        let mut func = Function::with_name_signature(UserFuncName::user(0, 0), sig);
        let mut func_ctx = FunctionBuilderContext::new();
        let mut builder = FunctionBuilder::new(&mut func, &mut func_ctx);

        let memory_pointer = Variable::new(0);
        builder.declare_var(memory_pointer, pointer_type);

        let block = builder.create_block();
        builder.seal_block(block);

        builder.append_block_params_for_function_params(block);
        builder.switch_to_block(block);

        let zero = builder.ins().iconst(pointer_type, memory as i64);
        builder.def_var(memory_pointer, zero);

        let mem_flags = MemFlags::new();

        for instruction in self.code.iter() {
            match instruction {
                Instruction::MemSetToValue { ptr, value } => {
                    let memory_pointer = builder.use_var(memory_pointer);

                    let offset = builder.ins().iconst(pointer_type, ptr.0 as i64);

                    let size_of_object = std::mem::size_of::<ScratchObject>();
                    let size_of_object_value =
                        builder.ins().iconst(pointer_type, size_of_object as i64);
                    let offset_multiplied = builder.ins().imul(offset, size_of_object_value);

                    let memory_address = builder.ins().iadd(memory_pointer, offset_multiplied);

                    for offset in 0..size_of_object {
                        // builder
                        //     .ins()
                        //     .store(mem_flags, 0, memory_address, offset as u8);
                    }

                    // let cell_value = builder
                    //     .ins()
                    //     .load(types::I128X2, self.mem_flags, memory_address, 0);
                    // let cell_value = self.builder.ins().iadd_imm(cell_value, 1);
                    // self.builder
                    //     .ins()
                    //     .store(self.mem_flags, cell_value, memory_address, 0);
                    todo!();
                }
                _ => {
                    todo!()
                }
            }
        }
        Ok(())
    }
}

#[derive(Debug)]
pub enum JitError {
    ArchitectureNotSupported,
    CodegenError(CodegenError),
}

impl From<CodegenError> for JitError {
    fn from(value: CodegenError) -> Self {
        JitError::CodegenError(value)
    }
}
