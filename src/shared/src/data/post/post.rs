use serde::{Deserialize, Serialize};

use crate::data::post::{poll_post::PollPost, text_post::TextPost};

// TODO: Implement TTL correctly for Post and PollActionLog
// TODO: Remove expired posts from chats or allow different behavior like closing polls instead of deleting the messages

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "Type", rename_all = "PascalCase")]
pub enum Post {
    Text(TextPost),
    Poll(PollPost),
}
