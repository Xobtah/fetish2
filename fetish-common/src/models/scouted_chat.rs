use rusqlite::OptionalExtension;
use serde::{Deserialize, Serialize};

use crate::error::FetishResult;

use super::AutoRequestable;

#[derive(Debug, Serialize, Deserialize)]
pub struct ScoutedChat {
    pub chat_id: i64,
    pub location: String,
    pub scouted_at: i64,
}

impl AutoRequestable for ScoutedChat {
    fn create_table_request() -> String {
        "CREATE TABLE IF NOT EXISTS SCOUTED_CHATS (
            chat_id INTEGER PRIMARY KEY,
            location TEXT NOT NULL,
            scouted_at INTEGER NOT NULL
        )"
        .into()
    }

    fn select_by_id(
        id: i64,
        conn: &rusqlite::Connection,
    ) -> FetishResult<Option<Self>> {
        Ok(conn
            .prepare(r#"SELECT * FROM SCOUTED_CHATS WHERE chat_id = :chat_id"#)?
            .query_row(
                rusqlite::named_params! {
                    r#":chat_id"#: id,
                },
                |row| {
                    Ok(ScoutedChat {
                        chat_id: id,
                        location: row.get("location")?,
                        scouted_at: row.get("scouted_at")?,
                    })
                },
            )
            .optional()?)
    }

    fn insert(&self, conn: &rusqlite::Connection) -> FetishResult<()> {
        conn.execute(
            "INSERT INTO SCOUTED_CHATS (chat_id, location, scouted_at) VALUES (?1, ?2, ?3)",
            rusqlite::params![self.chat_id, self.location, self.scouted_at],
        )?;
        Ok(())
    }
}
