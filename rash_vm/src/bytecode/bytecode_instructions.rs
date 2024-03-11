use crate::data_types::DataValue;

/// A struct representing a point in the code to jump or goto to.
/// Think of it like C goto.
/// Mostly used for the compiling stage, as it is slow to run.
pub type JumpPoint = usize;

/// A pointer to data.
/// It's just an index to the array of VM memory.
// pub struct DataPointer(pub usize);
pub type DataPointer = usize;

/// The bytecode instructions.
/// This is what the VM runs.
///
/// # Notes about branching
/// * Raw Location is directly the index of the instruction in the array of bytecode.
/// * Whereas with Places, you define a place and give it an id, and jump to a place with that id. Think of it like C goto.
/// * So the VM will have to search where that place is defined.
/// * Places are easier to compile but slow to run, so we convert them to raw locations at the last stage.
pub enum Instruction {
    MemSetToValue {
        ptr: DataPointer,
        value: DataValue,
    },
    MemCopy {
        start: DataPointer,
        destination: DataPointer,
    },
    MathAdd {
        a: DataPointer,
        b: DataPointer,
        result: DataPointer,
    },
    MathSubtract {
        a: DataPointer,
        b: DataPointer,
        result: DataPointer,
    },
    MathMultiply {
        a: DataPointer,
        b: DataPointer,
        result: DataPointer,
    },
    MathDivide {
        a: DataPointer,
        b: DataPointer,
        result: DataPointer,
    },
    MathMod {
        a: DataPointer,
        b: DataPointer,
        result: DataPointer,
    },
    CompGreater {
        a: DataPointer,
        b: DataPointer,
        result: DataPointer,
    },
    CompLesser {
        a: DataPointer,
        b: DataPointer,
        result: DataPointer,
    },
    JumpDefinePoint {
        place: JumpPoint,
    },
    JumpToPointIfTrue {
        place: JumpPoint,
        condition: DataPointer,
    },
    JumpToRawLocationIfTrue {
        location: usize,
        condition: DataPointer,
    },
    ThreadPause,
    ThreadKill,
}
