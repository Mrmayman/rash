use crate::data_types::ScratchObject;

pub extern "C" fn op_join_string(a: *mut String, b: *mut String, out: *mut usize) {
    /*unsafe {
        println!(
            "join string: {:X}, {:X}, {:X}",
            *(a as *const i64),
            *(a as *const i64).offset(1),
            *(a as *const i64).offset(2)
        );
        println!(
            "join string: {:X}, {:X}, {:X}",
            *(b as *const i64),
            *(b as *const i64).offset(1),
            *(b as *const i64).offset(2)
        );
    }*/
    let a_ref = unsafe { &*a };
    let b_ref = unsafe { &*b };
    // println!("joining string {a_ref}, {b_ref}");
    let result = format!("{}{}", a_ref, b_ref);
    let data: [usize; 3] = unsafe { std::mem::transmute(result) };
    unsafe {
        out.write(data[0]);
        out.offset(1).write(data[1]);
        out.offset(2).write(data[2]);
    }

    // Drop a and b
    unsafe {
        std::ptr::drop_in_place(a);
        std::ptr::drop_in_place(b);
    }
}

pub extern "C" fn var_read(ptr: *const ScratchObject, dest: *mut usize) {
    let obj = unsafe { (*ptr).clone() };
    // println!("reading var - {obj}");
    let data: [usize; 4] = unsafe { std::mem::transmute(obj) };
    unsafe {
        dest.write(data[0]);
        dest.offset(1).write(data[1]);
        dest.offset(2).write(data[2]);
        dest.offset(3).write(data[3]);
    }
}
