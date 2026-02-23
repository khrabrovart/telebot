use crate::{
    data::{PollPostingRule, PollPostingRuleActionLog, PostingRuleTrait},
    date,
};
use serde::{Deserialize, Serialize};
use teloxide::types::{MessageId, PollId};

// TODO: Remove unnecessary fields that exist in the Post and use the Post as the source of truth for the post data instead of duplicating it in the PollActionLog

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct PollActionLog {
    pub id: String,
    pub chat_id: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub topic_id: Option<i32>,
    pub message_id: i32,
    pub action_log_chat_id: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub action_log_topic_id: Option<i32>,
    pub action_log_message_id: i32,
    pub posting_rule_id: String,
    pub text: String,
    pub records: Vec<PollActionLogRecord>,
    pub timezone: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expires_at: Option<i64>,
    pub version: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct PollActionLogRecord {
    pub actor_id: u64,
    pub actor_first_name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub actor_last_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub actor_username: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub option_id: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub option_text: Option<String>,
    pub timestamp: i64,
}

impl PollActionLog {
    pub fn new(
        poll_posting_rule: &PollPostingRule,
        poll_posting_rule_action_log: &PollPostingRuleActionLog,
        poll_id: PollId,
        message_id: MessageId,
        action_log_message_id: MessageId,
        text: String,
    ) -> Self {
        let ttl_hours = match &poll_posting_rule.poll_action_log {
            Some(action_log) => action_log.ttl_hours,
            None => None,
        };

        let expires_at = ttl_hours.map(date::calculate_expires_at);

        PollActionLog {
            id: poll_id.to_string(),
            chat_id: poll_posting_rule.chat_id().0,
            topic_id: poll_posting_rule.topic_id().map(|topic_id| topic_id.0),
            message_id: message_id.0,
            action_log_chat_id: poll_posting_rule_action_log.chat_id().0,
            action_log_topic_id: poll_posting_rule_action_log
                .topic_id()
                .map(|topic_id| topic_id.0),
            action_log_message_id: action_log_message_id.0,
            posting_rule_id: poll_posting_rule.id().to_string(),
            text,
            records: vec![],
            timezone: poll_posting_rule.timezone().to_string(),
            expires_at,
            version: 0,
        }
    }
}
