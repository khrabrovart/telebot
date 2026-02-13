use crate::{formatter, TelegramBotClient};
use anyhow::{anyhow, Error};
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
            bot.send_text_with_markup(chat_id.clone(), "🏠 Главное меню", &main_menu())
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

    bot.answer_callback_query(query.id.clone()).await?;

    let parts = query
        .data
        .as_deref()
        .unwrap()
        .split(':')
        .collect::<Vec<_>>();

    let command = parts[0];
    let params = &parts[1..];

    let message_id = query.message.as_ref().unwrap().id();

    let posting_rules_table_name = std::env::var("POSTING_RULES_TABLE")
        .map_err(|_| anyhow!("POSTING_RULES_TABLE environment variable not set"))?;

    match command {
        "list_rules" => {
            let posting_rules = db.get_all::<PostingRule>(&posting_rules_table_name).await?;
            let filtered_rules: Vec<PostingRule> = posting_rules
                .iter()
                .filter(|posting_rule| posting_rule.bot_id == bot.bot_id)
                .cloned()
                .collect();

            bot.edit_message_text_with_markup(
                chat_id.clone(),
                message_id,
                "📋 Список правил",
                &list_rules_menu(&filtered_rules),
            )
            .await?;
        }
        "rule_details" => {
            let posting_rule_id = params[0];

            let posting_rule = db
                .get_item::<PostingRule>(&posting_rules_table_name, posting_rule_id)
                .await?;

            let posting_rule = match posting_rule {
                Some(r) => r,
                None => {
                    bot.send_text(chat_id.clone(), "Правило не найдено").await?;
                    return Ok(());
                }
            };

            let posting_rules_chat_id: Recipient = posting_rule.chat_id.clone().into();
            let chat_name = bot.get_chat_title(posting_rules_chat_id).await?;

            let formatted_rule = formatter::format_rule(&posting_rule, &chat_name);

            bot.edit_message_text_with_markup(
                chat_id.clone(),
                message_id,
                &formatted_rule,
                &rule_details_menu(&posting_rule),
            )
            .await?;
        }
        "activate_rule" => {
            let posting_rule_id = params[0];

            let posting_rule = db
                .get_item::<PostingRule>(&posting_rules_table_name, posting_rule_id)
                .await?;

            let mut posting_rule = match posting_rule {
                Some(r) => r,
                None => {
                    bot.send_text(chat_id.clone(), "Правило не найдено").await?;
                    return Ok(());
                }
            };

            posting_rule.is_active = true;

            db.put_item(&posting_rules_table_name, &posting_rule)
                .await?;

            let posting_rules_chat_id: Recipient = posting_rule.chat_id.clone().into();
            let chat_name = bot.get_chat_title(posting_rules_chat_id).await?;

            let formatted_rule = formatter::format_rule(&posting_rule, &chat_name);

            bot.edit_message_text_with_markup(
                chat_id.clone(),
                message_id,
                &formatted_rule,
                &rule_details_menu(&posting_rule),
            )
            .await?;
        }
        "deactivate_rule" => {
            let posting_rule_id = params[0];

            let posting_rule = db
                .get_item::<PostingRule>(&posting_rules_table_name, posting_rule_id)
                .await?;

            let mut posting_rule = match posting_rule {
                Some(r) => r,
                None => {
                    bot.send_text(chat_id.clone(), "Правило не найдено").await?;
                    return Ok(());
                }
            };

            posting_rule.is_active = false;

            db.put_item(&posting_rules_table_name, &posting_rule)
                .await?;

            let posting_rules_chat_id: Recipient = posting_rule.chat_id.clone().into();
            let chat_name = bot.get_chat_title(posting_rules_chat_id).await?;

            let formatted_rule = formatter::format_rule(&posting_rule, &chat_name);

            bot.edit_message_text_with_markup(
                chat_id.clone(),
                message_id,
                &formatted_rule,
                &rule_details_menu(&posting_rule),
            )
            .await?;
        }
        "back" => {
            let message_id = query.message.as_ref().unwrap().id();

            bot.edit_message_text_with_markup(
                chat_id.clone(),
                message_id,
                "🏠 Главное меню",
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
        "📋 Список правил",
        "list_rules",
    )]])
}

fn list_rules_menu(posting_rules: &[PostingRule]) -> InlineKeyboardMarkup {
    let mut buttons: Vec<Vec<InlineKeyboardButton>> = posting_rules
        .iter()
        .map(|posting_rule| {
            vec![InlineKeyboardButton::callback(
                posting_rule.name.clone(),
                format!("rule_details:{}", posting_rule.id),
            )]
        })
        .collect();

    buttons.push(vec![InlineKeyboardButton::callback("⬅️ Назад", "back")]);

    InlineKeyboardMarkup::new(buttons)
}

fn rule_details_menu(posting_rule: &PostingRule) -> InlineKeyboardMarkup {
    let action = if posting_rule.is_active {
        vec![InlineKeyboardButton::callback(
            "🔴 Выключить",
            format!("deactivate_rule:{}", posting_rule.id),
        )]
    } else {
        vec![InlineKeyboardButton::callback(
            "🟢 Включить",
            format!("activate_rule:{}", posting_rule.id),
        )]
    };

    InlineKeyboardMarkup::new(vec![
        action,
        vec![InlineKeyboardButton::callback("⬅️ Назад", "back")],
    ])
}
