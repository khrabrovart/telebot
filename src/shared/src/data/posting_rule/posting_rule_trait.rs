use crate::data::posting_rule::BasePostingRule;
use teloxide::types::{ChatId, MessageId};

pub trait PostingRuleTrait {
    fn base(&self) -> &BasePostingRule;

    fn id(&self) -> &str {
        &self.base().id
    }

    fn bot_id(&self) -> &str {
        &self.base().bot_id
    }

    fn chat_id(&self) -> ChatId {
        ChatId(self.base().chat_id)
    }

    fn topic_id(&self) -> Option<MessageId> {
        self.base().topic_id.map(MessageId)
    }

    fn name(&self) -> &str {
        &self.base().name
    }

    fn description(&self) -> Option<&str> {
        self.base().description.as_deref()
    }

    fn schedule(&self) -> &str {
        &self.base().schedule
    }

    fn timezone(&self) -> &str {
        &self.base().timezone
    }

    fn should_pin(&self) -> bool {
        self.base().should_pin
    }

    fn is_active(&self) -> bool {
        self.base().is_active
    }

    fn expire_after_hours(&self) -> Option<i64> {
        self.base().expire_after_hours
    }

    fn set_active(&mut self, active: bool);
}
