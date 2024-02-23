use async_trait::async_trait;
use log::info;
use tdlib::{enums::AuthorizationState, functions};

use crate::error::FetishResult;

use super::{ApplicationData, ApplicationState};

pub struct ClosingState {
    client_data: ApplicationData,
}

#[async_trait]
impl ApplicationState for ClosingState {
    async fn run(mut self: Box<Self>) -> FetishResult<Option<Box<dyn ApplicationState>>> {
        info!("Closing connection");
        functions::close(self.client_data.client_id).await?;
        self.handle_close().await;
        info!("Connection closed");
        Ok(None)
    }
}

impl ClosingState {
    pub fn new(client_data: ApplicationData) -> Self {
        Self { client_data }
    }

    async fn handle_close(&mut self) {
        while let Some(state) = self.client_data.auth_rx.recv().await {
            match state {
                AuthorizationState::Closed => {
                    break;
                }
                _ => (),
            }
        }
    }
}
