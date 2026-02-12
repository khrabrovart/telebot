use telebot_shared::data::{PostingRule, PostingRuleContent};

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
    let day_of_week = map_day_of_week(parts[4]);

    format!("<b>Расписание</b>\n{} в {}:{}", day_of_week, hours, minutes,)
}

fn map_day_of_week(day: &str) -> String {
    match day {
        "1" => "Каждое воскресенье",
        "2" => "Каждый понедельник",
        "3" => "Каждый вторник",
        "4" => "Каждую среду",
        "5" => "Каждый четверг",
        "6" => "Каждую пятницу",
        "7" => "Каждую субботу",
        "?" => "Каждый день",
        _ => "INVALID_DAY",
    }
    .to_string()
}
