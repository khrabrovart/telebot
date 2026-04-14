use chrono::{Datelike, Duration, Local, Weekday};

const RUSSIAN_MONTH_NAMES_NOMINATIVE: [&str; 12] = [
    "январь",
    "февраль",
    "март",
    "апрель",
    "май",
    "июнь",
    "июль",
    "август",
    "сентябрь",
    "октябрь",
    "ноябрь",
    "декабрь",
];

pub fn next_month_name_russian() -> String {
    let month = match Local::now().date_naive().month() {
        12 => 1,
        m => m + 1,
    };

    RUSSIAN_MONTH_NAMES_NOMINATIVE[(month - 1) as usize].to_string()
}

pub fn get_next_weekday(target: Weekday) -> chrono::NaiveDate {
    let now = Local::now().date_naive();

    let days_until =
        (target.num_days_from_monday() as i32 - now.weekday().num_days_from_monday() as i32 + 7)
            % 7;

    let days_until = if days_until == 0 { 7 } else { days_until };

    now + Duration::days(days_until as i64)
}
