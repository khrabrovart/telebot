use crate::data::PollPostingRule;

pub struct PollPostingRuleValidator;

impl PollPostingRuleValidator {
    pub async fn validate(posting_rule: &PollPostingRule, issues: &mut Vec<String>) {
        if posting_rule.content.question.trim().is_empty() {
            issues.push("Question is empty".to_string());
        }

        if posting_rule.content.options.is_empty() {
            issues.push("Options are empty".to_string());
        } else {
            for (i, option) in posting_rule.content.options.iter().enumerate() {
                if option.trim().is_empty() {
                    issues.push(format!("Option {} is empty", i + 1));
                }
            }
        }
    }
}
