use serde::{Deserialize, Serialize};
use teloxide::types::{ChatId, MessageId};

// TODO: Add a PostingRule MessageType field to determine the type of the message (poll or text) and put specific fields for each type instead of having all the fields in the PostingRule
// TODO: Add proper repository structures for data types and create DynamoDB only once at the Cold Start

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "Type", rename_all = "PascalCase")]
pub enum PostingRule {
    Text(TextPostingRule),
    Poll(PollPostingRule),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct BasePostingRule {
    pub id: String,
    pub bot_id: String,
    pub chat_id: i64,
    pub topic_id: Option<i32>,
    pub name: String,
    pub description: Option<String>,
    pub schedule: String,
    pub timezone: String,
    #[serde(default)]
    pub should_pin: bool,
    #[serde(default)]
    pub is_active: bool,
    pub expire_after_hours: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct TextPostingRule {
    #[serde(flatten)]
    pub base: BasePostingRule,
    pub content: TextPostingRuleContent,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct PollPostingRule {
    #[serde(flatten)]
    pub base: BasePostingRule,
    pub content: PollPostingRuleContent,
    pub poll_action_log: Option<PollActionLogConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct TextPostingRuleContent {
    pub text: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct PollPostingRuleContent {
    pub question: String,
    pub options: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct PollActionLogConfig {
    pub chat_id: i64,
    pub topic_id: Option<i32>,
    pub output: PollActionLogOutput,
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

impl PostingRule {
    pub fn is_valid(&self) -> bool {
        match self {
            PostingRule::Text(rule) => {
                !rule.base.bot_id.is_empty()
                    && !rule.base.name.is_empty()
                    && !rule.base.schedule.is_empty()
                    && !rule.base.timezone.is_empty()
                    && !rule.content.text.is_empty()
            }
            PostingRule::Poll(rule) => {
                !rule.base.bot_id.is_empty()
                    && !rule.base.name.is_empty()
                    && !rule.base.schedule.is_empty()
                    && !rule.base.timezone.is_empty()
                    && !rule.content.question.is_empty()
                    && !rule.content.options.is_empty()
            }
        }
    }

    pub fn base(&self) -> &BasePostingRule {
        match self {
            PostingRule::Text(rule) => &rule.base,
            PostingRule::Poll(rule) => &rule.base,
        }
    }

    pub fn id(&self) -> &str {
        match self {
            PostingRule::Text(rule) => &rule.base.id,
            PostingRule::Poll(rule) => &rule.base.id,
        }
    }

    pub fn bot_id(&self) -> &str {
        match self {
            PostingRule::Text(rule) => &rule.base.bot_id,
            PostingRule::Poll(rule) => &rule.base.bot_id,
        }
    }

    pub fn chat_id(&self) -> ChatId {
        match self {
            PostingRule::Text(rule) => ChatId(rule.base.chat_id),
            PostingRule::Poll(rule) => ChatId(rule.base.chat_id),
        }
    }

    pub fn topic_id(&self) -> Option<MessageId> {
        match self {
            PostingRule::Text(rule) => rule.base.topic_id.map(MessageId),
            PostingRule::Poll(rule) => rule.base.topic_id.map(MessageId),
        }
    }

    pub fn name(&self) -> &str {
        match self {
            PostingRule::Text(rule) => &rule.base.name,
            PostingRule::Poll(rule) => &rule.base.name,
        }
    }

    pub fn description(&self) -> Option<&str> {
        match self {
            PostingRule::Text(rule) => rule.base.description.as_deref(),
            PostingRule::Poll(rule) => rule.base.description.as_deref(),
        }
    }

    pub fn schedule(&self) -> &str {
        match self {
            PostingRule::Text(rule) => &rule.base.schedule,
            PostingRule::Poll(rule) => &rule.base.schedule,
        }
    }

    pub fn timezone(&self) -> &str {
        match self {
            PostingRule::Text(rule) => &rule.base.timezone,
            PostingRule::Poll(rule) => &rule.base.timezone,
        }
    }

    pub fn should_pin(&self) -> bool {
        match self {
            PostingRule::Text(rule) => rule.base.should_pin,
            PostingRule::Poll(rule) => rule.base.should_pin,
        }
    }

    pub fn is_active(&self) -> bool {
        match self {
            PostingRule::Text(rule) => rule.base.is_active,
            PostingRule::Poll(rule) => rule.base.is_active,
        }
    }

    pub fn set_active(&mut self, active: bool) {
        match self {
            PostingRule::Text(rule) => rule.base.is_active = active,
            PostingRule::Poll(rule) => rule.base.is_active = active,
        }
    }

    pub fn expire_after_hours(&self) -> Option<i64> {
        match self {
            PostingRule::Text(rule) => rule.base.expire_after_hours,
            PostingRule::Poll(rule) => rule.base.expire_after_hours,
        }
    }
}

impl PollActionLogConfig {
    pub fn chat_id(&self) -> ChatId {
        ChatId(self.chat_id)
    }

    pub fn topic_id(&self) -> Option<MessageId> {
        self.topic_id.map(MessageId)
    }
}
