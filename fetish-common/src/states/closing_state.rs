use async_trait::async_trait;
use log::{debug, info};
use tdlib::{enums::AuthorizationState, functions};

use crate::{application::ApplicationData, error::FetishResult};

use super::ApplicationState;

pub struct ClosingState;

#[async_trait]
impl ApplicationState for ClosingState {
    async fn run(&self, mut app_data: ApplicationData) -> FetishResult<ApplicationData> {
        info!("Closing connection");

        functions::close(app_data.client_id).await?;

        loop {
            tokio::select! {
                Some(state) = app_data.auth_rx.recv() => match state {
                    AuthorizationState::Closed => break,
                    _ => (),
                },
                _ = app_data.shutdown_rx.recv() => {
                    debug!("Received shutdown signal");
                    break;
                }
            }
        }

        info!("Connection closed");
        Ok(app_data)
    }
}
