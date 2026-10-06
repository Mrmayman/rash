use rash_core::{RunState, SpriteId};

declare_module!("looks.rs", 8, c_shown);

pub unsafe extern "C" fn c_shown(this: *mut RunState, id: SpriteId, shown: i64) {
    debug_assert!(!this.is_null());
    unsafe { &mut *this }.shown(id, shown == 1);
}
