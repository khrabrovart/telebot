use chrono::{Local, Weekday};
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
        "**{}**\n\nСтатус: {}\n\n{}\n\n{}",
        name, status, text, schedule
    );

    formatted_rule
}

fn format_schedule(schedule: &str) -> String {
    let parts: Vec<&str> = schedule.split_whitespace().collect();

    let minutes = parts[0];
    let hours = parts[1];
    let day_of_week = map_day_of_week(parts[4]);
    let day_of_week_str = map_day_of_week_str(parts[4]);

    let now = Local::now().date_naive();

    let next_post_1 = date_utils::get_next_date(day_of_week, now);
    let next_post_2 = date_utils::get_next_date(day_of_week, next_post_1);
    let next_post_3 = date_utils::get_next_date(day_of_week, next_post_2);

    format!(
        "Расписание\n**{} в {}:{}**\n\nСледующие публикации\n{}\n{}\n{}",
        day_of_week_str, hours, minutes, next_post_1, next_post_2, next_post_3
    )
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
        _ => "Неверный день недели",
    }
    .to_string()
}

fn map_day_of_week(day: &str) -> Weekday {
    match day {
        "1" => Weekday::Sun,
        "2" => Weekday::Mon,
        "3" => Weekday::Tue,
        "4" => Weekday::Wed,
        "5" => Weekday::Thu,
        "6" => Weekday::Fri,
        "7" => Weekday::Sat,
        _ => panic!("Invalid day of week"),
    }
}
