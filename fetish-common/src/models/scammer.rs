use rusqlite::OptionalExtension;
use serde::{Deserialize, Serialize};

use crate::error::FetishResult;

use super::AutoRequestable;

#[derive(Debug, Serialize, Deserialize)]
pub struct Scammer {
    pub user_id: i64,
}

impl AutoRequestable for Scammer {
    type UniqueIdentifier = i64;

    fn create_table_request() -> String {
        "CREATE TABLE IF NOT EXISTS SCAMMERS (
            user_id INTEGER PRIMARY KEY
        )"
        .into()
    }

    fn get_id(&self) -> Self::UniqueIdentifier {
        self.user_id
    }

    fn select_by_id(
        id: Self::UniqueIdentifier,
        conn: &rusqlite::Connection,
    ) -> FetishResult<Option<Self>> {
        Ok(conn
            .prepare(r#"SELECT * FROM SCAMMERS WHERE user_id = :user_id"#)?
            .query_row(
                rusqlite::named_params! {
                    r#":user_id"#: id,
                },
                |_| Ok(Scammer { user_id: id }),
            )
            .optional()?)
    }

    fn select_all(conn: &rusqlite::Connection) -> FetishResult<Vec<Self>> {
        Ok(conn
            .prepare(r#"SELECT * FROM SCAMMERS"#)?
            .query_map(rusqlite::named_params! {}, |row| {
                Ok(Scammer {
                    user_id: row.get("user_id")?,
                })
            })?
            .into_iter()
            .filter_map(Result::ok)
            .collect::<Vec<Self>>())
    }

    fn insert(&self, conn: &rusqlite::Connection) -> FetishResult<()> {
        conn.execute(
            "INSERT INTO SCAMMERS (user_id) VALUES (?1)",
            rusqlite::params![self.user_id],
        )?;
        Ok(())
    }

    fn update(&self, _conn: &rusqlite::Connection) -> FetishResult<()> {
        Ok(())
    }
}
