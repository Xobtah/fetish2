pub mod chat_wrapper;
pub mod message_wrapper;
pub mod scouted_chat;
pub mod user_wrapper;

pub trait AutoRequestable {
    fn create_table_request() -> String;
    fn insert(&self, conn: &rusqlite::Connection) -> crate::error::FetishResult<()>;
}
