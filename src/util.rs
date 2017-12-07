use std::time::{SystemTime, UNIX_EPOCH, Duration};

pub fn unixtime_to_systemtime(unixtime: f64) -> SystemTime {
    let duration = Duration::new(unixtime as u64, (unixtime * 1_000_000_000.0) as u32);
    UNIX_EPOCH + duration
}
