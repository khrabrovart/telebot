use telebot_shared::data::{PostingRule, PostingRuleTrait};
use teloxide::types::{InlineKeyboardButton, InlineKeyboardMarkup};

pub fn main_menu() -> InlineKeyboardMarkup {
    InlineKeyboardMarkup::new(vec![vec![InlineKeyboardButton::callback(
        "📋 Список правил",
        "list_rules",
    )]])
}

pub fn list_rules_menu(posting_rules: &[PostingRule]) -> InlineKeyboardMarkup {
    let mut buttons: Vec<Vec<InlineKeyboardButton>> = posting_rules
        .iter()
        .map(|posting_rule| {
            vec![InlineKeyboardButton::callback(
                posting_rule.name(),
                format!("rule_details:{}", posting_rule.id()),
            )]
        })
        .collect();

    buttons.push(vec![InlineKeyboardButton::callback(
        "⬅️ Назад",
        "back:list_rules",
    )]);

    InlineKeyboardMarkup::new(buttons)
}

pub fn rule_details_menu(posting_rule: &PostingRule) -> InlineKeyboardMarkup {
    let action = if posting_rule.is_active() {
        vec![InlineKeyboardButton::callback(
            "🔴 Выключить",
            format!("deactivate_rule:{}", posting_rule.id()),
        )]
    } else {
        vec![InlineKeyboardButton::callback(
            "🟢 Включить",
            format!("activate_rule:{}", posting_rule.id()),
        )]
    };

    InlineKeyboardMarkup::new(vec![
        action,
        vec![InlineKeyboardButton::callback("⬅️ Назад", "back:main_menu")],
    ])
}
