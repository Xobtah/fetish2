use std::path::Path;

use log::debug;
use rusqlite::Connection;
// use serde_rusqlite::to_params_named;

use crate::{
    error::FetishResult,
    models::{
        chat_wrapper::ChatWrapper, message_wrapper::MessageWrapper, user_wrapper::UserWrapper,
        AutoRequestable,
    },
};

pub struct Database {
    conn: Connection,
}

unsafe impl Send for Database {}
unsafe impl Sync for Database {}

impl Database {
    pub fn new(db_path: &Path) -> Result<Self, rusqlite::Error> {
        debug!("Creating database '{}'", db_path.display());
        let conn = Connection::open(db_path)?;

        conn.execute(&ChatWrapper::create_table_request(), rusqlite::params![])?;
        conn.execute(&MessageWrapper::create_table_request(), rusqlite::params![])?;
        conn.execute(&UserWrapper::create_table_request(), rusqlite::params![])?;

        Ok(Self { conn })
    }

    pub fn save<DatabaseEntity: AutoRequestable + serde::Serialize>(
        &self,
        entity: &DatabaseEntity,
    ) -> FetishResult<()> {
        entity.insert(&self.conn)?;
        Ok(())
    }
}
