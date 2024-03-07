use clap::Parser;
use fetish_common::{
    application::Application,
    error::FetishResult,
    location::Location,
    states::{
        closing_state::ClosingState, exploitation_state::ExploitationState, login_state::LoginState,
    },
};

mod args;

#[tokio::main]
async fn main() -> FetishResult<()> {
    env_logger::init();
    let args = args::Args::parse();
    Application::new()
        .add_state(LoginState::new(&args.tg_database_directory))
        .add_state(ExploitationState::new(
            Location::new(48.864716, 2.349014).compute_locations(860., 5),
        ))
        .add_state(ClosingState)
        .run(&args.database_path)
        .await
}
