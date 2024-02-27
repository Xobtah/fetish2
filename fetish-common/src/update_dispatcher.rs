use std::{
    pin::Pin,
    task::{Context, Poll},
};

use futures::{Stream, StreamExt};
use log::{debug, error, trace};
use tdlib::{
    enums::{AuthorizationState, Update},
    types::Message,
};
use tokio::sync::{
    broadcast,
    mpsc::{self, error::SendError},
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
}

impl UpdateDispatcher {
    pub fn new(
        shutdown_rx: broadcast::Receiver<()>,
        auth_tx: mpsc::UnboundedSender<AuthorizationState>,
        message_tx: mpsc::UnboundedSender<Message>,
    ) -> Self {
        debug!("Creating update dispatcher");
        Self {
            shutdown_rx,
            auth_tx,
            message_tx,
        }
    }

    pub async fn run(mut self) {
        debug!("Starting update dispatcher");

        let mut tdlib_update_stream = UpdateStream;
        loop {
            tokio::select! {
                Some((update, _)) = tdlib_update_stream.next() => {
                    trace!("{update:#?}");
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
                debug!("Handling authorization state update");
                Ok(self.auth_tx.send(update.authorization_state)?)
            }
            Update::NewMessage(message) => {
                debug!("Handling new message");
                Ok(self.message_tx.send(message.message)?)
            }
            _ => Ok(()),
        }
    }
}
