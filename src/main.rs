mod app;
mod error;
mod update_dispatcher;
mod utils;

#[tokio::main]
async fn main() -> Result<(), error::FetishError> {
    env_logger::init();
    app::run().await
}
