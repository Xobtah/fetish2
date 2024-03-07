use rusqlite::OptionalExtension;
use serde::{Deserialize, Serialize};

use crate::{error::FetishResult, location::Location};

use super::AutoRequestable;

#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
pub struct ScoutedChat {
    pub chat_id: i64,
    pub location: Location,
    pub scouted_at: i64,
    pub joined_at: Option<i64>,
}

impl AutoRequestable for ScoutedChat {
    type UniqueIdentifier = i64;

    fn create_table_request() -> String {
        "CREATE TABLE IF NOT EXISTS SCOUTED_CHATS (
            chat_id INTEGER PRIMARY KEY,
            location TEXT NOT NULL,
            scouted_at INTEGER NOT NULL,
            joined_at INTEGER
        )"
        .into()
    }

    fn get_id(&self) -> Self::UniqueIdentifier {
        self.chat_id
    }

    fn select_by_id(
        id: Self::UniqueIdentifier,
        conn: &rusqlite::Connection,
    ) -> FetishResult<Option<Self>> {
        Ok(conn
            .prepare(r#"SELECT * FROM SCOUTED_CHATS WHERE chat_id = :chat_id"#)?
            .query_row(
                rusqlite::named_params! {
                    r#":chat_id"#: id,
                },
                from_row,
            )
            .optional()?)
    }

    fn select_all(conn: &rusqlite::Connection) -> FetishResult<Vec<Self>> {
        Ok(conn
            .prepare(r#"SELECT * FROM SCOUTED_CHATS"#)?
            .query_map(rusqlite::named_params! {}, from_row)?
            .into_iter()
            .filter_map(Result::ok)
            .collect::<Vec<Self>>())
    }

    fn insert(&self, conn: &rusqlite::Connection) -> FetishResult<()> {
        conn.execute(
            "INSERT INTO SCOUTED_CHATS (chat_id, location, scouted_at, joined_at) VALUES (?1, ?2, ?3, ?4)",
            rusqlite::params![
                self.chat_id,
                serde_json::to_string(&self.location).unwrap(),
                self.scouted_at,
                self.joined_at,
            ],
        )?;
        Ok(())
    }

    fn update(&self, conn: &rusqlite::Connection) -> FetishResult<()> {
        conn.execute(
            r#"UPDATE SCOUTED_CHATS
            SET
                location = ?2,
                scouted_at = ?3,
                joined_at = ?4
            WHERE
                chat_id = ?1"#
                .into(),
            rusqlite::params![
                self.chat_id,
                serde_json::to_string(&self.location).unwrap(),
                self.scouted_at,
                self.joined_at,
            ],
        )?;
        Ok(())
    }
}

fn from_row(row: &rusqlite::Row) -> Result<ScoutedChat, rusqlite::Error> {
    Ok(ScoutedChat {
        chat_id: row.get("chat_id")?,
        location: serde_json::from_str(&row.get::<_, String>("location")?).unwrap(),
        scouted_at: row.get("scouted_at")?,
        joined_at: row.get("joined_at")?,
    })
}
