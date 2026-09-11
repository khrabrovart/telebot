mod common;
mod poll;
mod text;

use telebot_shared::data::{PostingRule, PostingRuleTrait};

pub fn format_rule(posting_rule: &PostingRule, chat_name: &str) -> String {
    let chat_name = if let Some(topic_id) = posting_rule.topic_id() {
        format!("{} (топик {})", chat_name, topic_id.0)
    } else {
        chat_name.to_string()
    };

    let is_valid = posting_rule.is_valid();

    match posting_rule {
        PostingRule::Text(text_posting_rule) => text::format(text_posting_rule, &chat_name, is_valid),
        PostingRule::Poll(poll_posting_rule) => poll::format(poll_posting_rule, &chat_name, is_valid),
    }
}
