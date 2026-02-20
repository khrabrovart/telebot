use serde::{Deserialize, Serialize};
use teloxide::types::{ChatId, MessageId};

use crate::data::{posting_rule::BasePostingRule, PostingRuleTrait};

// TODO: Rename poll_action_log to action_log

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct PollPostingRule {
    #[serde(flatten)]
    pub base: BasePostingRule,
    pub content: PollPostingRuleContent,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub poll_action_log: Option<PollPostingRuleActionLog>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct PollPostingRuleContent {
    pub question: String,
    pub options: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct PollPostingRuleActionLog {
    pub chat_id: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub topic_id: Option<i32>,
    pub output: PollPostingRuleActionLogOutput,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "Type", rename_all = "PascalCase")]
pub enum PollPostingRuleActionLogOutput {
    All,
    OnlyWhenTargetOptionRevoked {
        #[serde(rename = "TargetOptionId")]
        target_option_id: i32,
    },
}

impl PollPostingRuleActionLog {
    pub fn chat_id(&self) -> ChatId {
        ChatId(self.chat_id)
    }

    pub fn topic_id(&self) -> Option<MessageId> {
        self.topic_id.map(MessageId)
    }
}

impl PostingRuleTrait for PollPostingRule {
    fn base(&self) -> &BasePostingRule {
        &self.base
    }

    fn set_active(&mut self, active: bool) {
        self.base.is_active = active;
    }
}
