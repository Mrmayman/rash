use std::collections::BTreeMap;

use crate::{
    bytecode::instructions::{Instruction, JumpPoint},
    data_types::ScratchObject,
};

/// A code thread in the VM.
///
/// Will run until thread is paused (screen refresh)
/// or killed (end of script).
///
/// `Thread.killed` will be set to true if the code
/// has finished running so it has to be checked.
pub struct Thread {
    pub code: Box<[Instruction]>,
    pub instruction_counter: usize,
    pub killed: bool,
}

impl Thread {
    /// Creates a new VM thread.
    ///
    /// Code will begin running at the first instruction
    /// so any functions must be below the main procedure.
    pub fn new(code: Box<[Instruction]>) -> Self {
        Self {
            code,
            instruction_counter: 0,
            killed: false,
        }
    }

    /// Runs the code until the thread has paused
    /// (screen refresh) or it has finished running.
    pub fn run(&mut self, memory: &mut [ScratchObject]) {
        if self.killed {
            return;
        }

        loop {
            match self.code[self.instruction_counter] {
                Instruction::MemSetToValue { ref ptr, ref value } => {
                    // println!("{ptr} = {value:?}");
                    memory[ptr.0] = value.clone();
                }
                Instruction::MathAdd {
                    ref a,
                    ref b,
                    ref result,
                } => {
                    // println!("{result} = {a:?} + {b:?}");
                    memory[result.0] =
                        ScratchObject::Number(a.to_number(memory) + b.to_number(memory));
                }
                Instruction::MathSubtract {
                    ref a,
                    ref b,
                    ref result,
                } => {
                    // println!("{result} = {a:?} - {b:?}");
                    memory[result.0] =
                        ScratchObject::Number(a.to_number(memory) - b.to_number(memory));
                }
                Instruction::MathMultiply {
                    ref a,
                    ref b,
                    ref result,
                } => {
                    // println!("{result} = {a:?} * {b:?}");
                    memory[result.0] =
                        ScratchObject::Number(a.to_number(memory) * b.to_number(memory));
                }
                Instruction::MathDivide {
                    ref a,
                    ref b,
                    ref result,
                } => {
                    // println!("{result} = {a:?} / {b:?}");
                    memory[result.0] =
                        ScratchObject::Number(a.to_number(memory) / b.to_number(memory));
                }
                Instruction::MathMod {
                    ref a,
                    ref b,
                    ref result,
                } => {
                    // println!("{result} = {a:?} % {b:?}");
                    memory[result.0] =
                        ScratchObject::Number(a.to_number(memory).rem_euclid(b.to_number(memory)));
                }
                Instruction::CompGreater {
                    ref a,
                    ref b,
                    ref result,
                } => {
                    memory[result.0] =
                        ScratchObject::Bool(a.to_number(memory) > b.to_number(memory));
                }
                Instruction::CompLesser {
                    ref a,
                    ref b,
                    ref result,
                } => {
                    memory[result.0] =
                        ScratchObject::Bool(a.to_number(memory) < b.to_number(memory));
                }
                Instruction::JumpDefinePoint { .. } => unreachable!(),
                Instruction::JumpToPointIfTrue { .. } => unreachable!(),
                Instruction::JumpToRawLocationIfTrue {
                    ref location,
                    ref condition,
                } => {
                    if condition.to_bool(memory) {
                        self.instruction_counter = location - 1;
                    }
                }
                Instruction::ThreadPause => break,
                Instruction::ThreadKill => {
                    self.killed = true;
                    break;
                }
                Instruction::MathUncheckedAdd {
                    ref a,
                    ref b,
                    ref result,
                } => unsafe {
                    memory[result.0] = ScratchObject::Number(
                        a.to_number_unchecked(memory) + b.to_number_unchecked(memory),
                    );
                },
                Instruction::MathUncheckedSubtract {
                    ref a,
                    ref b,
                    ref result,
                } => unsafe {
                    memory[result.0] = ScratchObject::Number(
                        a.to_number_unchecked(memory) - b.to_number_unchecked(memory),
                    );
                },
                Instruction::MathUncheckedMultiply {
                    ref a,
                    ref b,
                    ref result,
                } => unsafe {
                    memory[result.0] = ScratchObject::Number(
                        a.to_number_unchecked(memory) * b.to_number_unchecked(memory),
                    );
                },
                Instruction::MathUncheckedDivide {
                    ref a,
                    ref b,
                    ref result,
                } => unsafe {
                    memory[result.0] = ScratchObject::Number(
                        a.to_number_unchecked(memory) / b.to_number_unchecked(memory),
                    );
                },
                Instruction::MathUncheckedMod {
                    ref a,
                    ref b,
                    ref result,
                } => unsafe {
                    memory[result.0] = ScratchObject::Number(
                        a.to_number_unchecked(memory)
                            .rem_euclid(b.to_number_unchecked(memory)),
                    );
                },
            }
            self.instruction_counter += 1;
        }
    }

    pub fn optimize(&mut self, memory: &mut [ScratchObject]) {
        #[cfg(feature = "jit")]
        self.jit(memory.as_ptr());

        self.flatten_places()
    }

    fn flatten_places(&mut self) {
        // Create a map to store jump points and their corresponding indices
        let places: BTreeMap<JumpPoint, usize> = self
            .code
            .iter()
            .enumerate()
            .filter_map(|(n, instruction)| {
                if let Instruction::JumpDefinePoint { place } = instruction {
                    Some((place.clone(), n))
                } else {
                    None
                }
            })
            .collect();

        // Filter out JumpDefinePoint instructions and replace JumpToPointIfTrue instructions with JumpToRawLocationIfTrue
        let new_code: Vec<Instruction> = self
            .code
            .iter()
            .filter_map(|instruction| match instruction {
                Instruction::JumpDefinePoint { .. } => None,
                Instruction::JumpToPointIfTrue { place, condition } => {
                    Some(Instruction::JumpToRawLocationIfTrue {
                        location: places[place],
                        condition: condition.clone(),
                    })
                }
                _ => Some(instruction.clone()),
            })
            .collect();

        // Update code with the new flattened instructions
        self.code = new_code.into_boxed_slice();
    }
}
