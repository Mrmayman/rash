pub extern "C" fn stack_push(ptr: *mut Vec<i64>, val: i64) {
    let vec = if ptr.is_null() {
        panic!("repeat_stack::stack_push : Pointer is null!")
    } else {
        unsafe { &mut *ptr }
    };
    vec.push(val);
}

pub extern "C" fn stack_pop(ptr: *mut Vec<i64>) -> i64 {
    let vec = if ptr.is_null() {
        panic!("repeat_stack::stack_pop : Pointer is null!")
    } else {
        unsafe { &mut *ptr }
    };
    if let Some(num) = vec.pop() {
        num
    } else {
        panic!("repeat_stack::stack_pop : No value left in stack!")
    }
}
