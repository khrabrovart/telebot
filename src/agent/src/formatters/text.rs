use telebot_shared::data::TextPostingRule;

use super::common;

pub fn format(text_posting_rule: &TextPostingRule, chat_name: &str, is_valid: bool) -> String {
    let mut message =
        common::format_base_fields(text_posting_rule, chat_name, is_valid);

    message.push_str("\n\n");
    message.push_str(&text_posting_rule.content.text);

    message
}
