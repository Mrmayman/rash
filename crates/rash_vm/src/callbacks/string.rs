use smol_str::{SmolStr, SmolStrBuilder, StrExt};

declare_module!("string.rs", 5, contains, letter, join, len);

pub unsafe extern "C" fn contains(string: *mut SmolStr, substring: *mut SmolStr) -> i64 {
    let contains = {
        let string = unsafe { &*string }.to_lowercase_smolstr();
        let substring = unsafe { &*substring }.to_lowercase_smolstr();

        string.contains(&*substring)
    };
    unsafe {
        std::ptr::drop_in_place(string);
        std::ptr::drop_in_place(substring);
    }
    i64::from(contains)
}

pub unsafe extern "C" fn letter(string: *mut SmolStr, index: f64, out: *mut SmolStr) {
    let string = unsafe { string.read() };
    let letter = get_char_at_index(index, &string);

    let mut output = SmolStrBuilder::new();
    if let Some(letter) = letter {
        output.push(letter);
    }

    unsafe {
        out.write(output.finish());
    }
}

/// Get character at index of a string, respecting UTF-16 behaviour
fn get_char_at_index(index: f64, string: &str) -> Option<char> {
    if index < 1.0 {
        return None;
    }

    let index = index as usize - 1;

    // Scratch encodes strings in UTF-16, so we have to convert it.
    // This HAS to be done for a fully correct implementation.
    string
        .encode_utf16()
        .nth(index)
        .map(|n| char::from_u32(u32::from(n)).unwrap_or('\u{FFFD}'))
    // For example, the emoji "💀" is 4 "chars" in rust string,
    // but 2 chars in UTF-16 Scratch string.
}

/// Callback from JIT code to join two strings
pub unsafe extern "C" fn join(a: *mut SmolStr, b: *mut SmolStr, out: *mut SmolStr) {
    let a_ref = unsafe { &*a };
    let b_ref = unsafe { &*b };

    let mut result = SmolStrBuilder::new();
    result.push_str(a_ref);
    result.push_str(b_ref);
    unsafe {
        out.write(result.finish());

        std::ptr::drop_in_place(a);
        std::ptr::drop_in_place(b);
    }
}

/// Callback from JIT code to get the length of a string
pub unsafe extern "C" fn len(s: *mut SmolStr) -> usize {
    let string = unsafe { &*s };
    // Scratch stores Strings in UTF-16 (unlike rust).
    // For example, skull emoji ("💀") is 4 chars in rust,
    // but 2 chars in Scratch.
    // So a conversion is needed.
    let len = string.encode_utf16().count();
    unsafe {
        std::ptr::drop_in_place(s);
    }
    len
}
