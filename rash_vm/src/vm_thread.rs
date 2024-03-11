use crate::{bytecode::bytecode_instructions::Instruction, data_types::DataValue};

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
    pub fn run(&mut self, memory: &mut [DataValue]) {
        if self.killed {
            return;
        }

        loop {
            match self.code[self.instruction_counter] {
                Instruction::MemSetToValue { ref ptr, ref value } => {
                    memory[*ptr] = value.clone();
                }
                Instruction::MemCopy {
                    ref start,
                    ref destination,
                } => {
                    memory[*destination] = memory[*start].clone();
                }
                Instruction::MathAdd {
                    ref a,
                    ref b,
                    ref result,
                } => {
                    memory[*result] =
                        DataValue::Number(memory[*a].to_number() + memory[*b].to_number());
                }
                Instruction::MathSubtract {
                    ref a,
                    ref b,
                    ref result,
                } => {
                    memory[*result] =
                        DataValue::Number(memory[*a].to_number() - memory[*b].to_number());
                }
                Instruction::MathMultiply {
                    ref a,
                    ref b,
                    ref result,
                } => {
                    memory[*result] =
                        DataValue::Number(memory[*a].to_number() * memory[*b].to_number());
                }
                Instruction::MathDivide {
                    ref a,
                    ref b,
                    ref result,
                } => {
                    memory[*result] =
                        DataValue::Number(memory[*a].to_number() / memory[*b].to_number());
                }
                Instruction::MathMod {
                    ref a,
                    ref b,
                    ref result,
                } => {
                    memory[*result] = DataValue::Number(
                        memory[*a].to_number().rem_euclid(memory[*b].to_number()),
                    );
                }
                Instruction::CompGreater {
                    ref a,
                    ref b,
                    ref result,
                } => {
                    memory[*result] =
                        DataValue::Bool(memory[*a].to_number() > memory[*b].to_number());
                }
                Instruction::CompLesser {
                    ref a,
                    ref b,
                    ref result,
                } => {
                    memory[*result] =
                        DataValue::Bool(memory[*a].to_number() < memory[*b].to_number());
                }
                Instruction::JumpDefinePoint { .. } => unreachable!(),
                Instruction::JumpToPointIfTrue { .. } => unreachable!(),
                Instruction::JumpToRawLocationIfTrue {
                    ref location,
                    ref condition,
                } => {
                    if memory[*condition].to_bool() {
                        self.instruction_counter = location - 1;
                    }
                }
                Instruction::ThreadPause => break,
                Instruction::ThreadKill => {
                    self.killed = true;
                    break;
                }
            }
            self.instruction_counter += 1;
        }
    }
}
