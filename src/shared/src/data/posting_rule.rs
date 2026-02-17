use serde::{Deserialize, Serialize};
use teloxide::types::{ChatId, MessageId};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct PostingRule {
    pub id: String,
    pub bot_id: String,
    pub chat_id: ChatId,
    pub topic_id: Option<MessageId>,
    pub name: String,
    pub content: PostingRuleContent,
    pub schedule: String,
    pub timezone: String,
    #[serde(default)]
    pub should_pin: bool,
    #[serde(default)]
    pub is_active: bool,
    pub poll_action_log: Option<PollActionLogConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct PollActionLogConfig {
    pub chat_id: ChatId,
    pub topic_id: Option<MessageId>,
    pub output: PollActionLogOutput,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "Type", rename_all = "PascalCase")]
pub enum PostingRuleContent {
    Text {
        #[serde(rename = "Text")]
        text: String,
    },
    Poll {
        #[serde(rename = "Question")]
        question: String,
        #[serde(rename = "Options")]
        options: Vec<String>,
    },
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
        !self.bot_id.is_empty()
            && !self.name.is_empty()
            && !self.schedule.is_empty()
            && !self.timezone.is_empty()
            && match &self.content {
                PostingRuleContent::Text { text } => !text.is_empty(),
                PostingRuleContent::Poll { question, options } => {
                    !question.is_empty() && !options.is_empty()
                }
            }
    }
}
