use std::collections::BTreeMap;

use rash_vm::bytecode::instructions::{DataPointer, Instruction};

use crate::{
    compiler::{
        error::CompilerError, structures::CompileResult, variable_allocator::VariableIdentifier,
        Compiler,
    },
    json_struct::{Block, JsonBlock},
};

impl Compiler {
    pub fn c_variable_set(
        &mut self,
        block: &Block,
        blocks: &BTreeMap<String, JsonBlock>,
    ) -> CompileResult {
        let register_id = self.deal_with_input(block, "VALUE", blocks)?;
        let register_vm_id = self.allocator.register_get(register_id, self.thread_id)?;

        const VARIABLE_HASH_INDEX: usize = 1;

        let variable_id = block
            .fields
            .get("VARIABLE")
            .ok_or(CompilerError::FieldInvalid(
                "block.fields.VARIABLE does not exist".to_owned(),
            ))?
            .as_array()
            .ok_or(CompilerError::FieldInvalid(
                "block.fields.VARIABLE, expected Array".to_owned(),
            ))?
            .get(VARIABLE_HASH_INDEX)
            .ok_or(CompilerError::FieldInvalid(
                "blocks.fields.VARIABLE[1] does not exist".to_owned(),
            ))?
            .as_str()
            .ok_or(CompilerError::FieldInvalid(
                "blocks.fields.VARIABLE[1], expected String".to_owned(),
            ))?
            .to_owned();

        self.thread_state
            .instructions
            .push(Instruction::MemSetToValue {
                ptr: DataPointer(
                    self.allocator
                        .variable_get(&VariableIdentifier::Hash(variable_id))?
                        .0,
                ),
                value: register_vm_id.into(),
            });

        self.allocator.register_free(register_id)?;

        Ok(None)
    }
}
