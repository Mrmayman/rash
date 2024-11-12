//! Callbacks for JIT code to call into Rust code.
//!
//! These functions are called by JIT code to perform
//! operations that are not possible in JIT code.

use crate::data_types::ScratchObject;
use rand::Rng;

pub mod types;

/// Callback from JIT code to join two strings
pub extern "C" fn op_str_join(
    a: *mut String,
    b: *mut String,
    out: *mut String,
    a_is_const: i64,
    b_is_const: i64,
) {
    let a_ref = unsafe { &mut *a };
    let b_ref = unsafe { &mut *b };

    // If a isn't const, we can just append b to it.
    if a_is_const == 0 {
        a_ref.push_str(b_ref);
        unsafe {
            if b_is_const == 0 {
                std::ptr::drop_in_place(b);
            }
            out.write(a.read());
        }
        return;
    }

    // Otherwise we create a new string.
    let result = format!("{a_ref}{b_ref}");
    unsafe {
        out.write(result);

        // Drop b.
        // We know that a is const, so no need to drop a.
        if b_is_const == 0 {
            std::ptr::drop_in_place(b);
        }
    }
}

/// Callback from JIT code to get the length of a string
pub extern "C" fn op_str_len(s: *mut String, is_const: i64) -> usize {
    let len = unsafe { (*s).len() };
    if is_const == 0 {
        unsafe {
            std::ptr::drop_in_place(s);
        }
    }
    len
}

/// Callback from JIT code to read a variable.
///
/// **Why can't you just directly read from memory?**
/// Because, if it's a String, it has to be cloned otherwise
/// there may be double frees and other memory issues.
///
/// # Arguments
/// * `ptr` - The pointer to the variable (supplied from the
///   MEMORY array beforehand by the compiler).
/// * `dest` - The pointer to the destination memory location.
///   (not a pointer to `ScratchObject` for simplicity sake)
pub extern "C" fn var_read(ptr: *const ScratchObject, dest: *mut i64) {
    let obj = unsafe { (*ptr).clone() };
    let data: [i64; 4] = unsafe { std::mem::transmute(obj) };
    unsafe {
        dest.write(data[0]);
        dest.offset(1).write(data[1]);
        dest.offset(2).write(data[2]);
        dest.offset(3).write(data[3]);
    }
}

/// Callback from JIT code to generate a random number.
///
/// # Arguments
/// * `a` - The lower bound of the random number.
/// * `b` - The upper bound of the random number.
/// * `is_decimal` - Whether the number should be a decimal
///   (eg: 3.1415) or round (eg: 3.0). If `is_decimal` is 1,
///   the number will be a decimal. Represented this way for simplicity.
pub extern "C" fn op_random(a: f64, b: f64, is_decimal: i64) -> f64 {
    let mut rng = rand::thread_rng();
    let num = rng.gen_range(a..b);
    if is_decimal == 1 {
        num
    } else {
        num.round()
    }
}
