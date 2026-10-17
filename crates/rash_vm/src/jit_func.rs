#![doc = include_str!("../../../docs/JIT_SIGNATURE.md")]

use cranelift::codegen::ir::Value;
use rash_core::RunState;

use crate::{
    ScratchObject,
    runtime::{JumpId, ScratchThread, SpawnableScripts},
};

pub type JitFunction = unsafe extern "C" fn(
    JumpId,
    *mut Vec<i64>, // Stack repeat
    *const ScratchObject,
    *const SpawnableScripts,
    *mut RunState,
    u8, // Is screen refresh (1/0)
    *mut Option<Box<ScratchThread>>,
) -> JumpId;

/// See [`crate::jit_func`] docs for more info.
pub struct JitArgs {
    pub jump_id: Value,
    pub loop_stack_ptr: Value,
    pub args_ptr: Value,
    pub script_ptr: Value,
    pub graphics_ptr: Value,
    pub is_called_by_pausable: Value,
    pub child_thread_ptr: Value,
}

impl JitArgs {
    pub fn new(args: &[Value]) -> Self {
        Self {
            jump_id: args[0],
            loop_stack_ptr: args[1],
            args_ptr: args[2],
            script_ptr: args[3],
            graphics_ptr: args[4],
            is_called_by_pausable: args[5],
            child_thread_ptr: args[6],
        }
    }
}
