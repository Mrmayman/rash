/// Definitions for the bytecode instructions,
/// in the `Instruction` enum
pub mod bytecode;
/// The Scratch data types, in a `ScratchObject` enum.
pub mod data_types;
/// The bytecode interpreter that actually runs the code.
pub mod vm_thread;

#[cfg(feature = "jit")]
pub mod jit;
