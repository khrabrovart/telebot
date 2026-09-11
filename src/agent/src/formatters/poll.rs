use telebot_shared::data::PollPostingRule;

use super::common;

pub fn format(poll_posting_rule: &PollPostingRule, chat_name: &str, is_valid: bool) -> String {
    let mut message = common::format_base_fields(poll_posting_rule, chat_name, is_valid);

    message.push_str("\n\n");
    message.push_str(&format_content(poll_posting_rule));

    message
}

fn format_content(poll_posting_rule: &PollPostingRule) -> String {
    if poll_posting_rule.content.option_sourcing.is_some() {
        format!(
            "{}\n\n<i>Опции формируются динамически</i>",
            poll_posting_rule.content.question
        )
    } else {
        let options = poll_posting_rule
            .content
            .options
            .iter()
            .map(|opt| format!("🔘 {}", opt))
            .collect::<Vec<_>>()
            .join("\n");

        format!("{}\n\n{}", poll_posting_rule.content.question, options)
    }
}
