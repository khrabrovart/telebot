use crate::TelegramBotClient;
use crate::REPLACEMENTS;
use lambda_runtime::{Error, LambdaEvent};
use telebot_shared::{
    aws::DynamoDbClient,
    data::{
        BotData, PollActionLog, PollActionLogConfig, PollActionLogOutput, Post, PostContent,
        PostingRule, PostingRuleContent, SchedulerEvent,
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

    let post_repository = PostRepository::new(db.client.clone()).await?;

    post_message(&bot, &posting_rule, post_repository).await?;

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
    post_repository: PostRepository,
) -> Result<(), anyhow::Error> {
    let chat_id: Recipient = posting_rule.chat_id().into();
    let topic_id = posting_rule.topic_id();

    match &posting_rule.content {
        PostingRuleContent::Text { text } => {
            let text = replace_variables(text);
            let message = bot.send_text(chat_id.clone(), topic_id, &text).await?;

            if posting_rule.should_pin {
                bot.pin_message(chat_id.clone(), message.id).await?;
            }

            let post = Post {
                chat_id: posting_rule.chat_id(),
                topic_id: posting_rule.topic_id(),
                message_id: message.id,
                bot_id: posting_rule.bot_id.clone(),
                name: posting_rule.name.clone(),
                content: PostContent::Text { text: text.clone() },
                schedule: posting_rule.schedule.clone(),
                timezone: posting_rule.timezone.clone(),
                is_pinned: posting_rule.should_pin,
                timestamp: message.date.timestamp(),
            };

            post_repository.put(&post).await?;

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

            let post = Post {
                chat_id: posting_rule.chat_id(),
                topic_id: posting_rule.topic_id(),
                message_id: message.id,
                bot_id: posting_rule.bot_id.clone(),
                name: posting_rule.name.clone(),
                content: PostContent::Poll {
                    question: question.clone(),
                    options: options.clone(),
                },
                schedule: posting_rule.schedule.clone(),
                timezone: posting_rule.timezone.clone(),
                is_pinned: posting_rule.should_pin,
                timestamp: message.date.timestamp(),
            };

            post_repository.put(&post).await?;

            match &posting_rule.poll_action_log {
                Some(action_log) => {
                    info!(
                        "Poll action log enabled for posting rule {}, messages will be sent to chat {}",
                        posting_rule.id, action_log.chat_id()
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
    let chat_id: Recipient = action_log.chat_id().into();
    let topic_id = action_log.topic_id();

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
