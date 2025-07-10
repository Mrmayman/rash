pub unsafe extern "C" fn stack_push(ptr: *mut Vec<i64>, val: i64) {
    debug_assert!(!ptr.is_null());
    let vec = unsafe { &mut *ptr };
    vec.push(val);
}

pub unsafe extern "C" fn stack_pop(ptr: *mut Vec<i64>) -> i64 {
    debug_assert!(!ptr.is_null());
    let vec = unsafe { &mut *ptr };
    if let Some(num) = vec.pop() {
        num
    } else {
        panic!("repeat_stack::stack_pop : No value left in stack!")
    }
}
