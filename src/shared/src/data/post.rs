use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct Post {
    pub chat_id: i64,
    pub topic_id: Option<i32>,
    pub message_id: i32,
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
