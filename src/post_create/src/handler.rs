use crate::TelegramBotClient;
use crate::REPLACEMENTS;
use lambda_runtime::{Error, LambdaEvent};
use std::collections::HashMap;
use telebot_shared::{
    aws::DynamoDbClient,
    data::{
        BotDataRepository, PollActionLog, PollActionLogRecord, PollActionLogRepository, PollPost,
        PollPostingRule, PollPostingRuleActionLog, PollPostingRuleActionLogOutput,
        PollPostingRuleOptionIntersectionSource, PollPostingRuleOptionSource, Post, PostRepository,
        PostingRule, PostingRuleRepository, PostingRuleTrait, SchedulerEvent, TextPost,
    },
};
use teloxide::types::{Message, Recipient};
use tracing::{error, info, warn};

// TODO: Split this handler into multiple smaller functions and move them into separate modules for better readability and maintainability

pub async fn handle(event: LambdaEvent<SchedulerEvent>) -> Result<(), Error> {
    let (payload, _context) = event.into_parts();

    info!(posting_rule_id = %payload.posting_rule_id, "Received event");

    let db = DynamoDbClient::new().await;

    let posting_rule_repository = PostingRuleRepository::new(db.client.clone()).await?;

    let posting_rule = match posting_rule_repository
        .get(&payload.posting_rule_id)
        .await?
    {
        Some(rule) => rule,
        None => {
            return Err(format!("Posting rule not found: {}", payload.posting_rule_id).into());
        }
    };

    if !posting_rule.is_valid() {
        return Err(format!("Posting rule is misconfigured: {}", posting_rule.id()).into());
    }

    if !posting_rule.is_active() {
        return Err(format!("Posting rule is not active: {}", posting_rule.id()).into());
    }

    let bot_data_repository = BotDataRepository::new(&db).await?;

    let bot_data = match bot_data_repository.get(posting_rule.bot_id()).await? {
        Some(data) => data,
        None => {
            return Err(format!("Bot data not found: {}", posting_rule.bot_id()).into());
        }
    };

    info!(bot_id = %bot_data.id, "Bot data found");

    let bot = TelegramBotClient::new(&bot_data).await?;

    let post_repository = PostRepository::new(db.client.clone()).await?;
    let poll_action_log_repository = PollActionLogRepository::new(db.client.clone()).await?;

    post_message(
        &bot,
        &posting_rule,
        &post_repository,
        &poll_action_log_repository,
        &db,
    )
    .await?;

    info!(post_id = %posting_rule.id(), "Posting completed successfully");

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
    post_repository: &PostRepository,
    poll_action_log_repository: &PollActionLogRepository,
    db: &DynamoDbClient,
) -> Result<(), anyhow::Error> {
    let chat_id: Recipient = posting_rule.chat_id().into();
    let topic_id = posting_rule.topic_id();

    match posting_rule {
        PostingRule::Text(text_posting_rule) => {
            let text = replace_variables(&text_posting_rule.content.text);
            let message = bot.send_text(chat_id.clone(), topic_id, &text).await?;

            if text_posting_rule.should_pin() {
                bot.pin_message(chat_id.clone(), message.id).await?;
            }

            info!("Message sent successfully, saving post to repository");

            let text_post = TextPost::new(
                text_posting_rule,
                message.id.0,
                message.date.timestamp(),
                &text,
            );

            post_repository.put(&Post::Text(text_post)).await?;

            Ok(())
        }
        PostingRule::Poll(poll_posting_rule) => {
            let question = replace_variables(&poll_posting_rule.content.question);

            let mut options: Vec<String> = vec![];

            if let Some(source) = &poll_posting_rule.content.source {
                let sourced_options =
                    get_sourced_poll_options(source, post_repository, poll_action_log_repository)
                        .await?;

                if sourced_options.is_empty() {
                    warn!(
                        posting_rule_id = %poll_posting_rule.id(),
                        "No options found from source, poll will not be posted"
                    );
                    return Ok(());
                }

                options.extend(sourced_options.iter().map(|s| s.to_string()));
            }

            options.extend(
                poll_posting_rule
                    .content
                    .options
                    .iter()
                    .map(|s| s.to_string()),
            );

            let message = bot
                .send_poll(chat_id.clone(), topic_id, &question, &options[..])
                .await?;

            if poll_posting_rule.should_pin() {
                bot.pin_message(chat_id.clone(), message.id).await?;
            }

            info!("Poll sent successfully, saving post to repository");

            let poll_post = PollPost::new(
                poll_posting_rule,
                message.id.0,
                message.date.timestamp(),
                &question,
                &options,
            );
            post_repository.put(&Post::Poll(poll_post)).await?;

            info!("Post saved successfully, checking if poll action log is enabled");

            match &poll_posting_rule.action_log {
                Some(poll_posting_rule_action_log) => {
                    info!(
                        "Poll action log enabled for posting rule {}, messages will be sent to chat {}",
                        poll_posting_rule.id(), poll_posting_rule_action_log.chat_id()
                    );

                    let poll_action_log_message = post_poll_action_log_message(
                        &question,
                        poll_posting_rule_action_log,
                        bot,
                        poll_posting_rule,
                    )
                    .await?;

                    create_poll_action_log(
                        message,
                        poll_action_log_message,
                        poll_posting_rule,
                        poll_posting_rule_action_log,
                        db,
                    )
                    .await?;
                }
                None => {
                    info!(
                        "Poll action log not enabled for posting rule {}, no messages will be sent",
                        poll_posting_rule.id()
                    );
                }
            }

            Ok(())
        }
    }
}

