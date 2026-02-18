use chrono::{Duration, Local};

pub fn calculate_expires_at(hours_from_now: i64) -> i64 {
    let now = Local::now();
    let expiry_time = now + Duration::hours(hours_from_now);

    expiry_time.timestamp()
}
