use std::fmt::Display;

use colored::Colorize;

use crate::data_types::ScratchObject;

/// A struct representing a point in the code to jump or goto to.
/// Think of it like C goto.
/// Only used for the compiling stage, as it is slow to run.
///
/// It is replaced with raw jumps that directly change the
/// program counter, when `thread.flatten_places()` is called.
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Debug, Hash)]
pub struct JumpPoint(pub usize);

/// A pointer to data.
/// It's just an index to the array of VM memory.
#[derive(Clone, Debug)]
pub struct DataPointer(pub usize);

impl Display for DataPointer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}{}",
            "*".purple(),
            self.0.to_string().bright_purple().bold()
        )
    }
}

/// The bytecode instructions.
/// This is what the VM runs.
///
/// # Notes about branching
/// * Raw Location is directly the index of the instruction in the array of bytecode.
/// * Whereas with Places, you define a place and give it an id, and jump to a place with that id. Think of it like C goto.
/// * So the VM will have to search where that place is defined.
/// * Places are easier to compile but slow to run, so we convert them to raw locations at the last stage.
#[derive(Clone, Debug)]
pub enum Instruction {
    MemSetToValue {
        ptr: DataPointer,
        value: ScratchObject,
    },
    MathAdd {
        a: ScratchObject,
        b: ScratchObject,
        result: DataPointer,
    },
    MathUncheckedAdd {
        a: ScratchObject,
        b: ScratchObject,
        result: DataPointer,
    },
    MathSubtract {
        a: ScratchObject,
        b: ScratchObject,
        result: DataPointer,
    },
    MathUncheckedSubtract {
        a: ScratchObject,
        b: ScratchObject,
        result: DataPointer,
    },
    MathMultiply {
        a: ScratchObject,
        b: ScratchObject,
        result: DataPointer,
    },
    MathUncheckedMultiply {
        a: ScratchObject,
        b: ScratchObject,
        result: DataPointer,
    },
    MathDivide {
        a: ScratchObject,
        b: ScratchObject,
        result: DataPointer,
    },
    MathUncheckedDivide {
        a: ScratchObject,
        b: ScratchObject,
        result: DataPointer,
    },
    MathMod {
        a: ScratchObject,
        b: ScratchObject,
        result: DataPointer,
    },
    MathUncheckedMod {
        a: ScratchObject,
        b: ScratchObject,
        result: DataPointer,
    },
    CompGreater {
        a: ScratchObject,
        b: ScratchObject,
        result: DataPointer,
    },
    CompLesser {
        a: ScratchObject,
        b: ScratchObject,
        result: DataPointer,
    },
    JumpDefinePoint {
        place: JumpPoint,
    },
    JumpToPointIfTrue {
        place: JumpPoint,
        condition: ScratchObject,
    },
    JumpToRawLocationIfTrue {
        location: usize,
        condition: ScratchObject,
    },
    ThreadPause,
    ThreadKill,
}

impl Display for Instruction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Instruction::MemSetToValue { ptr, value } => {
                write!(f, "{ptr} = {value}")
            }
            Instruction::MathAdd { a, b, result } => {
                write!(f, "{result} = {a} {} {b}", "+".red().bold())
            }
            Instruction::MathUncheckedAdd { a, b, result } => {
                write!(f, "{result} = {a} {}{} {b}", "?".dimmed(), "+".red().bold())
            }
            Instruction::MathSubtract { a, b, result } => {
                write!(f, "{result} = {a} {} {b}", "-".red().bold())
            }
            Instruction::MathUncheckedSubtract { a, b, result } => {
                write!(f, "{result} = {a} {}{} {b}", "?".dimmed(), "-".red().bold())
            }
            Instruction::MathMultiply { a, b, result } => write!(f, "{result} = {a} * {b}"),
            Instruction::MathUncheckedMultiply { a, b, result } => {
                write!(f, "{result} = {a} {}{} {b}", "?".dimmed(), "*".red().bold())
            }
            Instruction::MathDivide { a, b, result } => write!(f, "{result} = {a} / {b}"),
            Instruction::MathUncheckedDivide { a, b, result } => {
                write!(f, "{result} = {a} {}{} {b}", "?".dimmed(), "/".red().bold())
            }
            Instruction::MathMod { a, b, result } => write!(f, "{result} = {a} % {b}"),
            Instruction::MathUncheckedMod { a, b, result } => {
                write!(f, "{result} = {a} {}{} {b}", "?".dimmed(), "%".red().bold())
            }
            Instruction::CompGreater { a, b, result } => write!(f, "{result} = {a} > {b}"),
            Instruction::CompLesser { a, b, result } => write!(f, "{result} = {a} < {b}"),
            Instruction::JumpDefinePoint { place } => write!(
                f,
                "{}{}:",
                "BLOCK_".underline().bold(),
                place.0.to_string().bold().underline()
            ),
            Instruction::JumpToPointIfTrue { place, condition } => {
                write!(
                    f,
                    "if {condition} goto {}{}",
                    "BLOCK_".bold().underline(),
                    place.0.to_string().underline().bold()
                )
            }
            Instruction::JumpToRawLocationIfTrue {
                location,
                condition,
            } => write!(f, "if {condition} goto (*{location})"),
            Instruction::ThreadPause => write!(f, "SCREEN_REFRESH"),
            Instruction::ThreadKill => write!(f, "THREAD_END"),
        }
    }
}
