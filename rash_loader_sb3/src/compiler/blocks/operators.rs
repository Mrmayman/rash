use std::collections::BTreeMap;

use rash_vm::bytecode::instructions::{DataPointer, Instruction};

use crate::{
    compiler::{structures::CompileResult, Compiler},
    json_struct::{Block, JsonBlock},
};

impl Compiler {
    pub fn c_operators_add(
        &mut self,
        block: &Block,
        blocks: &BTreeMap<String, JsonBlock>,
    ) -> CompileResult {
        let in1 = self.deal_with_input(block, "NUM1", blocks)?;
        let in2 = self.deal_with_input(block, "NUM2", blocks)?;

        self.thread_state.instructions.push(Instruction::MathAdd {
            a: self.allocator.register_get(in1, self.thread_id)?.into(),
            b: self.allocator.register_get(in2, self.thread_id)?.into(),
            result: DataPointer(self.allocator.register_get(in1, self.thread_id)?.0),
        });

        self.allocator.register_free(in2)?;

        Ok(Some(in1))
    }
}
