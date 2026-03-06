use crate::TelegramBotClient;
use anyhow::{anyhow, Error};
use chrono_tz::Tz;
use std::collections::HashMap;
use telebot_shared::{
    aws::DynamoDbClient,
    data::{
        PollActionLog, PollActionLogOutput, PollActionLogRecord, PollActionLogRepository, PollPost,
        Post, PostRepository, PostTrait,
    },
};
use teloxide::types::{PollAnswer, Recipient, Update};

pub async fn process(
    poll_answer: &PollAnswer,
    update: &Update,
    bot: &TelegramBotClient,
    db: &DynamoDbClient,
) -> Result<(), Error> {
    let poll_action_log_repository = PollActionLogRepository::new(db.client.clone()).await?;
    let post_repository = PostRepository::new(db.client.clone()).await?;

    let action_log = poll_action_log_repository
        .get(&poll_answer.poll_id.to_string())
        .await?;

    let action_log = match action_log {
        Some(log) => log,
        None => {
            return Ok(());
        }
    };

    let post = post_repository
        .get(action_log.chat_id, action_log.message_id)
        .await?;

    let post = match post {
        Some(post) => post,
        None => {
            return Err(anyhow!(
                "Post not found, chat_id: {}, message_id: {}",
                action_log.chat_id,
                action_log.message_id
            ));
        }
    };

    let poll_post = match post {
        Post::Poll(poll_post) => poll_post,
        _ => {
            return Err(anyhow!(
                "Associated post is not a poll, chat_id: {}, message_id: {}",
                action_log.chat_id,
                action_log.message_id
            ));
        }
    };

    let poll_options = &poll_post.content.options;

    let (option_id, option_text) = if !poll_answer.option_ids.is_empty() {
        let option_id = poll_answer.option_ids[0];
        let option_text = poll_options[option_id as usize].clone();
        (Some(option_id as i32), Some(option_text))
    } else {
        (None, None)
    };

    // TODO: Check if the record with the same update_id elready exists and omit adding a new one in this case, to prevent duplicates when receiving the same update multiple times due to network issues or other reasons

    let action_record = PollActionLogRecord::new(
        update.id,
        poll_answer.voter.user().unwrap(),
        option_id,
        option_text,
    );

    let mut updated_action_log = action_log.clone();
    updated_action_log.records.push(action_record);

    poll_action_log_repository.put(&updated_action_log).await?;

    update_action_log_message(&updated_action_log, &poll_post, bot).await?;

    Ok(())
}

async fn update_action_log_message(
    action_log: &PollActionLog,
    poll_post: &PollPost,
    bot: &TelegramBotClient,
) -> Result<(), Error> {
    let mut grouped_records: HashMap<u64, Vec<PollActionLogRecord>> = HashMap::new();

    for record in action_log.records.iter() {
        grouped_records
            .entry(record.actor_id)
            .or_default()
            .push(record.clone());
    }

    let mut filtered_records: HashMap<u64, Vec<PollActionLogRecord>> = HashMap::new();

    match action_log.output {
        PollActionLogOutput::All => {
            filtered_records = grouped_records;
        }
        PollActionLogOutput::OnlyWhenTargetOptionRevoked { target_option_id } => {
            for (actor_id, records) in grouped_records.into_iter() {
                let mut target_option_timestamp: Option<i64> = None;
                let mut target_option_revoked: bool = false;

                for record in records.into_iter() {
                    if target_option_revoked {
                        filtered_records
                            .entry(actor_id)
                            .or_default()
                            .push(record.clone());

                        continue;
                    }

                    if record.option_id == Some(target_option_id) {
                        target_option_timestamp = Some(record.timestamp);
                    } else if let Some(timestamp) = target_option_timestamp {
                        if record.timestamp > timestamp {
                            target_option_revoked = true;
                            filtered_records.insert(actor_id, vec![record.clone()]);
                        }
                    }
                }
            }
        }
    }

    let mut actor_ids: Vec<_> = filtered_records.keys().copied().collect();
    actor_ids.sort();

    let mut records_text = actor_ids
        .iter()
        .map(|actor_id| {
            let records = &filtered_records[actor_id];
            let last_name = records[0]
                .actor_last_name
                .clone()
                .map_or("".to_string(), |ln| format!(" {}", ln));

            let username = records[0]
                .actor_username
                .clone()
                .map_or("".to_string(), |un| format!(" (@{})", un));

            let actor_name = format!(
                "<b>{}{}{}</b>",
                records[0].actor_first_name, last_name, username
            );

            let actions_list = records
                .iter()
                .map(|record| {
                    let tz: Tz = action_log.timezone.parse().unwrap();
                    let date = chrono::DateTime::from_timestamp(record.timestamp, 0)
                        .unwrap()
                        .with_timezone(&tz)
                        .format("%d.%m.%Y %H:%M:%S");

                    format!(
                        "{} → {}",
                        date,
                        record
                            .option_text
                            .clone()
                            .unwrap_or("<b>Голос отозван</b>".to_string())
                    )
                })
                .collect::<Vec<String>>()
                .join("\n");

            format!("{}\n{}", actor_name, actions_list)
        })
        .collect::<Vec<String>>()
        .join("\n\n");

    if records_text.is_empty() {
        records_text = "<i>Здесь будут отображаться действия с данным голосованием</i>".to_string();
    }

    let output_description = match action_log.output {
        PollActionLogOutput::All => "Отображаются все действия".to_string(),
        PollActionLogOutput::OnlyWhenTargetOptionRevoked {
            target_option_id: _,
        } => "Отображаются только действия после изменения голоса с целевой опции".to_string(),
    };

    let text = format!(
        "<b>Лог событий голосования</b>\n{}\n\n{}\n\n{}\n\n{}",
        poll_post.posting_rule_name(),
        output_description,
        &poll_post.content.question,
        records_text
    );

    let chat_id: Recipient = action_log.action_log_chat_id().into();
    let message_id = action_log.action_log_message_id();

    match bot.edit_message_text(chat_id, message_id, &text).await {
        Ok(_) => Ok(()),
        Err(err) => {
            if err.to_string().contains("message is not modified") {
                Ok(())
            } else {
                Err(err)
            }
        }
    }
}
