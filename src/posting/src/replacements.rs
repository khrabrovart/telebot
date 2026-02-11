use chrono::Weekday;
use once_cell::sync::Lazy;
use std::collections::HashMap;

pub static REPLACEMENTS: Lazy<HashMap<&'static str, Box<dyn Fn() -> String + Sync + Send>>> =
    Lazy::new(|| {
        let mut m = HashMap::new();
        m.insert(
            "{next_monday}",
            Box::new(|| {
                let next_monday = crate::date_utils::get_next_weekday(Weekday::Mon);
                next_monday.format("%d.%m.%Y").to_string()
            }) as Box<dyn Fn() -> String + Sync + Send>,
        );
        m.insert(
            "{next_tuesday}",
            Box::new(|| {
                let next_tuesday = crate::date_utils::get_next_weekday(Weekday::Tue);
                next_tuesday.format("%d.%m.%Y").to_string()
            }) as Box<dyn Fn() -> String + Sync + Send>,
        );
        m.insert(
            "{next_wednesday}",
            Box::new(|| {
                let next_wednesday = crate::date_utils::get_next_weekday(Weekday::Wed);
                next_wednesday.format("%d.%m.%Y").to_string()
            }) as Box<dyn Fn() -> String + Sync + Send>,
        );
        m.insert(
            "{next_thursday}",
            Box::new(|| {
                let next_thursday = crate::date_utils::get_next_weekday(Weekday::Thu);
                next_thursday.format("%d.%m.%Y").to_string()
            }) as Box<dyn Fn() -> String + Sync + Send>,
        );
        m.insert(
            "{next_friday}",
            Box::new(|| {
                let next_friday = crate::date_utils::get_next_weekday(Weekday::Fri);
                next_friday.format("%d.%m.%Y").to_string()
            }) as Box<dyn Fn() -> String + Sync + Send>,
        );
        m.insert(
            "{next_saturday}",
            Box::new(|| {
                let next_saturday = crate::date_utils::get_next_weekday(Weekday::Sat);
                next_saturday.format("%d.%m.%Y").to_string()
            }) as Box<dyn Fn() -> String + Sync + Send>,
        );
        m.insert(
            "{next_sunday}",
            Box::new(|| {
                let next_sunday = crate::date_utils::get_next_weekday(Weekday::Sun);
                next_sunday.format("%d.%m.%Y").to_string()
            }) as Box<dyn Fn() -> String + Sync + Send>,
        );
        m
    });
