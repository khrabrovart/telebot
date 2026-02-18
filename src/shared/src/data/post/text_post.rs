use serde::{Deserialize, Serialize};

use crate::data::{post::base_post::BasePost, posting_rule::TextPostingRule};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct TextPost {
    #[serde(flatten)]
    pub base: BasePost,
    pub content: TextPostContent,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct TextPostContent {
    pub text: String,
}

impl TextPost {
    pub fn new(
        text_posting_rule: &TextPostingRule,
        message_id: i32,
        timestamp: i64,
        text: &str,
    ) -> Self {
        let base = BasePost::new(&text_posting_rule.base, message_id, timestamp);
        let content = TextPostContent {
            text: text.to_string(),
        };

        TextPost { base, content }
    }
}
