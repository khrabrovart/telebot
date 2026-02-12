use chrono::{Local, NaiveDate, Weekday};
use telebot_shared::data::{PostingRule, PostingRuleContent};

use crate::date_utils;

pub fn format_rule(posting_rule: &PostingRule) -> String {
    let name = &posting_rule.name;

    let status = if posting_rule.is_active {
        "ВКЛЮЧЕНО"
    } else {
        "ВЫКЛЮЧЕНО"
    };

    let text = match &posting_rule.content {
        PostingRuleContent::Text { text } => text,
        PostingRuleContent::Poll { question, options } => {
            let options = options
                .iter()
                .map(|opt| format!("- {}", opt))
                .collect::<Vec<_>>()
                .join("\n");

            &format!("{}\n\n{}", question, options)
        }
    };

    let schedule = format_schedule(&posting_rule.schedule);

    let formatted_rule = format!(
        "<b>{}</b>\n\nСтатус: <b>{}</b>\n\n{}\n\n{}",
        name, status, text, schedule
    );

    formatted_rule
}

fn format_schedule(schedule: &str) -> String {
    let parts: Vec<&str> = schedule.split_whitespace().collect();

    let minutes = parts[0];
    let hours = parts[1];
    let day_of_week_str = map_day_of_week_str(parts[4]);

    let (next_post_1, next_post_2, next_post_3) = match map_day_of_week(parts[4]) {
        Some(day_of_week) => {
            let now = Local::now().date_naive();
            let np1 = date_utils::get_next_date(day_of_week, now);
            let np2 = date_utils::get_next_date(day_of_week, np1);
            let np3 = date_utils::get_next_date(day_of_week, np2);

            (np1, np2, np3)
        }
        None => {
            let dummy = NaiveDate::from_ymd_opt(1970, 1, 1).unwrap();

            (dummy, dummy, dummy)
        }
    };

    format!(
        "<b>Расписание</b>\n{} в {}:{}\n\nСледующие публикации\n{}\n{}\n{}",
        day_of_week_str,
        hours,
        minutes,
        format_date(next_post_1, hours, minutes),
        format_date(next_post_2, hours, minutes),
        format_date(next_post_3, hours, minutes)
    )
}

fn format_date(date: NaiveDate, hours: &str, minutes: &str) -> String {
    format!("- {} в {}:{}", date.format("%d.%m.%Y"), hours, minutes)
}

fn map_day_of_week_str(day: &str) -> String {
    match day {
        "1" => "Каждое воскресенье",
        "2" => "Каждый понедельник",
        "3" => "Каждый вторник",
        "4" => "Каждую среду",
        "5" => "Каждый четверг",
        "6" => "Каждую пятницу",
        "7" => "Каждую субботу",
        "*" => "Каждый день",
        _ => "INVALID_DAY",
    }
    .to_string()
}

fn map_day_of_week(day: &str) -> Option<Weekday> {
    match day {
        "1" => Some(Weekday::Sun),
        "2" => Some(Weekday::Mon),
        "3" => Some(Weekday::Tue),
        "4" => Some(Weekday::Wed),
        "5" => Some(Weekday::Thu),
        "6" => Some(Weekday::Fri),
        "7" => Some(Weekday::Sat),
        _ => None,
    }
}
