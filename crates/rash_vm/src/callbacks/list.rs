use ecow::EcoString;

use crate::{ScratchObject, lists::ScratchList};

declare_module!("list.rs", 10, clear_obj, clear_num, clear_bool, clear_str);

pub unsafe extern "C" fn clear_obj(lst: *mut ScratchList<ScratchObject>) {
    unsafe { &mut *lst }.clear();
}

pub unsafe extern "C" fn clear_num(lst: *mut ScratchList<f64>) {
    unsafe { &mut *lst }.clear();
}

pub unsafe extern "C" fn clear_bool(lst: *mut ScratchList<bool>) {
    unsafe { &mut *lst }.clear();
}

pub unsafe extern "C" fn clear_str(lst: *mut ScratchList<EcoString>) {
    unsafe { &mut *lst }.clear();
}
