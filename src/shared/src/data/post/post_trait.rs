use crate::data::post::BasePost;
use teloxide::types::{ChatId, MessageId};

pub trait PostTrait {
    fn base(&self) -> &BasePost;

    fn chat_id(&self) -> ChatId {
        ChatId(self.base().chat_id)
    }

    fn topic_id(&self) -> Option<MessageId> {
        self.base().topic_id.map(MessageId)
    }

    fn message_id(&self) -> MessageId {
        MessageId(self.base().message_id)
    }

    fn bot_id(&self) -> &str {
        &self.base().bot_id
    }

    fn posting_rule_id(&self) -> &str {
        &self.base().posting_rule_id
    }

    fn schedule(&self) -> &str {
        &self.base().schedule
    }

    fn timezone(&self) -> &str {
        &self.base().timezone
    }

    fn is_pinned(&self) -> bool {
        self.base().is_pinned
    }

    fn timestamp(&self) -> i64 {
        self.base().timestamp
    }

    fn expires_at(&self) -> Option<i64> {
        self.base().expires_at
    }
}
