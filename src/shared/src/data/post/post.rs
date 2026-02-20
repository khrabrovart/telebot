use crate::data::{
    post::{poll_post::PollPost, text_post::TextPost},
    PostTrait,
};
use serde::{Deserialize, Serialize};

// TODO: Implement TTL correctly for Post and PollActionLog
// TODO: Remove expired posts from chats or allow different behavior like closing polls instead of deleting the messages

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "Type", rename_all = "PascalCase")]
pub enum Post {
    Text(TextPost),
    Poll(PollPost),
}

impl PostTrait for Post {
    fn base(&self) -> &crate::data::post::base_post::BasePost {
        match self {
            Post::Text(text_post) => text_post.base(),
            Post::Poll(poll_post) => poll_post.base(),
        }
    }
}
