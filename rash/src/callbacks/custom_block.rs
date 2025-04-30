use rash_render::RunState;

use crate::{
    data_types::ScratchObject,
    scheduler::{CustomBlockId, Scheduler, ScratchThread},
};

pub fn call_no_screen_refresh(
    arg_buffer: *const ScratchObject,
    id: i64,
    scheduler: *mut Scheduler,
    graphics: *mut RunState,
) {
    debug_assert!(!arg_buffer.is_null());
    debug_assert!(!scheduler.is_null());

    let scheduler = unsafe { &mut *scheduler };
    // println!("calling custom block: {id}");
    let id = CustomBlockId(id as usize);

    let Some(script) = scheduler.scripts.custom_blocks.get(&id) else {
        panic!(
            "custom_block::call_no_screen_refresh : No custom block found with id {}",
            id.0
        )
    };

    let args = unsafe { vec_from_raw(arg_buffer, script.num_args) };

    let mut script = script.thread.spawn(false, args);
    while !script.tick(scheduler, graphics) {}
}

pub fn call_screen_refresh(
    arg_buffer: *const ScratchObject,
    id: i64,
    scheduler: *mut Scheduler,
    graphics: *mut RunState,
    child_thread: *mut Option<ScratchThread>,
    parent_is_screen_refresh: i64,
) -> i64 {
    debug_assert!(!arg_buffer.is_null());
    debug_assert!(!scheduler.is_null());
    debug_assert!(!child_thread.is_null());
    unsafe {
        debug_assert!((*child_thread).is_none());
    }

    let scheduler = unsafe { &mut *scheduler };
    // println!("calling custom block: {id}");
    let id = CustomBlockId(id as usize);

    let Some(script) = scheduler.scripts.custom_blocks.get(&id) else {
        panic!(
            "custom_block::call_no_screen_refresh : No custom block found with id {}",
            id.0
        )
    };

    let is_screen_refresh = parent_is_screen_refresh == 1 && script.is_screen_refresh;

    let args = unsafe { vec_from_raw(arg_buffer, script.num_args) };
    let mut script = script.thread.spawn(is_screen_refresh, args);

    let ended = script.tick(scheduler, graphics);

    if ended {
        0
    } else {
        unsafe { *child_thread = Some(script) }
        1
    }
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
