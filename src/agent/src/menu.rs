use anyhow::Error;
use teloxide::{
    payloads::{EditMessageTextSetters, SendMessageSetters},
    prelude::Requester,
    types::{InlineKeyboardButton, InlineKeyboardMarkup, Update, UpdateKind},
};

use crate::TelegramBotClient;

pub async fn process_update(update: &Update, bot: &TelegramBotClient) -> Result<(), Error> {
    // match update.kind {
    //     UpdateKind::Message(msg) if msg.text() == Some("/start") => {
    //         bot_client
    //             .bot
    //             .send_message(msg.chat.id, "Главное меню")
    //             .reply_markup(main_menu())
    //             .await?;
    //     }
    //     UpdateKind::CallbackQuery(q) => {
    //         let chat_id = q.message.as_ref().map(|m| m.chat().id).unwrap();
    //         let msg_id = q.message.as_ref().map(|m| m.id()).unwrap();

    //         match q.data.as_deref() {
    //             Some("posting_settings") => {
    //                 bot_client
    //                     .bot
    //                     .edit_message_text(chat_id, msg_id, "⚙️ Настройки")
    //                     .reply_markup(settings_menu())
    //                     .await?;
    //             }
    //             Some("back") => {
    //                 bot_client
    //                     .bot
    //                     .edit_message_text(chat_id, msg_id, "Главное меню")
    //                     .reply_markup(main_menu())
    //                     .await?;
    //             }
    //             _ => {}
    //         }
    //     }
    //     _ => {}
    // }

    Ok(())
}

fn main_menu() -> InlineKeyboardMarkup {
    InlineKeyboardMarkup::new(vec![vec![InlineKeyboardButton::callback(
        "Автопостинг",
        "posting_settings",
    )]])
}

fn list_rules_menu() -> InlineKeyboardMarkup {
    InlineKeyboardMarkup::new(vec![
        vec![
            InlineKeyboardButton::callback("Правило 1", "rule_1"),
            InlineKeyboardButton::callback("Правило 2", "rule_2"),
        ],
        vec![InlineKeyboardButton::callback(
            "< Назад",
            "posting_settings",
        )],
    ])
}
