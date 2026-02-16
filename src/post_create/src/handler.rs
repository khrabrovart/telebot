use crate::telegram::TelegramBotClient;
use crate::REPLACEMENTS;
use lambda_runtime::{Error, LambdaEvent};
use telebot_shared::{
    aws::DynamoDbClient,
    data::{
        posting_rule::{PollActionLogConfig, PollActionLogOutput},
        BotData, PollActionLog, PostingRule,
        PostingRuleContent::{self},
        SchedulerEvent,
    },
    repositories::PollActionLogRepository,
};
use teloxide::types::{Message, MessageId, PollId, Recipient};
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
    post_message(&bot, &posting_rule).await?;

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

async fn post_message(
    bot: &TelegramBotClient,
    posting_rule: &PostingRule,
) -> Result<(), anyhow::Error> {
    let chat_id: Recipient = posting_rule.chat_id.clone().into();
    let topic_id = posting_rule.topic_id.clone();

    match &posting_rule.content {
        PostingRuleContent::Text { text } => {
            let text = replace_variables(text);
            let message = bot.send_text(chat_id.clone(), topic_id, &text).await?;

            if posting_rule.should_pin {
                bot.pin_message(chat_id.clone(), message.id).await?;
            }

            Ok(())
        }
        PostingRuleContent::Poll { question, options } => {
            let question = replace_variables(question);
            let message = bot
                .send_poll(chat_id.clone(), topic_id, &question, options)
                .await?;

            if posting_rule.should_pin {
                bot.pin_message(chat_id.clone(), message.id).await?;
            }

            match &posting_rule.poll_action_log {
                Some(action_log) => {
                    info!(
                        "Poll action log enabled for posting rule {}, messages will be sent to chat {}",
                        posting_rule.id, action_log.chat_id
                    );

                    let poll_action_log_message =
                        post_poll_action_log_message(&question, action_log, bot, posting_rule)
                            .await?;

                    create_poll_action_log(
                        message.poll().unwrap().id.clone(),
                        posting_rule,
                        poll_action_log_message.id,
                        &question,
                    )
                    .await?;
                }
                None => {
                    info!(
                        "Poll action log not enabled for posting rule {}, no messages will be sent",
                        posting_rule.id
                    );
                }
            }

            Ok(())
        }
    }
}

async fn post_poll_action_log_message(
    message_text: &str,
    action_log: &PollActionLogConfig,
    bot: &TelegramBotClient,
    posting_rule: &PostingRule,
) -> Result<Message, anyhow::Error> {
    let chat_id: Recipient = action_log.chat_id.clone().into();
    let topic_id = action_log.topic_id.clone();

    let output_description = match posting_rule.poll_action_log.as_ref().unwrap().output {
        PollActionLogOutput::All => "Отображаются все действия".to_string(),
        PollActionLogOutput::OnlyWhenTargetOptionRevoked {
            target_option_id: _,
        } => "Отображаются только действия после изменения голоса с целевой опции".to_string(),
    };

    let text = format!(
            "<b>Лог событий опроса</b>\n{}\n\n{}\n\n<i>Здесь будут отображаться действия с данным опросом</i>",
            output_description, message_text
        );

    let message = bot.send_text(chat_id, topic_id, &text).await?;

    Ok(message)
}

async fn create_poll_action_log(
    poll_id: PollId,
    posting_rule: &PostingRule,
    action_log_message_id: MessageId,
    text: &str,
) -> Result<(), anyhow::Error> {
    let poll_action_log_repository = PollActionLogRepository::new().await?;

    let poll_action_log = PollActionLog {
        id: poll_id.to_string(),
        posting_rule_id: posting_rule.id.to_string(),
        action_log_message_id: action_log_message_id.0,
        text: text.to_string(),
        records: vec![],
        timezone: posting_rule.timezone.clone(),
        version: 0,
    };

    poll_action_log_repository.put(&poll_action_log).await?;

    Ok(())
}
