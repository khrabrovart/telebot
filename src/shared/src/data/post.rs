use serde::{Deserialize, Serialize};
use teloxide::types::{ChatId, MessageId};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct Post {
    pub chat_id: ChatId,
    pub topic_id: Option<MessageId>,
    pub message_id: MessageId,
    pub bot_id: String,
    pub name: String,
    pub content: PostContent,
    pub schedule: String,
    pub timezone: String,
    pub is_pinned: bool,
    pub timestamp: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "Type", rename_all = "PascalCase")]
pub enum PostContent {
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
