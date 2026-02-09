use crate::TelegramBotClient;
use lambda_http::{Body, Error, Request, Response};
use telebot_shared::SsmClient;
use teloxide::{
    payloads::{EditMessageTextSetters, SendMessageSetters},
    prelude::Requester,
    types::{InlineKeyboardButton, InlineKeyboardMarkup, Update, UpdateKind},
};

pub async fn handle(req: Request) -> Result<Response<Body>, Error> {
    let ssm = SsmClient::from_env().await?;
    let bot_client = TelegramBotClient::from_ssm(&ssm).await?;

    let update: Update = serde_json::from_slice(req.body())?;

    match update.kind {
        UpdateKind::Message(msg) if msg.text() == Some("/start") => {
            bot_client
                .bot
                .send_message(msg.chat.id, "🏠 Главное меню")
                .reply_markup(main_menu())
                .await?;
        }
        UpdateKind::CallbackQuery(q) => {
            let chat_id = q.message.as_ref().map(|m| m.chat().id).unwrap();
            let msg_id = q.message.as_ref().map(|m| m.id()).unwrap();

            match q.data.as_deref() {
                Some("settings") => {
                    bot_client
                        .bot
                        .edit_message_text(chat_id, msg_id, "⚙️ Настройки")
                        .reply_markup(settings_menu())
                        .await?;
                }
                Some("back") => {
                    bot_client
                        .bot
                        .edit_message_text(chat_id, msg_id, "🏠 Главное меню")
                        .reply_markup(main_menu())
                        .await?;
                }
                _ => {
                    bot_client
                        .bot
                        .send_message(chat_id, "❓ Неизвестная команда")
                        .await?;
                }
            }
        }
        _ => {}
    }

    Ok(Response::builder().status(200).body(Body::Empty)?)
}

fn main_menu() -> InlineKeyboardMarkup {
    InlineKeyboardMarkup::new(vec![vec![InlineKeyboardButton::callback(
        "Настройки",
        "settings",
    )]])
}

fn settings_menu() -> InlineKeyboardMarkup {
    InlineKeyboardMarkup::new(vec![
        vec![
            InlineKeyboardButton::callback("🔔 Звук", "sound"),
            InlineKeyboardButton::callback("🌐 Язык", "lang"),
        ],
        vec![InlineKeyboardButton::callback("« Назад", "back")],
    ])
}
