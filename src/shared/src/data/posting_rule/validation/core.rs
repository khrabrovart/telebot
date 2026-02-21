use crate::data::{PostingRule, PostingRuleTrait};

use crate::data::posting_rule::validation::{
    base::BasePostingRuleValidator, PollPostingRuleValidator, TextPostingRuleValidator,
};

pub struct PostingRuleValidator;

impl PostingRuleValidator {
    pub fn validate(posting_rule: &PostingRule) -> Vec<String> {
        let mut issues = vec![];

        BasePostingRuleValidator::validate(posting_rule.base(), &mut issues);

        match posting_rule {
            PostingRule::Text(text_rule) => {
                TextPostingRuleValidator::validate(text_rule, &mut issues);
            }
            PostingRule::Poll(poll_rule) => {
                PollPostingRuleValidator::validate(poll_rule, &mut issues);
            }
        }

        issues
    }
}
