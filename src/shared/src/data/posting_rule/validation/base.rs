use crate::data::BasePostingRule;

pub struct BasePostingRuleValidator;

impl BasePostingRuleValidator {
    pub async fn validate(posting_rule: &BasePostingRule, issues: &mut Vec<String>) {
        if posting_rule.id.trim().is_empty() {
            issues.push("Id is empty".to_string());
        }

        if posting_rule.bot_id.trim().is_empty() {
            issues.push("BotId is empty".to_string());
        }

        if posting_rule.chat_id == 0 {
            issues.push("ChatId is empty".to_string());
        }

        if let Some(topic_id) = posting_rule.topic_id {
            if topic_id <= 0 {
                issues.push("TopicId is invalid".to_string());
            }
        }

        if posting_rule.name.trim().is_empty() {
            issues.push("Name is empty".to_string());
        }

        if let Some(description) = &posting_rule.description {
            if description.trim().is_empty() {
                issues.push("Description is empty".to_string());
            }
        }

        Self::validate_schedule(&posting_rule.schedule, issues);

        if posting_rule.timezone.trim().is_empty() {
            issues.push("Timezone is empty".to_string());
        }

        if let Some(ttl_hours) = posting_rule.ttl_hours {
            if ttl_hours <= 0 {
                issues.push("TtlHours is invalid".to_string());
            }
        }
    }

    fn validate_schedule(schedule: &str, issues: &mut Vec<String>) {
        if schedule.trim().is_empty() {
            issues.push("Schedule is empty".to_string());
            return;
        }

        let parts = schedule.split_whitespace().collect::<Vec<_>>();

        if parts.len() != 6 {
            issues.push("Schedule must have 6 parts".to_string());
        }
    }
}
