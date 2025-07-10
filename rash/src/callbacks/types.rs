use crate::data_types::ScratchObject;

/// Converts a `ScratchObject` to a boolean.
///
/// # Arguments
/// - `i1, i2, i3, i4` - The `ScratchObject` to convert
///   (represented this way for a predictable layout in JIT code,
///   can be `std::mem::transmute`d).
///
/// # Return
/// - `i64` - The boolean value of the `ScratchObject`
///   (represented as `i64` for predictable layout).
pub unsafe extern "C" fn to_bool(i1: i64, i2: i64, i3: i64, i4: i64) -> i64 {
    let i1 = (i1 as i32) as i64;
    #[cfg(debug_assertions)]
    {
        if !(0..4).contains(&i1) {
            eprintln!("error converting to bool - enum id: {i1}");
            eprintln!("hex: {i1:X} {i2:X} {i3:X} {i4:X}");
        }
    }
    debug_assert!(i1 < 4);
    debug_assert!(i1 >= 0);
    let obj: ScratchObject = unsafe { std::mem::transmute([i1, i2, i3, i4]) };
    i64::from(obj.convert_to_bool())
}

/// Converts a `ScratchObject` to a number.
/// If the object is a string, it will attempt to parse it as a number.
/// If the object is a boolean, it will return 1 for true and 0 for false.
///
/// # Arguments
/// - `i1, i2, i3, i4` - The `ScratchObject` to convert
///   (represented this way for a predictable layout in JIT code,
///   can be `std::mem::transmute`d).
///
/// # Return
/// - `f64` - The number value of the `ScratchObject`
pub unsafe extern "C" fn to_number(i1: i64, i2: i64, i3: i64, i4: i64) -> f64 {
    let i1 = (i1 as i32) as i64;
    #[cfg(debug_assertions)]
    {
        if !(0..4).contains(&i1) {
            eprintln!("error converting to number - enum id: {i1}");
            eprintln!("hex: {i1:X} {i2:X} {i3:X} {i4:X}");
        }
    }
    debug_assert!(i1 < 4);
    debug_assert!(i1 >= 0);
    let obj: ScratchObject = unsafe { std::mem::transmute([i1, i2, i3, i4]) };
    obj.convert_to_number()
}

pub struct DecimalCheck {
    _number: f64,
    _decimal: i64,
}

/// Converts a `ScratchObject` to a number with a check
/// to see if it's a decimal value. Mainly used by
/// the `OpRandom` block.
///
/// # Arguments
/// - `i1, i2, i3, i4` - The `ScratchObject` to convert
///   (represented this way for a predictable layout in JIT code,
///   can be `std::mem::transmute`d).
/// - `out` - The pointer to the `DecimalCheck` struct to write to.
pub unsafe extern "C" fn to_number_with_decimal_check(
    i1: i64,
    i2: i64,
    i3: i64,
    i4: i64,
    out: *mut DecimalCheck,
) {
    let i1 = (i1 as i32) as i64;
    #[cfg(debug_assertions)]
    {
        if !(0..4).contains(&i1) {
            eprintln!("error converting to number (decimal check) - enum id: {i1}");
            eprintln!("hex: {i1:X} {i2:X} {i3:X} {i4:X}");
        }
    }
    debug_assert!(i1 < 4);
    debug_assert!(i1 >= 0);
    let obj: ScratchObject = unsafe { std::mem::transmute([i1, i2, i3, i4]) };
    let (n, d) = obj.convert_to_number_with_decimal_check();
    unsafe {
        out.write(DecimalCheck {
            _number: n,
            _decimal: i64::from(d),
        });
    }
}

/// Converts a `ScratchObject` to a string.
///
/// # Arguments
/// - `i1, i2, i3, i4` - The `ScratchObject` to convert
///   (represented this way for a predictable layout in JIT code,
///   can be `std::mem::transmute`d).
/// - `out` - The pointer to the memory location to write the string to.
pub unsafe extern "C" fn to_string(i1: i64, i2: i64, i3: i64, i4: i64, out: *mut String) {
    let i1 = (i1 as i32) as i64;
    #[cfg(debug_assertions)]
    {
        if !(0..4).contains(&i1) {
            eprintln!("error converting to string - enum id: {i1}");
            eprintln!("hex: {i1:X} {i2:X} {i3:X} {i4:X}");
        }
    }
    debug_assert!(i1 < 4);
    debug_assert!(i1 >= 0);
    let obj: ScratchObject = unsafe { std::mem::transmute([i1, i2, i3, i4]) };
    let string = obj.convert_to_string();
    unsafe {
        out.write(string);
    }
}

/// Converts an f64 to a String `ScratchObject`.
pub unsafe extern "C" fn to_string_from_num(i1: f64, out: *mut ScratchObject) {
    let obj = ScratchObject::Number(i1);
    let string = obj.convert_to_string();
    unsafe { out.write(ScratchObject::String(string)) }
}

/// Converts a boolean to a String `ScratchObject`.
///
/// - If the boolean is true (1), the string will be "true".
/// - If the boolean is false (0), the string will be "false".
pub unsafe extern "C" fn to_string_from_bool(i1: i64, out: *mut ScratchObject) {
    let obj = ScratchObject::Bool(i1 != 0);
    let string = obj.convert_to_string();
    unsafe { out.write(ScratchObject::String(string)) }
}

/// Drops a `ScratchObject` in memory,
/// used to free heap allocated strings.
///
/// Ran when a variable is set to a new value,
/// dropping the old value.
pub unsafe extern "C" fn drop_obj(i1: *mut ScratchObject) {
    unsafe {
        // println!("dropping obj {:?} at mem {:X}", *i1, i1 as usize);
        std::ptr::drop_in_place(i1);
    }
}

pub unsafe extern "C" fn clone_obj(i1: i64, i2: i64, i3: i64, i4: i64, out: *mut ScratchObject) {
    let obj: ScratchObject = unsafe { std::mem::transmute([i1, i2, i3, i4]) };
    let new_obj = obj.clone();
    std::mem::forget(obj);
    unsafe { out.write(new_obj) };
}
