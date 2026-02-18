use serde::{Deserialize, Serialize};

use crate::{data::posting_rule::BasePostingRule, date};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct BasePost {
    pub chat_id: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub topic_id: Option<i32>,
    pub message_id: i32,
    pub bot_id: String,
    pub posting_rule_id: String,
    pub schedule: String,
    pub timezone: String,
    pub is_pinned: bool,
    pub timestamp: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expires_at: Option<i64>,
}

impl BasePost {
    pub fn new(base_posting_rule: &BasePostingRule, message_id: i32, timestamp: i64) -> Self {
        BasePost {
            chat_id: base_posting_rule.chat_id,
            topic_id: base_posting_rule.topic_id,
            message_id,
            bot_id: base_posting_rule.bot_id.clone(),
            posting_rule_id: base_posting_rule.id.clone(),
            schedule: base_posting_rule.schedule.clone(),
            timezone: base_posting_rule.timezone.clone(),
            is_pinned: base_posting_rule.should_pin,
            timestamp,
            expires_at: base_posting_rule
                .expire_after_hours
                .map(date::calculate_expires_at),
        }
    }
}
