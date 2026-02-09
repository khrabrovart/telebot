use serde::Deserialize;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum PostParseError {
    #[error("Deserialization error: {0}")]
    DeserializationError(String),
}

#[derive(Debug, Clone, Deserialize)]
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

// TODO: Make everything optional and add validation to ensure we have the necessary fields for scheduling
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct Post {
    pub id: String,
    pub chat_id: String,
    pub content: PostContent,
    pub schedule: String,
    pub timezone: String,
    #[serde(default)]
    pub is_active: bool,
    #[serde(default)]
    pub is_ready: bool,
}
