use colored::Colorize;
use ecow::EcoString;

declare_module!("env.rs", 2, dbg_log);

pub unsafe extern "C" fn dbg_log(msg: *mut EcoString) {
    let msg_val = unsafe { &mut *msg };
    if !msg_val.is_empty() {
        println!("{} {msg_val:?}", "[say]".bright_black());
    }
    unsafe { msg.drop_in_place() };
}
