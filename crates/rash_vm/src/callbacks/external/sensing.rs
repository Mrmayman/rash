use rash_core::RunState;

declare_module!("sensing.rs", 9, mouse_down, days_since_2000);

pub unsafe extern "C" fn mouse_down(this: *const RunState) -> i64 {
    let this = unsafe { &*this };
    i64::from(this.mouse_down())
}

pub extern "C" fn days_since_2000() -> f64 {
    // Seconds between Unix epoch (1970-01-01) and 2000-01-01 UTC
    const SECONDS_1970_TO_2000: f64 = 946_684_800.0;
    const SECONDS_PER_DAY: f64 = 86_400.0;

    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default();

    let seconds = now.as_secs() as f64;
    let millis = f64::from(now.subsec_nanos()) / 1_000_000_000.0;

    let seconds_since_2000 = seconds + millis - SECONDS_1970_TO_2000;
    seconds_since_2000 / SECONDS_PER_DAY
}
