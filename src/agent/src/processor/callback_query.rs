use crate::{
    formatter,
    processor::{access_validator, menus},
    TelegramBotClient,
};
use anyhow::Error;
use telebot_shared::{
    aws::DynamoDbClient,
    data::{BotData, PostingRule, PostingRuleRepository, PostingRuleTrait},
};
use teloxide::{
    dispatching::dialogue::GetChatId,
    types::{CallbackQuery, Recipient, Update},
};

pub async fn process(
    callback_query: &CallbackQuery,
    update: &Update,
    bot: &TelegramBotClient,
    bot_data: &BotData,
    db: &DynamoDbClient,
) -> Result<(), Error> {
    let chat_id: Recipient = match update.chat_id().unwrap().as_user() {
        Some(user) => user.into(),
        None => {
            return Ok(());
        }
    };

    match access_validator::validate_access(update, chat_id.clone(), bot_data, bot).await? {
        true => (),
        false => return Ok(()),
    }

    let parts = callback_query
        .data
        .as_deref()
        .unwrap()
        .split(':')
        .collect::<Vec<_>>();

    let command = parts[0];
    let params = &parts[1..];

    let message_id = callback_query.message.as_ref().unwrap().id();

    let posting_rule_repository = PostingRuleRepository::new(db.client.clone()).await?;

    match command {
        "list_rules" => {
            let posting_rules = posting_rule_repository.get_all().await?;
            let mut filtered_rules: Vec<PostingRule> = posting_rules
                .iter()
                .filter(|posting_rule| posting_rule.bot_id() == bot.bot_id)
                .cloned()
                .collect();

            filtered_rules.sort_by(|a, b| a.name().cmp(&b.name()));

            bot.edit_message_text_with_markup(
                chat_id.clone(),
                message_id,
                "📋 Список правил",
                &menus::list_rules_menu(&filtered_rules),
            )
            .await?;

            bot.answer_callback_query(callback_query.id.clone()).await?;
        }
        "rule_details" => {
            let posting_rule_id = params[0];

            let posting_rule = posting_rule_repository.get(posting_rule_id).await?;

            let posting_rule = match posting_rule {
                Some(r) => r,
                None => {
                    bot.send_text(chat_id.clone(), "Правило не найдено").await?;
                    return Ok(());
                }
            };

            let posting_rules_chat_id: Recipient = posting_rule.chat_id().into();
            let chat_name = bot.get_chat_title(posting_rules_chat_id).await?;

            let formatted_rule = formatter::format_rule(&posting_rule, &chat_name);

            bot.edit_message_text_with_markup(
                chat_id.clone(),
                message_id,
                &formatted_rule,
                &menus::rule_details_menu(&posting_rule),
            )
            .await?;

            bot.answer_callback_query(callback_query.id.clone()).await?;
        }
        "activate_rule" => {
            let posting_rule_id = params[0];
            let posting_rule = posting_rule_repository.get(posting_rule_id).await?;

            let mut posting_rule = match posting_rule {
                Some(r) => r,
                None => {
                    bot.send_text(chat_id.clone(), "Правило не найдено").await?;
                    return Ok(());
                }
            };

            posting_rule.set_active(true);

            posting_rule_repository.put_item(&posting_rule).await?;

            let posting_rules_chat_id: Recipient = posting_rule.chat_id().into();
            let chat_name = bot.get_chat_title(posting_rules_chat_id).await?;

            let formatted_rule = formatter::format_rule(&posting_rule, &chat_name);

            bot.edit_message_text_with_markup(
                chat_id.clone(),
                message_id,
                &formatted_rule,
                &menus::rule_details_menu(&posting_rule),
            )
            .await?;

            bot.answer_callback_query(callback_query.id.clone()).await?;
        }
        "deactivate_rule" => {
            let posting_rule_id = params[0];
            let posting_rule = posting_rule_repository.get(posting_rule_id).await?;

            let mut posting_rule = match posting_rule {
                Some(r) => r,
                None => {
                    bot.send_text(chat_id.clone(), "Правило не найдено").await?;
                    return Ok(());
                }
            };

            posting_rule.set_active(false);

            posting_rule_repository.put_item(&posting_rule).await?;

            let posting_rules_chat_id: Recipient = posting_rule.chat_id().into();
            let chat_name = bot.get_chat_title(posting_rules_chat_id).await?;

            let formatted_rule = formatter::format_rule(&posting_rule, &chat_name);

            bot.edit_message_text_with_markup(
                chat_id.clone(),
                message_id,
                &formatted_rule,
                &menus::rule_details_menu(&posting_rule),
            )
            .await?;

            bot.answer_callback_query(callback_query.id.clone()).await?;
        }
        "back" => {
            let message_id = callback_query.message.as_ref().unwrap().id();

            bot.edit_message_text_with_markup(
                chat_id.clone(),
                message_id,
                "🏠 Главное меню",
                &menus::main_menu(),
            )
            .await?;

            bot.answer_callback_query(callback_query.id.clone()).await?;
        }
        _ => return Ok(()),
    }

    Ok(())
}
