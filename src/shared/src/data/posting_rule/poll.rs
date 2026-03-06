use crate::data::{posting_rule::BasePostingRule, PostingRuleTrait};
use serde::{Deserialize, Serialize};
use teloxide::types::{ChatId, MessageId};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct PollPostingRule {
    #[serde(flatten)]
    base: BasePostingRule,
    pub content: PollPostingRuleContent,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub action_log: Option<PollPostingRuleActionLog>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct PollPostingRuleContent {
    pub question: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source: Option<PollOptionSource>,
    pub options: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "Type", rename_all = "PascalCase")]
pub enum PollOptionSource {
    Intersection(PollOptionIntersectionSource),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct PollOptionIntersectionSource {
    pub source_posting_rule_id: String,
    pub source_post_selector: PollOptionIntersectionSourcePostSelector,
    pub target_option_id: i32,
    pub voter_ids: Vec<Vec<u64>>,
    pub no_results_behavior: PollOptionSourceNoResultsBehavior,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "Type", rename_all = "PascalCase")]
pub enum PollOptionIntersectionSourcePostSelector {
    MostRecent,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "Type", rename_all = "PascalCase")]
pub enum PollOptionSourceNoResultsBehavior {
    SkipPosting,
    FallbackToPostingRule {
        #[serde(rename = "PostingRuleId")]
        posting_rule_id: String,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct PollPostingRuleActionLog {
    pub chat_id: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub topic_id: Option<i32>,
    pub output: PollPostingRuleActionLogOutput,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ttl_hours: Option<i64>,
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
