use crate::{
    data_types::ScratchObject,
    scheduler::{CustomBlockId, Scheduler},
};

pub fn call_no_screen_refresh(
    arg_buffer: *const ScratchObject,
    id: i64,
    scheduler: *mut Scheduler,
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

    let mut script = script.thread.spawn(Some(arg_buffer));
    while !script.tick(scheduler) {}
}
