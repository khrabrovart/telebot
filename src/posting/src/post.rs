use serde::Deserialize;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum PostValidationError {
    #[error("Text cannot be empty")]
    EmptyText,

    #[error("Poll question cannot be empty")]
    EmptyPollQuestion,

    #[error("Poll must have at least 2 options")]
    InsufficientPollOptions,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "Type", rename_all = "PascalCase")]
pub enum PostContent {
    Text {
        text: String,
    },
    Poll {
        question: String,
        options: Vec<String>,
    },
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct Post {
    pub id: String,
    pub chat_id: String,
    pub content: PostContent,
    pub schedule: String,
    #[serde(default)]
    pub is_active: bool,
    #[serde(default)]
    pub is_partial: bool,
}

impl Post {
    pub fn is_ready(&self) -> bool {
        self.is_active && !self.is_partial && self.validate().is_ok()
    }

    pub fn validate(&self) -> Result<(), PostValidationError> {
        match &self.content {
            PostContent::Text { text } => {
                if text.is_empty() {
                    return Err(PostValidationError::EmptyText);
                }
            }
            PostContent::Poll { question, options } => {
                if question.is_empty() {
                    return Err(PostValidationError::EmptyPollQuestion);
                }
                if options.len() < 2 {
                    return Err(PostValidationError::InsufficientPollOptions);
                }
            }
        }
        Ok(())
    }
}
