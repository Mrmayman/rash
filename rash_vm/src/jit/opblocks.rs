use std::collections::HashMap;

use colored::Colorize;

use crate::bytecode::{Instruction, JumpPoint};

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

    pub fn len(&self) -> usize {
        self.blocks.len()
    }

    pub fn finish(self) -> Vec<OpBlock> {
        self.blocks
    }

    /// By default, [`Instruction::JumpToPointIfTrue`] jumps to the code-block based
    /// on the internal id given by the interpreter.
    /// However we need to change this to an ID that cranelift is happy with.
    pub fn fix_jumps(&mut self, block_lookup: &HashMap<JumpPoint, Option<usize>>) {
        // If an error happens, we can't borrow self to print it because
        // we are mutable looping over self.blocks.
        // So we get the debug print value beforehand.
        let debug_print = format!("{self:?}");

        self.blocks
            // 1) Loop over all the code-blocks mutably.
            .iter_mut()
            // 2) We only need the normal blocks, not the special ones
            // that mark a thread pause/end.
            .filter_map(|block| {
                if let OpBlock::NormalCode { code } = block {
                    Some(code)
                } else {
                    None
                }
            })
            // 3) Go through every normal code block
            .for_each(|code| {
                // a) For every instruction in a code block...
                code.iter_mut()
                    // b) Filter out the Instruction::JumpToPointIfTrue as
                    //    that's what we want to fix.
                    .filter_map(|instruction| {
                        if let Instruction::JumpToPointIfTrue { place, .. } = instruction {
                            Some(place)
                        } else {
                            None
                        }
                    })
                    // c) For every jump point taken from Instruction::JumpToPointIfTrue...
                    .for_each(|point| match block_lookup.get(point) {
                        // d) Get the cranelift-compatible code-block id from the HashMap.
                        Some(Some(n)) => *point = JumpPoint(*n),
                        _ => {
                            // TODO: Better error handling.
                            eprintln!("[error] JIT: jump block not defined\n\n{debug_print}")
                        }
                    });
            });
    }
}

impl std::fmt::Debug for OpBuffer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut buf = String::new();
        for (block_i, block) in self.blocks.iter().enumerate() {
            buf.push_str(&format!(
                "{}{}:\n",
                "block".blue(),
                block_i.to_string().bright_blue().bold()
            ));
            match block {
                OpBlock::NormalCode { code } => {
                    for instruction in code {
                        buf.push_str(&format!("    {instruction}\n"));
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
