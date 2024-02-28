use fetish_common::{
    application::Application,
    error::FetishResult,
    state::{closing_state::ClosingState, login_state::LoginState, message_state::MessageState},
};

#[tokio::main]
async fn main() -> FetishResult<()> {
    env_logger::init();
    Application::new()
        .add_state(LoginState::new("db_me"))
        .add_state(MessageState)
        .add_state(ClosingState)
        .run()
        .await
}
