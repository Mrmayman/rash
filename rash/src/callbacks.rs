use crate::data_types::{ScratchObject, ScratchObjectBytes};

pub extern "C" fn op_join_string(a1: i64, a2: i64, a3: i64, b1: i64, b2: i64, b3: i64) -> [i64; 3] {
    let a: String = unsafe { std::mem::transmute([a1, a2, a3]) };
    let b: String = unsafe { std::mem::transmute([b1, b2, b3]) };

    let c = a + &b;
    unsafe { std::mem::transmute(c) }
}

pub extern "C" fn var_read(ptr: *const ScratchObject, dest: *mut i64) {
    let obj = unsafe { (*ptr).clone() };
    println!("reading var - {obj}");
    let data: [i64; 4] = unsafe { std::mem::transmute(obj) };
    unsafe {
        dest.write(data[0]);
        dest.offset(1).write(data[1]);
        dest.offset(2).write(data[2]);
        dest.offset(3).write(data[3]);
    }
}
