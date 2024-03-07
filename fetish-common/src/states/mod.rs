use crate::{application::ApplicationData, error::FetishResult};

use async_trait::async_trait;

pub mod closing_state;
pub mod login_state;
pub mod exploitation_state;

#[async_trait]
pub trait ApplicationState: Sync + Send {
    async fn run(&self, app_data: ApplicationData) -> FetishResult<ApplicationData>;
}
