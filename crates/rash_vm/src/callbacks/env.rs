use colored::Colorize;

declare_module!("env.rs", 2, dbg_log, days_since_2000);

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
    let millis = f64::from(now.subsec_nanos()) / 1_000_000_000.0;

    let seconds_since_2000 = seconds + millis - SECONDS_1970_TO_2000;
    seconds_since_2000 / SECONDS_PER_DAY
}
