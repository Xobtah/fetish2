use tdlib::{
    enums::{Chat, User},
    functions,
};

use crate::{
    database::Database,
    error::FetishResult,
    models::{chat_wrapper::ChatWrapper, user_wrapper::UserWrapper},
};

impl Database {
    pub async fn resolve_chat(&self, id: i64, client_id: i32) -> FetishResult<Option<ChatWrapper>> {
        match self.load(id) {
            Ok(Some(chat)) => Ok(Some(chat)),
            Ok(None) => {
                let Chat::Chat(chat) = functions::get_chat(id, client_id).await?;
                let chat = ChatWrapper::from(chat);
                self.save(&chat)?;
                Ok(Some(chat))
            }
            Err(e) => Err(e),
        }
    }

    pub async fn resolve_user(&self, id: i64, client_id: i32) -> FetishResult<Option<UserWrapper>> {
        match self.load(id) {
            Ok(Some(user)) => Ok(Some(user)),
            Ok(None) => {
                let User::User(user) = functions::get_user(id, client_id).await?;
                let user = UserWrapper::from(user);
                self.save(&user)?;
                Ok(Some(user))
            }
            Err(e) => Err(e),
        }
    }
}
