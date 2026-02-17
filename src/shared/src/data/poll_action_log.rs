use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct PollActionLog {
    pub id: String,
    pub chat_id: i64,
    pub topic_id: Option<i32>,
    pub action_log_message_id: i32,
    pub posting_rule_id: String,
    pub text: String,
    pub records: Vec<PollActionLogRecord>,
    pub timezone: String,
    pub version: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct PollActionLogRecord {
    pub actor_id: u64,
    pub actor_first_name: String,
    pub actor_last_name: Option<String>,
    pub actor_username: Option<String>,
    pub option_id: Option<i32>,
    pub option_text: Option<String>,
    pub timestamp: i64,
}
