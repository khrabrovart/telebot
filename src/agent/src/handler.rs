use crate::{menu, TelegramBotClient};
use lambda_http::{Body, Error, Request, Response};
use telebot_shared::{aws::DynamoDbClient, data::BotData};
use teloxide::{
    dispatching::dialogue::GetChatId,
    types::{Recipient, Update},
};
use tracing::{error, info};

pub async fn handle(req: Request) -> Result<Response<Body>, Error> {
    if let Err(e) = handle_internal(req).await {
        error!(error = %e, "Failed to handle request");
    }

    Ok(Response::builder().status(200).body(Body::Empty)?)
}

async fn handle_internal(request: Request) -> Result<(), Error> {
    info!(request = ?request, "Received request");

    let path = request.uri().path();
    let bot_id = path.rsplit('/').next().unwrap();

    let update = serde_json::from_slice::<Update>(request.body())?;

    info!(update = ?update, "Parsed update");

    let db = DynamoDbClient::new().await;

    let bots_table_name = match std::env::var("BOTS_TABLE") {
        Ok(val) => val,
        Err(_) => {
            return Err("BOTS_TABLE environment variable not set".into());
        }
    };

    let bot_data = db.get_item::<BotData>(&bots_table_name, bot_id).await?;

    let bot_data = match bot_data {
        Some(data) => data,
        None => {
            return Err(format!("Bot data not found: {}", bot_id).into());
        }
    };

    info!(bot_id = %bot_data.id, "Bot data found");

    let bot = TelegramBotClient::new(&bot_data).await?;
    let chat_id: Recipient = update.chat_id().unwrap().as_user().unwrap().into();

    let sender_id = update.from().map(|u| u.id.to_string()).unwrap();

    info!(sender_id = ?sender_id, "Sender ID extracted");

    let admins = bot_data.admins;

    if !admins.contains(&sender_id) {
        bot.send_text(
            chat_id,
            "У вас недостаточно прав для выполнения этого действия",
        )
        .await?;
        return Ok(());
    }

    menu::process_update(&update, &bot, &db).await?;

    Ok(())
}
