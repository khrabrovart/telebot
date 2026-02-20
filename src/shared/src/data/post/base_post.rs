use crate::{data::PostingRuleTrait, date};
use serde::{Deserialize, Serialize};

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
    pub fn new(posting_rule: &impl PostingRuleTrait, message_id: i32, timestamp: i64) -> Self {
        let expires_at = posting_rule
            .base()
            .expire_after_hours
            .map(date::calculate_expires_at);

        BasePost {
            chat_id: posting_rule.base().chat_id,
            topic_id: posting_rule.base().topic_id,
            message_id,
            bot_id: posting_rule.base().bot_id.clone(),
            posting_rule_id: posting_rule.base().id.clone(),
            schedule: posting_rule.base().schedule.clone(),
            timezone: posting_rule.base().timezone.clone(),
            is_pinned: posting_rule.base().should_pin,
            timestamp,
            expires_at,
        }
    }
}
