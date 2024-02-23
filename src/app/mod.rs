use crate::error::FetishResult;

use async_trait::async_trait;
use tdlib::{enums::AuthorizationState, types::Message};
use tokio::sync::mpsc;

#[async_trait]
trait ApplicationState: Send {
    async fn run(mut self: Box<Self>) -> FetishResult<Option<Box<dyn ApplicationState>>>;
}

mod application;
mod closing_state;
mod login_state;
mod message_state;

pub use application::run;

pub struct ApplicationData {
    pub client_id: i32,
    pub auth_rx: mpsc::UnboundedReceiver<AuthorizationState>,
    pub message_rx: mpsc::UnboundedReceiver<Message>,
    pub shutdown_rx: tokio::sync::broadcast::Receiver<()>,
}
