use serde::{Deserialize, Serialize};

// TODO: Remove unnecessary fields that exist in the Post and use the Post as the source of truth for the post data instead of duplicating it in the PollActionLog

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct PollActionLog {
    pub id: String,
    pub chat_id: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub topic_id: Option<i32>,
    pub message_id: i32,
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
