use crate::{formatter, TelegramBotClient};
use anyhow::Error;
use telebot_shared::{aws::DynamoDbClient, data::PostingRule};
use teloxide::{
    dispatching::dialogue::GetChatId,
    types::{InlineKeyboardButton, InlineKeyboardMarkup, Recipient, Update, UpdateKind},
};

pub async fn process_update(
    update: &Update,
    bot: &TelegramBotClient,
    db: &DynamoDbClient,
) -> Result<(), Error> {
    let chat_id: Recipient = update.chat_id().unwrap().as_user().unwrap().into();

    if let UpdateKind::Message(msg) = &update.kind {
        if let Some("/start") = msg.text() {
            bot.send_text_with_markup(chat_id.clone(), "Главное меню", &main_menu())
                .await?;
            return Ok(());
        } else {
            return Ok(());
        }
    }

    let query = match &update.kind {
        UpdateKind::CallbackQuery(q) => q,
        _ => return Ok(()),
    };

    let parts = query
        .data
        .as_deref()
        .unwrap()
        .split(':')
        .collect::<Vec<_>>();

    let command = parts[0];
    let params = &parts[1..];

    let message_id = query.message.as_ref().unwrap().id();

    match command {
        "list_rules" => {
            let posting_rules = db.get_all::<PostingRule>("posting_rules").await?;
            let filtered_rules: Vec<PostingRule> = posting_rules
                .iter()
                .filter(|rule| rule.bot_id == bot.bot_id)
                .cloned()
                .collect();

            bot.edit_message_text_with_markup(
                chat_id.clone(),
                message_id,
                "Список правил",
                &list_rules_menu(&filtered_rules),
            )
            .await?;
        }
        "rule_details" => {
            let rule_id = params[0];

            let posting_rule = db.get_item::<PostingRule>("posting_rules", rule_id).await?;

            let rule = match posting_rule {
                Some(r) => r,
                None => {
                    bot.send_text(chat_id.clone(), "Правило не найдено").await?;
                    return Ok(());
                }
            };

            let formatted_rule = formatter::format_rule(&rule);

            bot.edit_message_text_with_markup(
                chat_id.clone(),
                message_id,
                &formatted_rule,
                &rule_details_menu(),
            )
            .await?;
        }
        "back" => {
            let message_id = query.message.as_ref().unwrap().id();

            bot.edit_message_text_with_markup(
                chat_id.clone(),
                message_id,
                "Главное меню",
                &main_menu(),
            )
            .await?;
        }
        _ => return Ok(()),
    }

    Ok(())
}

fn main_menu() -> InlineKeyboardMarkup {
    InlineKeyboardMarkup::new(vec![vec![InlineKeyboardButton::callback(
        "Список правил",
        "list_rules",
    )]])
}

fn list_rules_menu(posting_rules: &[PostingRule]) -> InlineKeyboardMarkup {
    InlineKeyboardMarkup::new(vec![
        posting_rules
            .iter()
            .map(|rule| {
                InlineKeyboardButton::callback(rule.id.clone(), format!("rule_details:{}", rule.id))
            })
            .collect(),
        vec![InlineKeyboardButton::callback("< Назад", "back")],
    ])
}

fn rule_details_menu() -> InlineKeyboardMarkup {
    InlineKeyboardMarkup::new(vec![vec![InlineKeyboardButton::callback(
        "< Назад",
        "back",
    )]])
}
