use std::{
    pin::Pin,
    sync::{Arc, Mutex},
    task::{Context, Poll},
};

use futures::{Stream, StreamExt};
use log::{debug, error, trace};
use tdlib::{
    enums::{AuthorizationState, MessageContent, Update},
    types::Message,
};
use tokio::sync::{
    broadcast,
    mpsc::{self, error::SendError},
};

use crate::{
    database::Database,
    error::FetishResult,
    models::{
        basic_group_wrapper::BasicGroupWrapper, chat_wrapper::ChatWrapper,
        message_wrapper::MessageWrapper, supergroup_wrapper::SupergroupWrapper,
        user_wrapper::UserWrapper,
    },
};

struct UpdateStream;

impl Stream for UpdateStream {
    type Item = (Update, i32);

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        trace!("Polling for updates");
        match tdlib::receive() {
            Some((update, client_id)) => Poll::Ready(Some((update, client_id))),
            None => {
                cx.waker().wake_by_ref();
                Poll::Pending
            }
        }
    }
}

#[derive(Debug)]
struct UpdateDispatchError;

impl From<SendError<AuthorizationState>> for UpdateDispatchError {
    fn from(_: SendError<AuthorizationState>) -> Self {
        Self
    }
}

impl From<SendError<Message>> for UpdateDispatchError {
    fn from(_: SendError<Message>) -> Self {
        Self
    }
}

pub struct UpdateDispatcher {
    shutdown_rx: broadcast::Receiver<()>,
    auth_tx: mpsc::UnboundedSender<AuthorizationState>,
    message_tx: mpsc::UnboundedSender<Message>,
    db: Arc<Mutex<Database>>,
}

impl UpdateDispatcher {
    pub fn new(
        shutdown_rx: broadcast::Receiver<()>,
        auth_tx: mpsc::UnboundedSender<AuthorizationState>,
        message_tx: mpsc::UnboundedSender<Message>,
        db: Arc<Mutex<Database>>,
    ) -> FetishResult<Self> {
        debug!("Creating update dispatcher");
        Ok(Self {
            shutdown_rx,
            auth_tx,
            message_tx,
            db,
        })
    }

    pub async fn run(mut self) {
        debug!("Starting update dispatcher");

        let mut tdlib_update_stream = UpdateStream;
        loop {
            tokio::select! {
                Some((update, _)) = tdlib_update_stream.next() => {
                    trace!("{update:?}");
                    if let Err(_) = self.handle_update(update) {
                        error!("Update dispatch error");
                    }
                }

                _ = self.shutdown_rx.recv() => {
                    debug!("Shutting down update dispatcher");
                    break;
                }
            }
        }
    }

    fn handle_update(&self, update: Update) -> Result<(), UpdateDispatchError> {
        match update {
            Update::AuthorizationState(update) => {
                Ok(self.auth_tx.send(update.authorization_state)?)
            }
            Update::NewMessage(message) => {
                if let MessageContent::MessageText(message_text) = &message.message.content {
                    debug!("New message: {}", message_text.text.text);
                } else {
                    debug!("New message (not text)");
                }
                trace!("{:#?}", message.message);
                if let Err(e) = self
                    .db
                    .lock()
                    .unwrap()
                    .save(&MessageWrapper::from(message.message.clone()))
                {
                    error!("{e:#?}");
                }
                Ok(self.message_tx.send(message.message)?)
            }
            Update::NewChat(tdlib::types::UpdateNewChat { chat }) => {
                debug!("New chat: {}", chat.title);
                trace!("{chat:#?}");
                if let Err(e) = self.db.lock().unwrap().save(&ChatWrapper::from(chat)) {
                    error!("{e:#?}");
                }
                Ok(())
            }
            Update::Supergroup(tdlib::types::UpdateSupergroup { supergroup }) => {
                debug!("New supergroup: {}", supergroup.id);
                trace!("{supergroup:#?}");
                if let Err(e) = self
                    .db
                    .lock()
                    .unwrap()
                    .save(&SupergroupWrapper::from(supergroup))
                {
                    error!("{e:#?}");
                }
                Ok(())
            }
            Update::BasicGroup(tdlib::types::UpdateBasicGroup { basic_group }) => {
                debug!("New basic group: {}", basic_group.id);
                trace!("{basic_group:#?}");
                if let Err(e) = self
                    .db
                    .lock()
                    .unwrap()
                    .save(&BasicGroupWrapper::from(basic_group))
                {
                    error!("{e:#?}");
                }
                Ok(())
            }
            Update::User(tdlib::types::UpdateUser { user }) => {
                debug!("New user: '{} {}'", user.first_name, user.last_name);
                trace!("{user:#?}");
                if let Err(e) = self.db.lock().unwrap().save(&UserWrapper::from(user)) {
                    error!("{e:#?}");
                }
                Ok(())
            }
            _ => Ok(()),
        }
    }
}
