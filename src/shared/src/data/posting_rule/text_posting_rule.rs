use crate::data::{posting_rule::BasePostingRule, PostingRuleTrait};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct TextPostingRule {
    #[serde(flatten)]
    pub base: BasePostingRule,
    pub content: TextPostingRuleContent,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct TextPostingRuleContent {
    pub text: String,
}

impl PostingRuleTrait for TextPostingRule {
    fn base(&self) -> &BasePostingRule {
        &self.base
    }

    fn set_active(&mut self, active: bool) {
        self.base.is_active = active;
    }
}
