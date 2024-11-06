use crate::data_types::ScratchObject;

pub extern "C" fn to_bool(i1: i64, i2: i64, i3: i64, i4: i64) -> i64 {
    let i1 = (i1 as i32) as i64;
    #[cfg(debug_assertions)]
    {
        if !(0..4).contains(&i1) {
            eprintln!("error converting to bool - enum id: {i1}");
            eprintln!("hex: {i1:X} {i2:X} {i3:X} {i4:X}");
            eprintln!("hex: {:X} {:X} (bitshift)", i1 << 32, i2 << 32);
        }
    }
    debug_assert!(i1 < 4);
    debug_assert!(i1 >= 0);
    let obj: ScratchObject = unsafe { std::mem::transmute([i1, i2, i3, i4]) };
    obj.convert_to_bool() as i64
}

pub extern "C" fn to_number(i1: i64, i2: i64, i3: i64, i4: i64) -> f64 {
    let i1 = (i1 as i32) as i64;
    #[cfg(debug_assertions)]
    {
        if !(0..4).contains(&i1) {
            eprintln!("error converting to number - enum id: {i1}");
            eprintln!("hex: {i1:X} {i2:X} {i3:X} {i4:X}");
            eprintln!("hex: {:X} {:X} (bitshift)", i1 << 32, i2 << 32);
        }
    }
    debug_assert!(i1 < 4);
    debug_assert!(i1 >= 0);
    let obj: ScratchObject = unsafe { std::mem::transmute([i1, i2, i3, i4]) };
    obj.convert_to_number()
}

pub extern "C" fn to_string(i1: i64, i2: i64, i3: i64, i4: i64, out: *mut i64) {
    let i1 = (i1 as i32) as i64;
    // println!("{i1}");
    #[cfg(debug_assertions)]
    {
        if !(0..4).contains(&i1) {
            eprintln!("error converting to string - enum id: {i1}");
            eprintln!("hex: {i1:X} {i2:X} {i3:X} {i4:X}");
            eprintln!("hex: {:X} {:X} (bitshift)", i1 << 32, i2 << 32);
        }
    }
    debug_assert!(i1 < 4);
    debug_assert!(i1 >= 0);
    let obj: ScratchObject = unsafe { std::mem::transmute([i1, i2, i3, i4]) };
    let string = obj.convert_to_string();
    // println!("to string: {obj}, {string}");
    let bytes: [i64; 3] = unsafe { std::mem::transmute(string) };
    // print!("writing: ");
    // for byte in bytes {
    // print!("{byte:X}, ")
    // }
    // println!();
    unsafe {
        out.write(bytes[0]);
        out.offset(1).write(bytes[1]);
        out.offset(2).write(bytes[2]);
    };
}

pub extern "C" fn to_string_from_num(i1: f64, out: *mut ScratchObject) {
    let obj = ScratchObject::Number(i1);
    let string = obj.convert_to_string();
    unsafe { out.write(ScratchObject::String(string)) }
}

pub extern "C" fn to_string_from_bool(i1: i64, out: *mut ScratchObject) {
    let obj = ScratchObject::Bool(i1 != 0);
    let string = obj.convert_to_string();
    unsafe { out.write(ScratchObject::String(string)) }
}

pub extern "C" fn drop_obj(i1: *mut ScratchObject) {
    unsafe {
        // println!("dropping obj {}", *i1);
        // println!("setting var: force drop obj: {}", *i1);
        std::ptr::drop_in_place(i1);
    }
}
