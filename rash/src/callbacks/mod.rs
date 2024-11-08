use crate::data_types::ScratchObject;

pub mod types;

/// Callback from JIT code to join two strings
pub extern "C" fn op_str_join(
    a: *mut String,
    b: *mut String,
    out: *mut usize,
    a_is_const: i64,
    b_is_const: i64,
) {
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
    let a_ref = unsafe { &mut *a };
    let b_ref = unsafe { &mut *b };

    if a_is_const == 0 {
        a_ref.push_str(b_ref);
        if b_is_const == 0 {
            unsafe { std::ptr::drop_in_place(b) };
        }

        unsafe {
            out.write((a as *const usize).read());
            out.offset(1).write((a as *const usize).offset(1).read());
            out.offset(2).write((a as *const usize).offset(2).read());
        }
        return;
    }

    let result = format!("{}{}", a_ref, b_ref);
    let data: [usize; 3] = unsafe { std::mem::transmute(result) };
    unsafe {
        out.write(data[0]);
        out.offset(1).write(data[1]);
        out.offset(2).write(data[2]);
    }

    // Drop a and b
    unsafe {
        if a_is_const == 0 {
            std::ptr::drop_in_place(a);
        }
        if b_is_const == 0 {
            std::ptr::drop_in_place(b);
        }
    }
}

pub extern "C" fn op_str_len(s: *mut String, is_const: i64) -> usize {
    let len = unsafe { (*s).len() };
    if is_const == 0 {
        unsafe {
            std::ptr::drop_in_place(s);
        }
    }
    len
}

pub extern "C" fn var_read(ptr: *const ScratchObject, dest: *mut i64) {
    let obj = unsafe { (*ptr).clone() };
    // println!("reading var - {obj}");
    let data: [i64; 4] = unsafe { std::mem::transmute(obj) };
    unsafe {
        dest.write(data[0]);
        dest.offset(1).write(data[1]);
        dest.offset(2).write(data[2]);
        dest.offset(3).write(data[3]);
    }
}
