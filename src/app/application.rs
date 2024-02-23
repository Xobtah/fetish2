use log::{debug, info};
use tokio::{
    signal::unix,
    sync::{broadcast, mpsc},
};

use crate::{
    app::{login_state::LoginState, ApplicationData},
    error::FetishResult,
    update_dispatcher::UpdateDispatcher,
};

use super::ApplicationState;

async fn run_application_state(starting_state: Box<dyn ApplicationState>) -> FetishResult<()> {
    let mut inner_state = starting_state;
    while let Some(next_state) = inner_state.run().await? {
        inner_state = next_state;
    }
    Ok(())
}

pub async fn run() -> FetishResult<()> {
    info!("Fetish started");

    let client_id = tdlib::create_client();
    debug!("Client ID '{client_id}' created");

    let (shutdown_tx, shutdown_rx) = broadcast::channel(10);
    let (auth_tx, auth_rx) = mpsc::unbounded_channel();
    let (message_tx, message_rx) = mpsc::unbounded_channel();

    // The update receiver is a separate task that listens for updates from the TDLib client
    // It must be spawned before any other task that sends updates to the client
    let update_receiver_handle =
        tokio::spawn(UpdateDispatcher::new(shutdown_rx.resubscribe(), auth_tx, message_tx).run());

    let client_data = ApplicationData {
        client_id,
        auth_rx,
        message_rx,
        shutdown_rx,
    };

    let mut sigint = unix::signal(unix::SignalKind::interrupt())?;
    let mut sigterm = unix::signal(unix::SignalKind::terminate())?;
    let mut sighup = unix::signal(unix::SignalKind::hangup())?;
    let mut sigquit = unix::signal(unix::SignalKind::quit())?;

    let mut state_machine_handle = tokio::spawn(run_application_state(Box::new(LoginState::new(
        client_data,
    ))));

    loop {
        tokio::select! {
            _ = &mut state_machine_handle => break,
            _ = sigint.recv() => shutdown(&shutdown_tx)?,
            _ = sigterm.recv() => shutdown(&shutdown_tx)?,
            _ = sighup.recv() => shutdown(&shutdown_tx)?,
            _ = sigquit.recv() => shutdown(&shutdown_tx)?,
        }
    }

    debug!("Waiting for update receiver to finish");
    update_receiver_handle.await?;
    info!("Fetish stopped");
    Ok(())
}

fn shutdown(shutdown_tx: &broadcast::Sender<()>) -> FetishResult<()> {
    debug!("Shutdown signal received");
    shutdown_tx.send(())?;
    Ok(())
}
