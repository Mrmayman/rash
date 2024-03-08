use crate::data_types::DataValue;

/// A struct representing a point in the code to jump or goto to.
/// Think of it like C goto.
/// Mostly used for the compiling stage, as it is slow to run.
pub struct JumpPoint(pub usize);

/// A pointer to data.
/// It's just an index to the array of VM memory.
pub struct DataPointer(pub usize);

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
        ptr: DataPointer,
        location: DataPointer,
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
    /// Think of this like defining a goto point in C.
    /// ```
    /// place:
    ///     // Your code here
    /// ```
    JumpDefinePoint {
        place: JumpPoint,
    },
    /// And think of this like goto place in C.
    /// ```
    ///     goto place;
    /// ```
    JumpToPointIfTrue {
        place: JumpPoint,
        condition: DataPointer,
    },
    /// See note about branching to know the difference between raw location and Point.
    JumpToRawLocationIfTrue {
        location: usize,
        condition: DataPointer,
    },
}
