use chrono::{Datelike, Duration, Local, Weekday};

pub fn get_next_weekday(target: Weekday) -> chrono::NaiveDate {
    let now = Local::now().date_naive();

    let days_until =
        (target.num_days_from_monday() as i32 - now.weekday().num_days_from_monday() as i32 + 7)
            % 7;

    let days_until = if days_until == 0 { 7 } else { days_until };

    now + Duration::days(days_until as i64)
}

pub fn get_expiry_timestamp(hours_from_now: i64) -> i64 {
    let now = Local::now();
    let expiry_time = now + Duration::hours(hours_from_now);

    expiry_time.timestamp()
}
