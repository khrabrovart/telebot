use crate::date_utils;
use crate::TelegramBotClient;
use crate::REPLACEMENTS;
use lambda_runtime::{Error, LambdaEvent};
use telebot_shared::data::posting_rule::PollPostingRule;
use telebot_shared::{
    aws::DynamoDbClient,
    data::{
        BotData, PollActionLog, PollActionLogConfig, PollActionLogOutput, Post, PostContent,
        PostingRule, SchedulerEvent,
    },
    repositories::{PollActionLogRepository, PostRepository},
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

    if !posting_rule.is_valid() {
        return Err(format!("Posting rule is misconfigured: {}", posting_rule.base().id).into());
    }

    if !posting_rule.base().is_active {
        return Err(format!("Posting rule is not active: {}", posting_rule.base().id).into());
    }

    let bots_table_name = match std::env::var("BOTS_TABLE") {
        Ok(val) => val,
        Err(_) => {
            return Err("BOTS_TABLE environment variable not set".into());
        }
    };

    let bot_data = db
        .get_item::<BotData>(&bots_table_name, &posting_rule.base().bot_id)
        .await?;

    let bot_data = match bot_data {
        Some(data) => data,
        None => {
            return Err(format!("Bot data not found: {}", posting_rule.base().bot_id).into());
        }
    };

    info!(bot_id = %bot_data.id, "Bot data found");

    let bot = TelegramBotClient::new(&bot_data).await?;

    let post_repository = PostRepository::new(db.client.clone()).await?;

    post_message(&bot, &posting_rule, post_repository).await?;

    info!(post_id = %posting_rule.base().id, "Posting completed successfully");

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
    post_repository: PostRepository,
) -> Result<(), anyhow::Error> {
    let chat_id: Recipient = posting_rule.chat_id().into();
    let topic_id = posting_rule.topic_id();

    match posting_rule {
        PostingRule::Text(text_posting_rule) => {
            let text = replace_variables(&text_posting_rule.content.text);
            let message = bot.send_text(chat_id.clone(), topic_id, &text).await?;

            if text_posting_rule.base.should_pin {
                bot.pin_message(chat_id.clone(), message.id).await?;
            }

            info!("Message sent successfully, saving post to repository");

            let post = Post {
                chat_id: text_posting_rule.base.chat_id,
                topic_id: text_posting_rule.base.topic_id,
                message_id: message.id.0,
                bot_id: text_posting_rule.base.bot_id.clone(),
                posting_rule_id: text_posting_rule.base.id.clone(),
                content: PostContent::Text { text: text.clone() },
                schedule: text_posting_rule.base.schedule.clone(),
                timezone: text_posting_rule.base.timezone.clone(),
                is_pinned: text_posting_rule.base.should_pin,
                timestamp: message.date.timestamp(),
                expires_at: text_posting_rule
                    .base
                    .expire_after_hours
                    .map(date_utils::get_expiry_timestamp),
            };

            post_repository.put(&post).await?;

            Ok(())
        }
        PostingRule::Poll(poll_posting_rule) => {
            let question = replace_variables(&poll_posting_rule.content.question);
            let message = bot
                .send_poll(
                    chat_id.clone(),
                    topic_id,
                    &question,
                    &poll_posting_rule.content.options,
                )
                .await?;

            if poll_posting_rule.base.should_pin {
                bot.pin_message(chat_id.clone(), message.id).await?;
            }

            info!("Poll sent successfully, saving post to repository");

            let post = Post {
                chat_id: poll_posting_rule.base.chat_id,
                topic_id: poll_posting_rule.base.topic_id,
                message_id: message.id.0,
                bot_id: poll_posting_rule.base.bot_id.clone(),
                posting_rule_id: poll_posting_rule.base.id.clone(),
                content: PostContent::Poll {
                    question: question.clone(),
                    options: poll_posting_rule.content.options.clone(),
                },
                schedule: poll_posting_rule.base.schedule.clone(),
                timezone: poll_posting_rule.base.timezone.clone(),
                is_pinned: poll_posting_rule.base.should_pin,
                timestamp: message.date.timestamp(),
                expires_at: poll_posting_rule
                    .base
                    .expire_after_hours
                    .map(date_utils::get_expiry_timestamp),
            };

            post_repository.put(&post).await?;

            info!("Post saved successfully, checking if poll action log is enabled");

            match &poll_posting_rule.poll_action_log {
                Some(action_log) => {
                    info!(
                        "Poll action log enabled for posting rule {}, messages will be sent to chat {}",
                        poll_posting_rule.base.id, action_log.chat_id()
                    );

                    let poll_action_log_message =
                        post_poll_action_log_message(&question, action_log, bot, poll_posting_rule)
                            .await?;

                    create_poll_action_log(
                        message.poll().unwrap().id.clone(),
                        poll_posting_rule,
                        poll_action_log_message.id,
                        &question,
                        message.id.0,
                    )
                    .await?;
                }
                None => {
                    info!(
                        "Poll action log not enabled for posting rule {}, no messages will be sent",
                        poll_posting_rule.base.id
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
    poll_posting_rule: &PollPostingRule,
) -> Result<Message, anyhow::Error> {
    let chat_id: Recipient = action_log.chat_id().into();
    let topic_id = action_log.topic_id();

    let output_description = match poll_posting_rule.poll_action_log.as_ref().unwrap().output {
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
    poll_posting_rule: &PollPostingRule,
    action_log_message_id: MessageId,
    text: &str,
    message_id: i32,
) -> Result<(), anyhow::Error> {
    let poll_action_log_repository = PollActionLogRepository::new().await?;

    // TODO: Move struct creating to their own functions new()

    let poll_action_log = PollActionLog {
        id: poll_id.to_string(),
        chat_id: poll_posting_rule.base.chat_id,
        topic_id: poll_posting_rule.base.topic_id,
        message_id,
        action_log_message_id: action_log_message_id.0,
        posting_rule_id: poll_posting_rule.base.id.to_string(),
        text: text.to_string(),
        records: vec![],
        timezone: poll_posting_rule.base.timezone.clone(),
        expires_at: poll_posting_rule
            .base
            .expire_after_hours
            .map(date_utils::get_expiry_timestamp),
        version: 0,
    };

    poll_action_log_repository.put(&poll_action_log).await?;

    Ok(())
}
