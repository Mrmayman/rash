use rand::Rng;

use crate::ScratchObject;

pub fn print_function_addresses() {
    fn print(name: &str, addr: *const ()) {
        println!("{name:35} = {:#018x}", addr as usize);
    }

    println!("\n========");
    println!("op.rs");
    println!("========");

    print("sin", sin as *const ());
    print("cos", cos as *const ());
    print("tan", tan as *const ());
    print("round", round as *const ());
    print("cmp", cmp as *const ());

    print("str_contains", str_contains as *const ());
    print("str_letter", str_letter as *const ());
    print("str_join", str_join as *const ());
    print("str_len", str_len as *const ());
    print("random", random as *const ());
}

pub extern "C" fn sin(value: f64) -> f64 {
    value.to_radians().sin()
}

pub extern "C" fn cos(value: f64) -> f64 {
    value.to_radians().cos()
}

pub extern "C" fn tan(value: f64) -> f64 {
    match (value + 90.0) % 360.0 {
        0.0 => f64::NEG_INFINITY,
        180.0 | -180.0 => f64::INFINITY,
        _ => value.to_radians().tan(),
    }
}

pub extern "C" fn round(value: f64) -> f64 {
    if (value - value.trunc()).abs() == 0.5 {
        // If number ends with .5 round up (Scratch behaviour).
        // Override of Rust behaviour
        value.ceil()
    } else {
        value.round()
    }
}

pub unsafe extern "C" fn str_contains(
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

pub unsafe extern "C" fn str_letter(
    string: *mut String,
    is_const: i64,
    index: f64,
    out: *mut String,
) {
    let letter = get_char_at_index(index, string);

    let string = if is_const == 0 {
        let mut string = unsafe { string.read() };

        // Reuse the input string, since it's gonna get dropped anyway.
        string.clear();
        if let Some(letter) = letter {
            string.push(letter);
        }
        string
    } else {
        letter.map(String::from).unwrap_or_default()
    };
    unsafe {
        out.write(string);
    }
}

/// Get character at index of a string, respecting UTF-16 behaviour
fn get_char_at_index(index: f64, string: *mut String) -> Option<char> {
    if index < 1.0 {
        return None;
    }

    let index = index as usize - 1;
    let string = unsafe { &*string };

    // Scratch encodes strings in UTF-16, so we have to convert it.
    // This HAS to be done for a fully correct implementation.
    string
        .encode_utf16()
        .nth(index)
        .map(|n| char::from_u32(n as u32).unwrap_or('\u{FFFD}'))
    // For example, the emoji "💀" is 4 "chars" in rust string,
    // but 2 chars in UTF-16 Scratch string.
}

/// Callback from JIT code to join two strings
pub unsafe extern "C" fn str_join(
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
pub unsafe extern "C" fn str_len(s: *mut String, is_const: i64) -> usize {
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

/// Callback to compare objects (Scratch-spec), returns i8-shaped value
pub unsafe extern "C" fn cmp(
    a1: i64,
    a2: i64,
    a3: i64,
    a4: i64,
    b1: i64,
    b2: i64,
    b3: i64,
    b4: i64,
) -> i64 {
    // println!("{a1}, {a2}, {a3}, {a4}");
    let a: ScratchObject = unsafe { std::mem::transmute([a1, a2, a3, a4]) };
    // println!("{b1}, {b2}, {b3}, {b4}");
    let b: ScratchObject = unsafe { std::mem::transmute([b1, b2, b3, b4]) };
    // println!("{a:?}, {b:?}");
    let r = a.scratch_cmp(&b) as i64;
    // TODO: there's a memory lifetime bug in b
    // This will be fixed when we migrate to SmolStr
    std::mem::forget(b);
    r
}

/// Callback from JIT code to generate a random number.
///
/// # Arguments
/// * `a` - The lower bound of the random number.
/// * `b` - The upper bound of the random number.
/// * `is_decimal` - Whether the number should be a decimal
///   (eg: 3.1415) or round (eg: 3.0). If `is_decimal` is 1,
///   the number will be a decimal. Represented this way for simplicity.
pub extern "C" fn random(a: f64, b: f64, is_decimal: i64) -> f64 {
    let mut rng = rand::thread_rng();
    let num = rng.gen_range(a..b);
    if is_decimal == 1 { num } else { num.round() }
}
