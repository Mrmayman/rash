use crate::bytecode::instructions::Instruction;

pub enum OpBlock {
    NormalCode { code: Vec<Instruction> },
    FlowStop { is_kill: bool },
}

#[derive(Default)]
pub struct OpBuffer {
    buffer: Vec<Instruction>,
    blocks: Vec<OpBlock>,
}

impl OpBuffer {
    pub fn new() -> OpBuffer {
        Default::default()
    }

    pub fn push(&mut self, instr: Instruction) {
        self.buffer.push(instr);
    }

    pub fn flush(&mut self) {
        if !self.buffer.is_empty() {
            self.blocks.push(OpBlock::NormalCode {
                code: self.buffer.clone(),
            });
            self.buffer.clear();
        }
    }

    pub fn push_flow_stop(&mut self, kill: bool) {
        self.blocks.push(OpBlock::FlowStop { is_kill: kill });
    }

    pub fn finish(self) -> Vec<OpBlock> {
        self.blocks
    }
}

impl std::fmt::Debug for OpBuffer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut buf = String::new();
        for (block_i, block) in self.blocks.iter().enumerate() {
            buf.push_str(&format!("block{block_i}:\n"));
            match block {
                OpBlock::NormalCode { code } => {
                    for instruction in code {
                        buf.push_str(&format!("    {instruction:?}\n"));
                    }
                    buf.push('\n');
                }
                OpBlock::FlowStop { is_kill } => {
                    if *is_kill {
                        buf.push_str("    THREAD KILL\n");
                    } else {
                        buf.push_str("    THREAD Pause\n");
                    }
                    buf.push('\n');
                }
            }
        }
        write!(f, "{buf}")
    }
}