async fn post_poll_action_log_message(
    message_text: &str,
    poll_posting_rule_action_log: &PollPostingRuleActionLog,
    bot: &TelegramBotClient,
    poll_posting_rule: &PollPostingRule,
) -> Result<Message, anyhow::Error> {
    let chat_id: Recipient = poll_posting_rule_action_log.chat_id().into();
    let topic_id = poll_posting_rule_action_log.topic_id();

    let output_description = match poll_posting_rule.action_log.as_ref().unwrap().output {
        PollPostingRuleActionLogOutput::All => "Отображаются все действия".to_string(),
        PollPostingRuleActionLogOutput::OnlyWhenTargetOptionRevoked {
            target_option_id: _,
        } => "Отображаются только действия после изменения голоса с целевой опции".to_string(),
    };

    let text = format!(
        "<b>Лог событий голосования</b>\n{}\n\n{}\n\n{}\n\n<i>Здесь будут отображаться действия с данным голосованием</i>",
        poll_posting_rule.name(),
        output_description,
        message_text
    );

    let message = bot.send_text(chat_id, topic_id, &text).await?;

    Ok(message)
}

async fn create_poll_action_log(
    message: Message,
    poll_action_log_message: Message,
    poll_posting_rule: &PollPostingRule,
    poll_posting_rule_action_log: &PollPostingRuleActionLog,
    db: &DynamoDbClient,
) -> Result<(), anyhow::Error> {
    let poll_action_log_repository = PollActionLogRepository::new(db.client.clone()).await?;

    let poll_id = message.poll().unwrap().id.clone();
    let message_id = message.id;
    let poll_action_log_message_id = poll_action_log_message.id;

    let poll_action_log = PollActionLog::new(
        poll_posting_rule,
        poll_posting_rule_action_log,
        poll_id,
        message_id,
        poll_action_log_message_id,
    );

    poll_action_log_repository.put(&poll_action_log).await?;

    Ok(())
}

async fn get_sourced_poll_options(
    source: &PollPostingRuleOptionSource,
    post_repository: &PostRepository,
    poll_action_log_repository: &PollActionLogRepository,
) -> Result<Vec<String>, anyhow::Error> {
    match source {
        PollPostingRuleOptionSource::Intersection(intersection_source) => {
            get_intersection_sourced_poll_options(
                intersection_source,
                post_repository,
                poll_action_log_repository,
            )
            .await
        }
    }
}

async fn get_intersection_sourced_poll_options(
    source: &PollPostingRuleOptionIntersectionSource,
    post_repository: &PostRepository,
    poll_action_log_repository: &PollActionLogRepository,
) -> Result<Vec<String>, anyhow::Error> {
    let recent_post = match post_repository
        .get_most_recent_by_posting_rule(&source.source_posting_rule_id)
        .await?
    {
        Some(Post::Poll(poll_post)) => poll_post,
        Some(Post::Text(_)) => {
            error!(
                posting_rule_id = %source.source_posting_rule_id,
                "Found post is not a poll"
            );
            return Ok(Vec::new());
        }
        None => {
            error!(
                posting_rule_id = %source.source_posting_rule_id,
                "No recent post found for source posting rule"
            );
            return Ok(Vec::new());
        }
    };

    let action_log = match poll_action_log_repository
        .get_by_chat_and_message(recent_post.base.chat_id, recent_post.base.message_id)
        .await?
    {
        Some(log) => log,
        None => {
            error!(
                chat_id = recent_post.base.chat_id,
                message_id = recent_post.base.message_id,
                "No action log found for poll post"
            );
            return Ok(Vec::new());
        }
    };

    let mut latest_records: HashMap<u64, PollActionLogRecord> = HashMap::new();

    for record in &action_log.records {
        if record.option_id == Some(source.target_option_id) {
            latest_records
                .entry(record.actor_id)
                .and_modify(|existing| {
                    if record.timestamp > existing.timestamp {
                        *existing = record.clone();
                    }
                })
                .or_insert_with(|| record.clone());
        }
    }

    let actor_ids: Vec<u64> = latest_records.keys().copied().collect();

    if actor_ids.is_empty() {
        info!(
            target_option_id = source.target_option_id,
            "No actors found voting for target option"
        );
        return Ok(Vec::new());
    }

    let matching_actors = source.voter_ids.iter().find_map(|voter_group| {
        let intersection: Vec<u64> = actor_ids
            .iter()
            .filter(|id| voter_group.contains(id))
            .copied()
            .collect();

        if !intersection.is_empty() {
            Some(intersection)
        } else {
            None
        }
    });

    let matching_actors = match matching_actors {
        Some(actors) => actors,
        None => {
            info!(
                target_option_id = source.target_option_id,
                "No actors found in voter groups"
            );
            return Ok(Vec::new());
        }
    };

    let result: Vec<String> = matching_actors
        .into_iter()
        .filter_map(|actor_id| {
            latest_records.get(&actor_id).map(|record| {
                let last_name = record
                    .actor_last_name
                    .as_deref()
                    .map_or("".to_string(), |last_name| format!(" {}", last_name));

                if last_name.is_empty() {
                    record.actor_first_name.clone()
                } else {
                    format!("{}{}", record.actor_first_name, last_name)
                }
            })
        })
        .collect();

    info!(
        target_option_id = source.target_option_id,
        matched_count = result.len(),
        "Found matched actor names for intersection"
    );

    Ok(result)
}
