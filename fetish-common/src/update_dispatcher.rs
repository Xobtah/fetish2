use std::{
    pin::Pin,
    sync::{Arc, Mutex},
    task::{Context, Poll},
};

use futures::{Stream, StreamExt};
use log::{error, info, trace};
use tdlib::{
    enums::{AuthorizationState, MessageContent, Update},
    functions,
    types::{Message, PhotoSize},
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
        Ok(Self {
            shutdown_rx,
            auth_tx,
            message_tx,
            db,
        })
    }

    pub async fn run(mut self) {
        info!("Starting update dispatcher");
        let mut tdlib_update_stream = UpdateStream;
        loop {
            tokio::select! {
                Some((update, client_id)) = tdlib_update_stream.next() => {
                    trace!("{update:?}");
                    if let Err(_) = self.handle_update(update, client_id) {
                        error!("Update dispatch error");
                    }
                }

                _ = self.shutdown_rx.recv() => {
                    info!("Shutting down update dispatcher");
                    break;
                }
            }
        }
    }

    fn handle_update(&self, update: Update, client_id: i32) -> Result<(), UpdateDispatchError> {
        match update {
            Update::AuthorizationState(update) => {
                Ok(self.auth_tx.send(update.authorization_state)?)
            }
            Update::NewMessage(message) => {
                if let MessageContent::MessagePhoto(message_photo) = &message.message.content {
                    if let Some(photo) = message_photo.photo.sizes.iter().fold(
                        None,
                        |acc: Option<&PhotoSize>, photo| {
                            if let Some(acc) = acc {
                                if photo.width * photo.height > acc.width * acc.height {
                                    Some(photo)
                                } else {
                                    Some(acc)
                                }
                            } else {
                                Some(photo)
                            }
                        },
                    ) {
                        download_file(photo.photo.id, client_id);
                    }
                } else if let MessageContent::MessageVideo(message_video) = &message.message.content
                {
                    download_file(message_video.video.video.id, client_id);
                } else if let MessageContent::MessageAnimation(message_animation) =
                    &message.message.content
                {
                    download_file(message_animation.animation.animation.id, client_id);
                }
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
                if let Some(photo) = &chat.photo {
                    download_file(photo.big.id, client_id);
                }
                if let Err(e) = self.db.lock().unwrap().save(&ChatWrapper::from(chat)) {
                    error!("{e:#?}");
                }
                Ok(())
            }
            Update::Supergroup(tdlib::types::UpdateSupergroup { supergroup }) => {
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
                if let Some(photo) = &user.profile_photo {
                    download_file(photo.big.id, client_id);
                }
                if let Err(e) = self.db.lock().unwrap().save(&UserWrapper::from(user)) {
                    error!("{e:#?}");
                }
                Ok(())
            }
            _ => Ok(()),
        }
    }
}

fn download_file(file_id: i32, client_id: i32) {
    tokio::spawn(async move {
        if let Err(e) = functions::download_file(file_id, 1, 0, 0, true, client_id).await {
            error!("{e:#?}");
        } else {
            trace!("Downloaded file: {file_id}");
        }
    });
}
