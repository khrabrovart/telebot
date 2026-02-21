use crate::data::{
    post::{BasePost, PostTrait},
    posting_rule::PollPostingRule,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct PollPost {
    #[serde(flatten)]
    pub base: BasePost,
    pub content: PollPostContent,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct PollPostContent {
    pub question: String,
    pub options: Vec<String>,
}

impl PollPost {
    pub fn new(
        poll_posting_rule: &PollPostingRule,
        message_id: i32,
        timestamp: i64,
        question: &str,
        options: &[String],
    ) -> Self {
        let base = BasePost::new(poll_posting_rule, message_id, timestamp);
        let content = PollPostContent {
            question: question.to_string(),
            options: options.to_vec(),
        };

        PollPost { base, content }
    }
}

impl PostTrait for PollPost {
    fn base(&self) -> &BasePost {
        &self.base
    }
}
