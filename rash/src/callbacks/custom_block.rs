use rash_render::RunState;

use crate::{
    data_types::ScratchObject,
    scheduler::{CustomBlockId, ScratchThread, Scripts},
};

#[repr(i64)]
pub enum PauseStatus {
    Ended = 0,
    Paused = 1,
}

pub unsafe extern "C" fn call_no_screen_refresh(
    arg_buffer: *const ScratchObject,
    id: i64,
    scripts: *const Scripts,
    graphics: *mut RunState,
) {
    debug_assert!(!arg_buffer.is_null());
    debug_assert!(!scripts.is_null());

    let scripts = unsafe { &*scripts };
    // println!("calling custom block: {id}");
    let id = CustomBlockId(id as usize);

    let Some(script) = scripts.custom_blocks.get(&id) else {
        panic!(
            "custom_block::call_no_screen_refresh : No custom block found with id {}",
            id.0
        )
    };

    let args = unsafe { vec_from_raw(arg_buffer, script.num_args) };

    let mut script = script.thread.spawn(false, args);
    while !unsafe { script.tick(scripts, &mut *graphics) } {}
}

pub unsafe extern "C" fn call_screen_refresh(
    arg_buffer: *const ScratchObject,
    id: i64,
    scripts: *const Scripts,
    graphics: *mut RunState,
    child_thread: *mut Option<ScratchThread>,
    parent_is_screen_refresh: i64,
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

    let Some(script) = scripts.custom_blocks.get(&id) else {
        panic!(
            "custom_block::call_no_screen_refresh : No custom block found with id {}",
            id.0
        )
    };

    let is_screen_refresh = parent_is_screen_refresh == 1 && script.is_screen_refresh;

    let args = unsafe { vec_from_raw(arg_buffer, script.num_args) };
    let mut script = script.thread.spawn(is_screen_refresh, args);

    if is_screen_refresh {
        let ended = unsafe { script.tick(scripts, &mut *graphics) };

        // The child thread has paused
        if !ended {
            // Save the execution context for later resuming it
            unsafe { *child_thread = Some(script) }
            return PauseStatus::Paused;
        }
    } else {
        while !unsafe { script.tick(scripts, &mut *graphics) } {}
    }
    PauseStatus::Ended
}

unsafe fn vec_from_raw<T: Clone>(ptr: *const T, count: usize) -> Vec<T> {
    debug_assert!(!ptr.is_null());
    let mut vec = Vec::with_capacity(count);

    for i in 0..count {
        let item = ptr.add(i);
        vec.push((*item).clone());
    }

    vec
}
