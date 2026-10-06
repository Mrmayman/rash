use rash_core::{RunState, SpriteId};

declare_module!(
    "motion.rs",
    7,
    c_go_to,
    c_set_x,
    c_set_y,
    c_get_x,
    c_get_y,
    c_change_x,
    c_change_y,
);

pub unsafe extern "C" fn c_go_to(this: *mut RunState, id: SpriteId, x: f64, y: f64) {
    debug_assert!(!this.is_null());
    (unsafe { &mut *this }).go_to(id, x as f32, y as f32);
}

pub unsafe extern "C" fn c_set_x(this: *mut RunState, id: SpriteId, x: f64) {
    debug_assert!(!this.is_null());
    (unsafe { &mut *this }).set_x(id, x as f32);
}

pub unsafe extern "C" fn c_set_y(this: *mut RunState, id: SpriteId, y: f64) {
    debug_assert!(!this.is_null());
    (unsafe { &mut *this }).set_y(id, y as f32);
}

pub unsafe extern "C" fn c_get_x(this: *mut RunState, id: SpriteId) -> f64 {
    debug_assert!(!this.is_null());
    f64::from((unsafe { &mut *this }).get_x(id))
}

pub unsafe extern "C" fn c_get_y(this: *mut RunState, id: SpriteId) -> f64 {
    debug_assert!(!this.is_null());
    f64::from((unsafe { &mut *this }).get_y(id))
}

pub unsafe extern "C" fn c_change_x(this: *mut RunState, id: SpriteId, x: f64) {
    debug_assert!(!this.is_null());
    (unsafe { &mut *this }).change_x(id, x as f32);
}

pub unsafe extern "C" fn c_change_y(this: *mut RunState, id: SpriteId, y: f64) {
    debug_assert!(!this.is_null());
    (unsafe { &mut *this }).change_y(id, y as f32);
}
