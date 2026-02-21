use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct BotData {
    pub id: String,
    pub token: String,
    pub admins: Vec<String>,
}
