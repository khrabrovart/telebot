use std::collections::HashMap;

use anyhow::Error;
use telebot_shared::{
    aws::DynamoDbClient,
    data::{poll_action_log::PollActionLogRecord, PollActionLog, PostingRule, PostingRuleContent},
};
use teloxide::types::{MessageId, PollAnswer, Recipient};

use crate::TelegramBotClient;

pub async fn process_poll_answer(
    poll_answer: &PollAnswer,
    bot: &TelegramBotClient,
    db: &DynamoDbClient,
) -> Result<(), Error> {
    let polls_action_log_table_name = match std::env::var("POLLS_ACTION_LOG_TABLE") {
        Ok(val) => val,
        Err(_) => {
            return Err(anyhow::anyhow!(
                "POLLS_ACTION_LOG_TABLE environment variable is not set"
            ));
        }
    };

    let poll_id: String = poll_answer.poll_id.to_string();

    let poll_action_log = db
        .get_item::<PollActionLog>(&polls_action_log_table_name, &poll_id)
        .await?;

    let poll_action_log = match poll_action_log {
        Some(log) => log,
        None => {
            return Err(anyhow::anyhow!(
                "Poll action log not found for poll_id: {}",
                poll_id
            ));
        }
    };

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
    let action = poll_options[poll_answer.option_ids[0] as usize].clone();
    let timestamp = chrono::Utc::now().to_rfc3339();

    let action_record = PollActionLogRecord {
        actor_id,
        actor_first_name,
        actor_last_name,
        actor_username,
        action,
        timestamp,
    };

    let mut updated_poll_action_log = poll_action_log.clone();
    updated_poll_action_log.actions.push(action_record);
    updated_poll_action_log.version += 1;

    db.put_item(&polls_action_log_table_name, &updated_poll_action_log)
        .await?;

    update_poll_action_log_message(&updated_poll_action_log, &posting_rule, bot).await?;

    Ok(())
}

async fn update_poll_action_log_message(
    poll_action_log: &PollActionLog,
    posting_rule: &PostingRule,
    bot: &TelegramBotClient,
) -> Result<(), Error> {
    let chat_id: Recipient = poll_action_log.posting_rule_id.clone().into();
    let message_id: MessageId = MessageId(poll_action_log.action_log_message_id);

    let mut grouped_actions: HashMap<u64, Vec<PollActionLogRecord>> = HashMap::new();

    for action in poll_action_log.actions.iter() {
        grouped_actions
            .entry(action.actor_id)
            .or_default()
            .push(action.clone());
    }

    let actions_text = grouped_actions
        .values()
        .map(|actions| {
            let actor_name = format!(
                "{} {} (@{})",
                actions[0].actor_first_name,
                actions[0].actor_last_name.clone().unwrap_or_default(),
                actions[0].actor_username.clone().unwrap_or_default()
            );

            let actions_list = actions
                .iter()
                .map(|action| format!("{} at {}", action.action, action.timestamp))
                .collect::<Vec<String>>()
                .join("\n");

            format!("{}:\n{}", actor_name, actions_list)
        })
        .collect::<Vec<String>>()
        .join("\n\n");

    let text = format!(
        "<b>Лог событий для опроса по правилу</b>\n{}\n\n{}\n\n{}",
        posting_rule.name, poll_action_log.text, actions_text
    );

    bot.edit_message_text(chat_id, message_id, &text).await?;

    Ok(())
}
