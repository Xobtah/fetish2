use async_trait::async_trait;
use dialoguer::{theme::ColorfulTheme, Input};
use log::{debug, error};
use tdlib::{enums::AuthorizationState, functions};

use crate::{application::ApplicationData, error::FetishResult};

use super::ApplicationState;

pub struct LoginState {
    database_name: String,
}

impl LoginState {
    pub fn new(db_name: &str) -> Self {
        Self {
            database_name: db_name.to_owned(),
        }
    }
}

#[async_trait]
impl ApplicationState for LoginState {
    async fn run(&self, mut app_data: ApplicationData) -> FetishResult<ApplicationData> {
        debug!("Logging in");

        functions::set_log_verbosity_level(1, app_data.client_id).await?;

        loop {
            tokio::select! {
                Some(state) = app_data.auth_rx.recv() => match state {
                    AuthorizationState::WaitTdlibParameters => {
                        debug!("Setting TDLib parameters");
                        let response = functions::set_tdlib_parameters(
                            false,
                            self.database_name.to_owned(),
                            Default::default(),
                            Default::default(),
                            true,
                            true,
                            true,
                            true,
                            include!("../../../app.id"),
                            include_str!("../../../app.hash").into(),
                            "en".into(),
                            "Desktop".into(),
                            Default::default(),
                            env!("CARGO_PKG_VERSION").into(),
                            false,
                            true,
                            app_data.client_id,
                        )
                        .await;

                        if let Err(error) = response {
                            error!("{}", error.message);
                        }
                    }
                    AuthorizationState::WaitPhoneNumber => loop {
                        debug!("Waiting for phone number");
                        let input =
                            ask_user("Enter your phone number (include the country calling code):")?;
                        let response = functions::set_authentication_phone_number(
                            input,
                            None,
                            app_data.client_id,
                        )
                        .await;
                        match response {
                            Ok(_) => break,
                            Err(e) => error!("{}", e.message),
                        }
                    },
                    AuthorizationState::WaitCode(_) => loop {
                        debug!("Waiting for code");
                        let input = ask_user("Enter the verification code:")?;
                        let response =
                            functions::check_authentication_code(input, app_data.client_id)
                                .await;
                        match response {
                            Ok(_) => break,
                            Err(e) => error!("{}", e.message),
                        }
                    },
                    AuthorizationState::WaitPassword(_) => loop {
                        debug!("Waiting for password");
                        let input = ask_user("Enter the password:")?;
                        let response =
                            functions::check_authentication_password(input, app_data.client_id)
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
                _ = app_data.shutdown_rx.recv() => {
                    debug!("Received shutdown signal");
                    break;
                },
            }
        }

        debug!("Logged in");
        Ok(app_data)
    }
}

fn ask_user(prompt: &str) -> FetishResult<String> {
    Ok(Input::<String>::with_theme(&ColorfulTheme::default())
        .with_prompt(prompt)
        .interact_text()?
        .trim()
        .into())
}
