use telebot_shared::data::PostingRuleTrait;

pub fn format_base_fields(
    posting_rule: &impl PostingRuleTrait,
    chat_name: &str,
    is_valid: bool,
) -> String {
    let name = posting_rule.name();
    let schedule = format_schedule(posting_rule.schedule(), posting_rule.timezone());
    let pin = format_bool(posting_rule.should_pin());
    let status = if posting_rule.is_active() {
        "🟢 ВКЛЮЧЕНО"
    } else {
        "🔴 ВЫКЛЮЧЕНО"
    };
    let validity = format_bool(is_valid);

    format!(
        "<b>{name}</b>\n\n\
        Канал: <b>{chat_name}</b>\n\
        Расписание: <b>{schedule}</b>\n\
        Закрепление: {pin}\n\
        Корректность данных: {validity}\n\
        Статус: <b>{status}</b>",
    )
}

fn format_schedule(schedule: &str, timezone: &str) -> String {
    let parts: Vec<&str> = schedule.split_whitespace().collect();

    if parts.len() != 6 {
        return "INVALID_SCHEDULE".to_string();
    }

    let minutes = parts[0];
    let hours = parts[1];
    let day_of_week = match parts[4] {
        "1" => "каждое воскресенье",
        "2" => "каждый понедельник",
        "3" => "каждый вторник",
        "4" => "каждую среду",
        "5" => "каждый четверг",
        "6" => "каждую пятницу",
        "7" => "каждую субботу",
        "?" => "каждый день",
        _ => return "INVALID_SCHEDULE".to_string(),
    };

    format!("{} в {}:{} ({})", day_of_week, hours, minutes, timezone)
}

pub fn format_bool(value: bool) -> &'static str {
    if value {
        "✅"
    } else {
        "<b>-</b>"
    }
}
