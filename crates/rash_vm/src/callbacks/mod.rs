//! Callbacks for JIT code to call into Rust code.
//!
//! These functions are called by JIT code to perform
//! operations that are not possible in JIT code.

use core::f64;

use colored::Colorize;

macro_rules! print_func {
    ($module:expr, $($fn:ident),+ $(,)?) => {
        pub fn print_function_addresses() {
            fn print(name: &str, addr: *const ()) {
                println!("{name:40} = {:#018x}", addr as usize);
            }

            println!("\n========");
            println!("{}", $module);
            println!("========");

            $(
                print(stringify!($fn), $fn as *const ());
            )+
        }
    };
}

pub mod custom_block;
pub mod op;
pub mod repeat_stack;
pub mod types;

pub fn print_function_addresses() {
    fn print(name: &str, addr: *const ()) {
        println!("{name:25} = {:#018x}", addr as usize);
    }

    custom_block::print_function_addresses();
    repeat_stack::print_function_addresses();
    types::print_function_addresses();
    op::print_function_addresses();

    println!("\n========");
    println!("mod.rs");
    println!("========");

    print("dbg_log", dbg_log as *const ());
    print("op_days_since_2000", days_since_2000 as *const ());
}

pub unsafe extern "C" fn dbg_log(msg: *mut String, is_const: i64) {
    let msg_val = unsafe { &mut *msg };
    if !msg_val.is_empty() {
        println!("{} {msg_val:?}", "[say]".bright_black());
    }
    if is_const == 0 {
        unsafe { msg.drop_in_place() };
    }
}

pub extern "C" fn days_since_2000() -> f64 {
    // Seconds between Unix epoch (1970-01-01) and 2000-01-01 UTC
    const SECONDS_1970_TO_2000: f64 = 946_684_800.0;
    const SECONDS_PER_DAY: f64 = 86_400.0;

    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("Time went backwards");

    let seconds = now.as_secs() as f64;
    let millis = now.subsec_nanos() as f64 / 1_000_000_000.0;

    let seconds_since_2000 = seconds + millis - SECONDS_1970_TO_2000;
    seconds_since_2000 / SECONDS_PER_DAY
}
