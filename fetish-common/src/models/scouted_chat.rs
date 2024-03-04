use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct ScoutedChat {
    pub id: i64,
    pub chat_id: i64,
    pub location: String,
    pub scouted_at: String,
}
