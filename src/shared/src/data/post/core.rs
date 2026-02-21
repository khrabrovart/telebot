use crate::data::{
    post::{PollPost, TextPost},
    PostTrait,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "Type", rename_all = "PascalCase")]
pub enum Post {
    Text(TextPost),
    Poll(PollPost),
}

impl PostTrait for Post {
    fn base(&self) -> &crate::data::post::BasePost {
        match self {
            Post::Text(text_post) => text_post.base(),
            Post::Poll(poll_post) => poll_post.base(),
        }
    }
}
