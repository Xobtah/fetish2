use clap::Parser;
use fetish_common::{
    application::Application,
    error::FetishResult,
    states::{closing_state::ClosingState, login_state::LoginState, message_state::MessageState},
};

mod args;

#[tokio::main]
async fn main() -> FetishResult<()> {
    env_logger::init();
    let args = args::Args::parse();
    Application::new()
        .add_state(LoginState::new(&args.tg_database_directory))
        .add_state(MessageState)
        .add_state(ClosingState)
        .run(&args.database_path)
        .await
}
