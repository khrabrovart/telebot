use chrono::{Datelike, Duration, Weekday};

pub fn get_next_date(target: Weekday, related_to: chrono::NaiveDate) -> chrono::NaiveDate {
    let days_until = (target.num_days_from_monday() as i32
        - related_to.weekday().num_days_from_monday() as i32
        + 7)
        % 7;

    let days_until = if days_until == 0 { 7 } else { days_until };

    related_to + Duration::days(days_until as i64)
}
