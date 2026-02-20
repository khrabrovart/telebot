use crate::data::{
    posting_rule::BasePostingRule, PollPostingRule, PostingRuleTrait, TextPostingRule,
};
use serde::{Deserialize, Serialize};

// TODO: Add proper repository structures for data types and create DynamoDB only once at the Cold Start

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "Type", rename_all = "PascalCase")]
pub enum PostingRule {
    Text(TextPostingRule),
    Poll(PollPostingRule),
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
}

impl PostingRuleTrait for PostingRule {
    fn base(&self) -> &BasePostingRule {
        match self {
            PostingRule::Text(rule) => &rule.base,
            PostingRule::Poll(rule) => &rule.base,
        }
    }

    fn set_active(&mut self, active: bool) {
        match self {
            PostingRule::Text(rule) => rule.base.is_active = active,
            PostingRule::Poll(rule) => rule.base.is_active = active,
        }
    }
}
