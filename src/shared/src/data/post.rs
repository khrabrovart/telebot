use serde::{Deserialize, Serialize};

use crate::{
    data::posting_rule::{BasePostingRule, PollPostingRule, TextPostingRule},
    date,
};

// TODO: Implement TTL correctly for Post and PollActionLog
// TODO: Remove expired posts from chats or allow different behavior like closing polls instead of deleting the messages
// TODO: Add a Post Type field to determine the type of the message (poll or text) and put specific fields for each type instead of having all the fields in the Post - the same as PostingRule

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "Type", rename_all = "PascalCase")]
pub enum Post {
    Text(TextPost),
    Poll(PollPost),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct BasePost {
    pub chat_id: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub topic_id: Option<i32>,
    pub message_id: i32,
    pub bot_id: String,
    pub posting_rule_id: String,
    pub schedule: String,
    pub timezone: String,
    pub is_pinned: bool,
    pub timestamp: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expires_at: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct TextPost {
    #[serde(flatten)]
    pub base: BasePost,
    pub content: TextPostContent,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct PollPost {
    #[serde(flatten)]
    pub base: BasePost,
    pub content: PollPostContent,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct TextPostContent {
    pub text: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct PollPostContent {
    pub question: String,
    pub options: Vec<String>,
}

impl BasePost {
    pub fn new(base_posting_rule: &BasePostingRule, message_id: i32, timestamp: i64) -> Self {
        BasePost {
            chat_id: base_posting_rule.chat_id,
            topic_id: base_posting_rule.topic_id,
            message_id,
            bot_id: base_posting_rule.bot_id.clone(),
            posting_rule_id: base_posting_rule.id.clone(),
            schedule: base_posting_rule.schedule.clone(),
            timezone: base_posting_rule.timezone.clone(),
            is_pinned: base_posting_rule.should_pin,
            timestamp,
            expires_at: base_posting_rule
                .expire_after_hours
                .map(date::calculate_expires_at),
        }
    }
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

impl PollPost {
    pub fn new(
        poll_posting_rule: &PollPostingRule,
        message_id: i32,
        timestamp: i64,
        question: &str,
        options: &Vec<String>,
    ) -> Self {
        let base = BasePost::new(&poll_posting_rule.base, message_id, timestamp);
        let content = PollPostContent {
            question: question.to_string(),
            options: options.clone(),
        };

        PollPost { base, content }
    }
}
