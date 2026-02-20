use crate::data::posting_rule::BasePostingRule;
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
