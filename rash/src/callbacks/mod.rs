//! Callbacks for JIT code to call into Rust code.
//!
//! These functions are called by JIT code to perform
//! operations that are not possible in JIT code.

use core::f64;

use crate::data_types::ScratchObject;
use rand::Rng;

pub mod types;

pub extern "C" fn op_sin(value: f64) -> f64 {
    let value = value.to_radians();
    value.sin()
}

pub extern "C" fn op_cos(value: f64) -> f64 {
    let value = value.to_radians();
    value.cos()
}

pub extern "C" fn op_tan(value: f64) -> f64 {
    if (value != 0.0) && (value % 90.0 == 0.0) {
        f64::INFINITY.copysign(value)
    } else {
        let value = value.to_radians();
        value.tan()
    }
}

pub extern "C" fn op_round(value: f64) -> f64 {
    // If number ends with .5 round up (Scratch behaviour).
    if (value - value.trunc()).abs() == 0.5 {
        value.ceil()
    } else {
        value.round()
    }
}

pub extern "C" fn op_str_contains(
    string: *mut String,
    string_is_const: i64,
    substring: *mut String,
    substring_is_const: i64,
) -> i64 {
    let contains = {
        let string = unsafe { &*string }.to_lowercase();
        let substring = unsafe { &*substring }.to_lowercase();

        string.contains(&substring)
    };
    if string_is_const == 0 {
        unsafe {
            std::ptr::drop_in_place(string);
        }
    }
    if substring_is_const == 0 {
        unsafe {
            std::ptr::drop_in_place(substring);
        }
    }
    contains as i64
}

pub extern "C" fn op_str_letter(string: *mut String, is_const: i64, index: f64, out: *mut String) {
    let letter = get_char_at_index(index, string);

    if is_const == 0 {
        let mut string = unsafe { string.read() };

        // Reuse the input string, since it's gonna get dropped anyway.
        string.clear();
        if let Some(letter) = letter {
            string.push(letter);
        }
        unsafe {
            out.write(string);
        }
    } else {
        let letter = letter.map(String::from).unwrap_or_default();
        unsafe {
            out.write(letter);
        }
    }
}

fn get_char_at_index(index: f64, string: *mut String) -> Option<char> {
    if index < 1.0 {
        return None;
    }

    let index = index as usize - 1;
    let string = unsafe { &*string };
    // Scratch encodes strings in UTF-16, so we have to convert it.
    // This HAS to be done for a fully correct implementation.
    // For example, the emoji "💀" is 4 "chars" in rust string,
    // but 2 chars in UTF-16 Scratch string.
    let string_utf16 = string.encode_utf16().collect::<Vec<u16>>();
    string_utf16
        .get(index)
        // Convert the UTF-16 to a char.
        // If failed, use the unicode replacement character
        // which represents an unknown character.
        .map(|n| {
            char::from_u32(*n as u32).unwrap_or({
                // '\u{FFFD}'

                // WARNING: Highly unsafe, could cause crashes.
                // TODO: Find a better way to handle this.
                unsafe { std::mem::transmute::<u32, char>(*n as u32) }
            })
        })
}

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
    let string = unsafe { &*s };
    // Scratch stores Strings in UTF-16 (unlike rust).
    // For example, skull emoji ("💀") is 4 chars in rust,
    // but 2 chars in Scratch.
    // So a conversion is needed.
    let len = string.encode_utf16().count();
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
