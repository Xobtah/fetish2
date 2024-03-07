use rusqlite::Connection;

use crate::error::FetishResult;

use self::{
    basic_group_wrapper::BasicGroupWrapper, chat_wrapper::ChatWrapper,
    message_wrapper::MessageWrapper, scammer::Scammer, scouted_chat::ScoutedChat,
    supergroup_wrapper::SupergroupWrapper, user_wrapper::UserWrapper,
};

pub mod basic_group_wrapper;
pub mod chat_wrapper;
pub mod message_wrapper;
pub mod scammer;
pub mod scouted_chat;
pub mod supergroup_wrapper;
pub mod user_wrapper;

pub trait AutoRequestable {
    type UniqueIdentifier;

    fn create_table_request() -> String;
    fn get_id(&self) -> Self::UniqueIdentifier;
    fn select_by_id(id: Self::UniqueIdentifier, conn: &rusqlite::Connection) -> FetishResult<Option<Self>>
    where
        Self: std::marker::Sized;
    fn select_all(conn: &rusqlite::Connection) -> FetishResult<Vec<Self>>
    where
        Self: std::marker::Sized;
    fn insert(&self, conn: &rusqlite::Connection) -> FetishResult<()>;
    fn update(&self, conn: &rusqlite::Connection) -> FetishResult<()>;
}

pub fn init_db(conn: &Connection) -> Result<(), rusqlite::Error> {
    conn.execute(
        &BasicGroupWrapper::create_table_request(),
        rusqlite::params![],
    )?;
    conn.execute(&ChatWrapper::create_table_request(), rusqlite::params![])?;
    conn.execute(&MessageWrapper::create_table_request(), rusqlite::params![])?;
    conn.execute(&Scammer::create_table_request(), rusqlite::params![])?;
    conn.execute(&ScoutedChat::create_table_request(), rusqlite::params![])?;
    conn.execute(
        &SupergroupWrapper::create_table_request(),
        rusqlite::params![],
    )?;
    conn.execute(&UserWrapper::create_table_request(), rusqlite::params![])?;
    Ok(())
}
