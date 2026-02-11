use crate::telegram::TelegramBotClient;
use crate::REPLACEMENTS;
use lambda_runtime::{Error, LambdaEvent};
use telebot_shared::{
    aws::DynamoDbClient,
    data::{BotData, PostingRule, PostingRuleContent, SchedulerEvent},
};
use teloxide::types::Recipient;
use tracing::info;

pub async fn handle(event: LambdaEvent<SchedulerEvent>) -> Result<(), Error> {
    let (payload, _context) = event.into_parts();

    info!(posting_rule_id = %payload.posting_rule_id, "Received event");

    let posting_rules_table_name = match std::env::var("POSTING_RULES_TABLE") {
        Ok(val) => val,
        Err(_) => {
            return Err("POSTING_RULES_TABLE environment variable not set".into());
        }
    };

    let db = DynamoDbClient::new().await;
    let posting_rule = db
        .get_item::<PostingRule>(&posting_rules_table_name, &payload.posting_rule_id)
        .await?;

    let posting_rule = match posting_rule {
        Some(rule) => rule,
        None => {
            return Err(format!("Posting rule not found: {}", payload.posting_rule_id).into());
        }
    };

    info!(
        post_id = %posting_rule.id,
        content = ?posting_rule.content,
        is_active = posting_rule.is_active,
        is_ready = posting_rule.is_ready,
        "Posting rule found"
    );

    if !posting_rule.is_valid() {
        return Err(format!("Posting rule is misconfigured: {}", posting_rule.id).into());
    }

    if !posting_rule.is_active {
        return Err(format!("Posting rule is not active: {}", posting_rule.id).into());
    }

    let bots_table_name = match std::env::var("BOTS_TABLE") {
        Ok(val) => val,
        Err(_) => {
            return Err("BOTS_TABLE environment variable not set".into());
        }
    };

    let bot_data = db
        .get_item::<BotData>(&bots_table_name, &posting_rule.bot_id)
        .await?;

    let bot_data = match bot_data {
        Some(data) => data,
        None => {
            return Err(format!("Bot data not found: {}", posting_rule.bot_id).into());
        }
    };

    info!(bot_id = %bot_data.id, "Bot data found");

    let bot = TelegramBotClient::new(&bot_data).await?;
    post(&bot, &posting_rule).await?;

    info!(post_id = %posting_rule.id, "Posting completed successfully");

    Ok(())
}

fn replace_variables(text: &str) -> String {
    let mut result = text.to_string();
    for (key, func) in REPLACEMENTS.iter() {
        result = result.replace(key, &func());
    }
    result
}

async fn post(bot: &TelegramBotClient, posting_rule: &PostingRule) -> Result<(), anyhow::Error> {
    let chat_id: Recipient = posting_rule.chat_id.clone().into();

    match &posting_rule.content {
        PostingRuleContent::Text { text } => {
            let replaced_text = replace_variables(text);
            bot.send_text(chat_id, &replaced_text).await
        }
        PostingRuleContent::Poll { question, options } => {
            let replaced_question = replace_variables(question);
            bot.send_poll(chat_id, &replaced_question, options).await
        }
    }
}
