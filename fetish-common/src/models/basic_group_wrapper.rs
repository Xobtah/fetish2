use std::ops::Deref;

use rusqlite::OptionalExtension;
use serde::{Serialize, Serializer};
use tdlib::types::BasicGroup;

use crate::error::FetishResult;

use super::AutoRequestable;

#[derive(Debug)]
pub struct BasicGroupWrapper(BasicGroup);

impl Deref for BasicGroupWrapper {
    type Target = BasicGroup;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl From<BasicGroup> for BasicGroupWrapper {
    fn from(basic_group: BasicGroup) -> Self {
        Self(basic_group)
    }
}

impl Serialize for BasicGroupWrapper {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        self.0.serialize(serializer)
    }
}

impl AutoRequestable for BasicGroupWrapper {
    fn create_table_request() -> String {
        r#"CREATE TABLE IF NOT EXISTS BASIC_GROUPS (
            id INTEGER PRIMARY KEY,
            member_count INTEGER NOT NULL,
            status TEXT NOT NULL,
            is_active BOOLEAN NOT NULL,
            upgraded_to_supergroup_id INTEGER NOT NULL
        )"#
        .into()
    }

    fn select_by_id(id: i64, conn: &rusqlite::Connection) -> FetishResult<Option<Self>> {
        Ok(conn
            .prepare(r#"SELECT * FROM BASIC_GROUPS WHERE id = :id"#)?
            .query_row(
                rusqlite::named_params! {
                    r#":id"#: id,
                },
                |row| {
                    Ok(BasicGroupWrapper(BasicGroup {
                        id,
                        member_count: row.get("member_count")?,
                        status: serde_json::from_str(&row.get::<_, String>("status")?).unwrap(),
                        is_active: row.get("is_active")?,
                        upgraded_to_supergroup_id: row.get("upgraded_to_supergroup_id")?,
                    }))
                },
            )
            .optional()?)
    }

    fn insert(&self, conn: &rusqlite::Connection) -> FetishResult<()> {
        if let Some(_) = Self::select_by_id(self.0.id, conn)? {
            return Ok(());
        }
        conn.execute(
            r#"INSERT INTO BASIC_GROUPS (
            id,
            member_count,
            status,
            is_active,
            upgraded_to_supergroup_id
        ) VALUES (
            :id,
            :member_count,
            :status,
            :is_active,
            :upgraded_to_supergroup_id
        )"#
            .into(),
            rusqlite::named_params! {
                r#":id"#: &self.0.id,
                r#":member_count"#: &self.0.member_count,
                r#":status"#: &serde_json::to_string(&self.0.status).unwrap(),
                r#":is_active"#: &self.0.is_active,
                r#":upgraded_to_supergroup_id"#: &self.0.upgraded_to_supergroup_id,
            },
        )?;
        Ok(())
    }
}
