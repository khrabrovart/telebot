use telebot_shared::data::{PostingRule, PostingRuleContent};

pub fn format_rule(posting_rule: &PostingRule, chat_name: &str) -> String {
    let name = &posting_rule.name;

    let chat_name = if let Some(topic_id) = &posting_rule.topic_id {
        format!("{} (топик #{})", chat_name, topic_id)
    } else {
        chat_name.to_string()
    };

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

    let schedule = format_schedule(&posting_rule.schedule, &posting_rule.timezone);

    let will_pin = if posting_rule.should_pin {
        "Есть"
    } else {
        "Нет"
    };

    let formatted_rule = format!(
        "<b>{}</b>\n\nКанал: <b>{}</b>\nРасписание: <b>{}</b>\nЗакрепление: <b>{}</b>\nСтатус: <b>{}</b>\n\n{}",
        name, chat_name, schedule, will_pin, status, text
    );

    formatted_rule
}

fn format_schedule(schedule: &str, timezone: &str) -> String {
    let parts: Vec<&str> = schedule.split_whitespace().collect();

    let minutes = parts[0];
    let hours = parts[1];
    let day_of_week = map_day_of_week(parts[4]);

    format!("{} в {}:{} ({})", day_of_week, hours, minutes, timezone)
}

fn map_day_of_week(day: &str) -> String {
    match day {
        "1" => "каждое воскресенье",
        "2" => "каждый понедельник",
        "3" => "каждый вторник",
        "4" => "каждую среду",
        "5" => "каждый четверг",
        "6" => "каждую пятницу",
        "7" => "каждую субботу",
        "?" => "каждый день",
        _ => "INVALID_DAY",
    }
    .to_string()
}
