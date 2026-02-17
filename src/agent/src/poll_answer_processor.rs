use std::collections::HashMap;

use anyhow::Error;
use chrono_tz::Tz;
use telebot_shared::{
    aws::DynamoDbClient,
    data::{
        PollActionLog, PollActionLogOutput, PollActionLogRecord, PostingRule, PostingRuleContent,
    },
    repositories::PollActionLogRepository,
};
use teloxide::types::{MessageId, PollAnswer, Recipient};

use crate::TelegramBotClient;

pub async fn process_poll_answer(
    poll_answer: &PollAnswer,
    bot: &TelegramBotClient,
    db: &DynamoDbClient,
) -> Result<(), Error> {
    let poll_action_log_repository = PollActionLogRepository::new().await?;

    let poll_action_log = poll_action_log_repository
        .get(&poll_answer.poll_id.to_string())
        .await?;

    let posting_rules_table_name = match std::env::var("POSTING_RULES_TABLE") {
        Ok(val) => val,
        Err(_) => {
            return Err(anyhow::anyhow!(
                "POSTING_RULES_TABLE environment variable is not set"
            ));
        }
    };

    let posting_rule = db
        .get_item::<PostingRule>(&posting_rules_table_name, &poll_action_log.posting_rule_id)
        .await?;

    let posting_rule = match posting_rule {
        Some(rule) => rule,
        None => {
            return Err(anyhow::anyhow!(
                "Posting rule not found for id: {}",
                poll_action_log.posting_rule_id
            ));
        }
    };

    let poll_options = match &posting_rule.content {
        PostingRuleContent::Poll { options, .. } => options,
        _ => {
            return Err(anyhow::anyhow!(
                "Posting rule content is not a poll for id: {}",
                &posting_rule.id
            ));
        }
    };

    let actor_id = poll_answer.voter.user().unwrap().id.0;
    let actor_first_name = poll_answer.voter.user().unwrap().first_name.clone();
    let actor_last_name = poll_answer.voter.user().unwrap().last_name.clone();
    let actor_username = poll_answer.voter.user().unwrap().username.clone();
    let (option_id, option_text) = if !poll_answer.option_ids.is_empty() {
        let option_id = poll_answer.option_ids[0];
        let option_text = poll_options[option_id as usize].clone();
        (Some(option_id as i32), Some(option_text))
    } else {
        (None, None)
    };

    let timestamp = chrono::Utc::now().timestamp();

    let action_record = PollActionLogRecord {
        actor_id,
        actor_first_name,
        actor_last_name,
        actor_username,
        option_id,
        option_text,
        timestamp,
    };

    let mut updated_poll_action_log = poll_action_log.clone();
    updated_poll_action_log.records.push(action_record);

    poll_action_log_repository
        .put(&updated_poll_action_log)
        .await?;

    update_poll_action_log_message(&updated_poll_action_log, &posting_rule, bot).await?;

    Ok(())
}

async fn update_poll_action_log_message(
    poll_action_log: &PollActionLog,
    posting_rule: &PostingRule,
    bot: &TelegramBotClient,
) -> Result<(), Error> {
    let chat_id: Recipient = posting_rule
        .poll_action_log
        .as_ref()
        .unwrap()
        .chat_id
        .into();

    let message_id: MessageId = MessageId(poll_action_log.action_log_message_id);

    let mut grouped_records: HashMap<u64, Vec<PollActionLogRecord>> = HashMap::new();

    for record in poll_action_log.records.iter() {
        grouped_records
            .entry(record.actor_id)
            .or_default()
            .push(record.clone());
    }

    let mut filtered_records: HashMap<u64, Vec<PollActionLogRecord>> = HashMap::new();

    let poll_action_log_config = posting_rule.poll_action_log.as_ref().unwrap();

    match poll_action_log_config.output {
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

    let mut records_text = filtered_records
        .values()
        .map(|records| {
            let actor_name = format!(
                "<b>{} {} (@{})</b>",
                records[0].actor_first_name,
                records[0].actor_last_name.clone().unwrap_or_default(),
                records[0].actor_username.clone().unwrap_or_default()
            );

            let actions_list = records
                .iter()
                .map(|record| {
                    let tz: Tz = poll_action_log.timezone.parse().unwrap();
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
        records_text = "<i>Здесь будут отображаться действия с данным опросом</i>".to_string();
    }

    let output_description = match poll_action_log_config.output {
        PollActionLogOutput::All => "Отображаются все действия".to_string(),
        PollActionLogOutput::OnlyWhenTargetOptionRevoked {
            target_option_id: _,
        } => "Отображаются только действия после изменения голоса с целевой опции".to_string(),
    };

    let text = format!(
        "<b>Лог событий опроса</b>\n{}\n\n{}\n\n{}",
        output_description, poll_action_log.text, records_text
    );

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
