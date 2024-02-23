use async_trait::async_trait;
use log::{debug, error};
use tdlib::{enums::AuthorizationState, functions};

use crate::{app::message_state::MessageState, error::FetishResult, utils::ask_user};

use super::{ApplicationData, ApplicationState};

pub struct LoginState {
    client_data: ApplicationData,
}

#[async_trait]
impl ApplicationState for LoginState {
    async fn run(mut self: Box<Self>) -> FetishResult<Option<Box<dyn ApplicationState>>> {
        debug!("Logging in");

        functions::set_log_verbosity_level(1, self.client_data.client_id).await?;

        loop {
            tokio::select! {
                Some(state) = self.client_data.auth_rx.recv() => match state {
                    AuthorizationState::WaitTdlibParameters => {
                        debug!("Setting TDLib parameters");
                        let response = functions::set_tdlib_parameters(
                            false,
                            "me_db".into(),
                            String::new(),
                            String::new(),
                            true,
                            true,
                            true,
                            true,
                            include!("../../app.id"),
                            include_str!("../../app.hash").into(),
                            "en".into(),
                            "Desktop".into(),
                            String::new(),
                            env!("CARGO_PKG_VERSION").into(),
                            false,
                            true,
                            self.client_data.client_id,
                        )
                        .await;
    
                        if let Err(error) = response {
                            error!("{}", error.message);
                        }
                    }
                    AuthorizationState::WaitPhoneNumber => loop {
                        debug!("Waiting for phone number");
                        let input =
                            ask_user("Enter your phone number (include the country calling code):");
                        let response = functions::set_authentication_phone_number(
                            input,
                            None,
                            self.client_data.client_id,
                        )
                        .await;
                        match response {
                            Ok(_) => break,
                            Err(e) => error!("{}", e.message),
                        }
                    },
                    AuthorizationState::WaitCode(_) => loop {
                        debug!("Waiting for code");
                        let input = ask_user("Enter the verification code:");
                        let response =
                            functions::check_authentication_code(input, self.client_data.client_id)
                                .await;
                        match response {
                            Ok(_) => break,
                            Err(e) => error!("{}", e.message),
                        }
                    },
                    AuthorizationState::WaitPassword(_) => loop {
                        debug!("Waiting for password");
                        let input = ask_user("Enter the password:");
                        let response =
                            functions::check_authentication_password(input, self.client_data.client_id)
                                .await;
                        match response {
                            Ok(_) => break,
                            Err(e) => error!("{}", e.message),
                        }
                    },
                    AuthorizationState::Ready => {
                        debug!("Authorization complete");
                        break;
                    }
                    _ => (),
                },
                _ = self.client_data.shutdown_rx.recv() => {
                    debug!("Received shutdown signal");
                    break;
                },
            }
        }

        debug!("Logged in");
        Ok(Some(Box::new(MessageState::new(self.client_data))))
    }
}

impl LoginState {
    pub fn new(client_data: ApplicationData) -> Self {
        Self { client_data }
    }
}
