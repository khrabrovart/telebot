use crate::{Post, SsmClient, TelegramBotClient};
use lambda_runtime::{Error, LambdaEvent};
use serde::Deserialize;
use telebot_shared::DynamoDbClient;
use tracing::{info, warn};

pub async fn handler(req: Request) -> Result<Response<Body>, Error> {
    let update: Update = serde_json::from_slice(req.body())?;

    match update.kind {
        UpdateKind::Message(msg) if msg.text() == Some("/start") => {
            bot.send_message(msg.chat.id, "🏠 Главное меню")
                .reply_markup(main_menu()).await?;
        }
        UpdateKind::CallbackQuery(q) => {
            let chat_id = q.message.map(|m| m.chat.id).unwrap();
            let msg_id = q.message.map(|m| m.id).unwrap();

            match q.data.as_deref() {
                Some("settings") => {
                    bot.edit_message_text(chat_id, msg_id, "⚙️ Настройки")
                        .reply_markup(settings_menu()).await?;
                }
                Some("back") => {
                    bot.edit_message_text(chat_id, msg_id, "🏠 Главное меню")
                        .reply_markup(main_menu()).await?;
                }
                _ => { bot.answer_callback_query(q.id).await?; }
            }
        }
        _ => {}
    }

    Ok(Response::builder().status(200).body(Body::Empty)?)
}

fn main_menu() -> InlineKeyboardMarkup {
    InlineKeyboardMarkup::new(vec![vec![
        InlineKeyboardButton::callback("Настройки", "settings")
    ]])
}

fn settings_menu() -> InlineKeyboardMarkup {
    InlineKeyboardMarkup::new(vec![
        vec![InlineKeyboardButton::callback("🔔 Звук", "sound"), InlineKeyboardButton::callback("🌐 Язык", "lang")],
        vec![InlineKeyboardButton::callback("« Назад", "back")]
    ])
}