use crate::data_types::ScratchObject;

/// A struct representing a point in the code to jump or goto to.
/// Think of it like C goto.
/// Mostly used for the compiling stage, as it is slow to run.
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub struct JumpPoint(pub usize);

/// A pointer to data.
/// It's just an index to the array of VM memory.
#[derive(Clone, Debug)]
pub struct DataPointer(pub usize);

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
    MathSubtract {
        a: ScratchObject,
        b: ScratchObject,
        result: DataPointer,
    },
    MathMultiply {
        a: ScratchObject,
        b: ScratchObject,
        result: DataPointer,
    },
    MathDivide {
        a: ScratchObject,
        b: ScratchObject,
        result: DataPointer,
    },
    MathMod {
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
