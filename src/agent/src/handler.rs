use crate::{processor, AppContext};
use lambda_http::{Body, Error, Request, Response};
use telebot_shared::data::BotDataRepository;
use teloxide::types::Update;
use tracing::{error, info};

pub async fn handle(req: Request, app: &AppContext) -> Result<Response<Body>, Error> {
    if let Err(e) = handle_internal(req, app).await {
        error!(error = %e, "Failed to handle request");
    }

    Ok(Response::builder().status(200).body(Body::Empty)?)
}

async fn handle_internal(request: Request, app: &AppContext) -> Result<(), Error> {
    info!(request = ?request, "Received request");

    let path = request.uri().path();
    let bot_id = path.rsplit('/').next().unwrap();

    let update = serde_json::from_slice::<Update>(request.body())?;

    info!(update = ?update, "Parsed update");

    let bot_data_repository = BotDataRepository::new(&app.dynamodb).await?;

    let bot_data = bot_data_repository.get(bot_id).await?;

    let bot_data = match bot_data {
        Some(data) => data,
        None => {
            return Err(format!("Bot data not found: {}", bot_id).into());
        }
    };

    info!(bot_id = %bot_data.id, "Bot data found");

    processor::process(&update, &bot_data, &app.dynamodb).await?;

    Ok(())
}
