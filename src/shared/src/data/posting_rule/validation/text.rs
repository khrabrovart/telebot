use crate::data::TextPostingRule;

pub struct TextPostingRuleValidator;

impl TextPostingRuleValidator {
    pub async fn validate(posting_rule: &TextPostingRule, issues: &mut Vec<String>) {
        if posting_rule.content.text.trim().is_empty() {
            issues.push("Text is empty".to_string());
        }
    }
}
