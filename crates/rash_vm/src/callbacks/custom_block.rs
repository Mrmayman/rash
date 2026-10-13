use crate::{
    data_types::ScratchObject,
    runtime::{CustomBlockId, JumpId, MaybeCompiled, ScratchThread, SpawnableScripts},
};
use rash_core::RunState;

declare_module!(
    "custom_block.rs",
    1,
    call_no_screen_refresh,
    call_screen_refresh
);

#[repr(i64)]
pub enum PauseStatus {
    Ended = 0,
    Paused = 1,
}

/// Fast path, functions that don't pause (no screen refresh)
pub unsafe extern "C" fn call_no_screen_refresh(
    arg_buffer: *mut ScratchObject,
    id: i64,
    scripts: *const SpawnableScripts,
    graphics: *mut RunState,
) {
    debug_assert!(!arg_buffer.is_null());
    debug_assert!(!scripts.is_null());

    let scripts = unsafe { &*scripts };
    // println!("calling custom block: {id}");
    let id = CustomBlockId(id as usize);

    let Some(script) = scripts.custom_blocks.get(id.0) else {
        panic!("No custom block found with id {}", id.0)
    };

    let MaybeCompiled::Compiled(s) = &script.script else {
        panic!("Custom block {} hasn't been compiled yet", id.0)
    };

    let mut repeat_stack = Vec::new();
    let mut child_thread: Option<Box<ScratchThread>> = None;
    let mut result = JumpId::default();
    // TODO: Warp timer
    // Poorly written code could loop infinitely and hang the runtime.
    // If a check is added, ensure it is optional (can be disabled for performance)
    while !result.is_done() {
        if let Some(child) = &mut child_thread {
            while !unsafe { child.tick(scripts, &mut *graphics) } {}
            child_thread = None;
        }

        result = unsafe {
            (s.func)(
                result,
                &raw mut repeat_stack,
                arg_buffer,
                scripts,
                graphics,
                0,
                &raw mut child_thread,
            )
        };
    }

    for offset in 0..script.num_args {
        let arg = unsafe { arg_buffer.add(offset) };
        unsafe { std::ptr::drop_in_place(arg) };
    }
}

/// Slow path, functions that can pause (screen refresh)
/// AKA sort-of coroutines.
pub unsafe extern "C" fn call_screen_refresh(
    arg_buffer: *const ScratchObject,
    id: i64,
    scripts: *const SpawnableScripts,
    graphics: *mut RunState,
    child_thread: *mut Option<Box<ScratchThread>>,
    parent_is_screen_refresh: u8,
) -> PauseStatus {
    debug_assert!(!arg_buffer.is_null());
    debug_assert!(!scripts.is_null());
    debug_assert!(!child_thread.is_null());
    unsafe {
        debug_assert!((*child_thread).is_none());
    }

    let scripts = unsafe { &*scripts };
    // println!("calling custom block: {id}");
    let id = CustomBlockId(id as usize);

    let Some(script) = scripts.custom_blocks.get(id.0) else {
        panic!("No custom block found with id {}", id.0)
    };

    let is_screen_refresh = parent_is_screen_refresh == 1 && script.is_screen_refresh;

    let args = unsafe { move_into_new_vec(arg_buffer, script.num_args) };

    let MaybeCompiled::Compiled(script) = &script.script else {
        panic!("Custom block {} hasn't been compiled yet", id.0)
    };
    let mut script = script.spawn(is_screen_refresh, args);

    if is_screen_refresh {
        let ended = unsafe { script.tick(scripts, &mut *graphics) };

        // Child thread paused; save execution context so we can resume it later
        if !ended {
            unsafe { *child_thread = Some(Box::new(script)) }
            return PauseStatus::Paused;
        }
    } else {
        while !unsafe { script.tick(scripts, &mut *graphics) } {}
    }
    PauseStatus::Ended
}

unsafe fn move_into_new_vec<T: Clone>(ptr: *const T, count: usize) -> Vec<T> {
    debug_assert!(!ptr.is_null());
    let mut vec = Vec::with_capacity(count);

    for i in 0..count {
        let item = unsafe { ptr.add(i) };
        vec.push(unsafe { item.read().clone() });
    }

    vec
}
