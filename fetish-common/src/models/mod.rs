use crate::error::FetishResult;

pub mod basic_group_wrapper;
pub mod chat_wrapper;
pub mod message_wrapper;
pub mod scammer;
pub mod scouted_chat;
pub mod supergroup_wrapper;
pub mod user_wrapper;

pub trait AutoRequestable {
    fn create_table_request() -> String;
    fn select_by_id(id: i64, conn: &rusqlite::Connection) -> FetishResult<Option<Self>>
    where
        Self: std::marker::Sized;
    fn insert(&self, conn: &rusqlite::Connection) -> FetishResult<()>;
}
