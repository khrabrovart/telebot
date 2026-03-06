use crate::{
    data::{
        PollPostingRule, PollPostingRuleActionLog, PollPostingRuleActionLogOutput, PostingRuleTrait,
    },
    date,
};
use serde::{Deserialize, Serialize};
use teloxide::types::{ChatId, MessageId, PollId, UpdateId, User};

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
    pub output: PollActionLogOutput,
    pub records: Vec<PollActionLogRecord>,
    pub timezone: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expires_at: Option<i64>,
    pub version: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct PollActionLogRecord {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub update_id: Option<UpdateId>,
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

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "Type", rename_all = "PascalCase")]
pub enum PollActionLogOutput {
    All,
    OnlyWhenTargetOptionRevoked {
        #[serde(rename = "TargetOptionId")]
        target_option_id: i32,
    },
}

impl PollActionLog {
    pub fn new(
        poll_posting_rule: &PollPostingRule,
        poll_posting_rule_action_log: &PollPostingRuleActionLog,
        poll_id: PollId,
        message_id: MessageId,
        action_log_message_id: MessageId,
    ) -> Self {
        let ttl_hours = match &poll_posting_rule.action_log {
            Some(action_log) => action_log.ttl_hours,
            None => None,
        };

        let expires_at = ttl_hours.map(date::calculate_expires_at);

        let output = match &poll_posting_rule_action_log.output {
            PollPostingRuleActionLogOutput::All => PollActionLogOutput::All,
            PollPostingRuleActionLogOutput::OnlyWhenTargetOptionRevoked { target_option_id } => {
                PollActionLogOutput::OnlyWhenTargetOptionRevoked {
                    target_option_id: *target_option_id,
                }
            }
        };

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
            output,
            records: vec![],
            timezone: poll_posting_rule.timezone().to_string(),
            expires_at,
            version: 0,
        }
    }

    pub fn chat_id(&self) -> ChatId {
        ChatId(self.chat_id)
    }

    pub fn topic_id(&self) -> Option<MessageId> {
        self.topic_id.map(MessageId)
    }

    pub fn message_id(&self) -> MessageId {
        MessageId(self.message_id)
    }

    pub fn action_log_chat_id(&self) -> ChatId {
        ChatId(self.action_log_chat_id)
    }

    pub fn action_log_topic_id(&self) -> Option<MessageId> {
        self.action_log_topic_id.map(MessageId)
    }

    pub fn action_log_message_id(&self) -> MessageId {
        MessageId(self.action_log_message_id)
    }
}

impl PollActionLogRecord {
    pub fn new(
        update_id: UpdateId,
        user: &User,
        option_id: Option<i32>,
        option_text: Option<String>,
    ) -> Self {
        let timestamp = chrono::Utc::now().timestamp();

        PollActionLogRecord {
            update_id: Some(update_id),
            actor_id: user.id.0,
            actor_first_name: user.first_name.clone(),
            actor_last_name: user.last_name.clone(),
            actor_username: user.username.clone(),
            option_id,
            option_text,
            timestamp,
        }
    }
}
