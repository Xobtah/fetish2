use std::{
    path::Path,
    sync::{Arc, Mutex},
};

use log::{debug, info};
use tdlib::{enums::AuthorizationState, types::Message};
use tokio::{
    signal::unix,
    sync::{broadcast, mpsc},
};

use crate::{
    database::Database, error::FetishResult, states::ApplicationState,
    update_dispatcher::UpdateDispatcher,
};

pub struct ApplicationData {
    pub client_id: i32,
    pub auth_rx: mpsc::UnboundedReceiver<AuthorizationState>,
    pub message_rx: mpsc::UnboundedReceiver<Message>,
    pub shutdown_rx: tokio::sync::broadcast::Receiver<()>,
    pub conn: Arc<Mutex<Database>>,
}

unsafe impl Send for ApplicationData {}
unsafe impl Sync for ApplicationData {}

pub struct Application {
    states: Vec<Box<dyn ApplicationState>>,
}

impl Application {
    pub fn new() -> Self {
        Self { states: Vec::new() }
    }

    pub fn add_state<AppState: ApplicationState + 'static>(mut self, state: AppState) -> Self {
        self.states.push(Box::new(state));
        self
    }

    pub async fn run(self, db_path: &Path) -> FetishResult<()> {
        info!("Fetish started");

        let client_id = tdlib::create_client();
        debug!("Client ID '{client_id}' created");

        let db = Arc::new(Mutex::new(Database::new(db_path)?));

        let (shutdown_tx, shutdown_rx) = broadcast::channel(1);
        let (shutdown_update_dispatcher_tx, shutdown_update_dispatcher_rx) = broadcast::channel(1);
        let (auth_tx, auth_rx) = mpsc::unbounded_channel();
        let (message_tx, message_rx) = mpsc::unbounded_channel();

        // The update receiver is a separate task that listens for updates from the TDLib client
        // It must be spawned before any other task that sends updates to the client
        let update_dispatcher_handle = tokio::spawn(
            UpdateDispatcher::new(
                shutdown_update_dispatcher_rx,
                auth_tx,
                message_tx,
                db.clone(),
            )?
            .run(),
        );

        let mut state_machine_handle = tokio::spawn(async move {
            let mut app_data = ApplicationData {
                client_id,
                auth_rx,
                message_rx,
                shutdown_rx,
                conn: db,
            };

            debug!("Running state machine");
            for state in self.states {
                app_data = state.run(app_data).await.unwrap();
            }
            debug!("State machine finished");
        });

        let mut sigint = unix::signal(unix::SignalKind::interrupt())?;
        let mut sigterm = unix::signal(unix::SignalKind::terminate())?;
        let mut sighup = unix::signal(unix::SignalKind::hangup())?;
        let mut sigquit = unix::signal(unix::SignalKind::quit())?;

        loop {
            tokio::select! {
                _ = &mut state_machine_handle => break,
                _ = sigint.recv() => shutdown(&shutdown_tx)?,
                _ = sigterm.recv() => shutdown(&shutdown_tx)?,
                _ = sighup.recv() => shutdown(&shutdown_tx)?,
                _ = sigquit.recv() => shutdown(&shutdown_tx)?,
            }
        }

        shutdown_update_dispatcher_tx.send(())?;
        debug!("Waiting for update receiver to finish");
        update_dispatcher_handle.await?;
        info!("Fetish stopped");
        Ok(())
    }
}

fn shutdown(shutdown_tx: &broadcast::Sender<()>) -> FetishResult<()> {
    debug!("Shutdown signal received");
    shutdown_tx.send(())?;
    Ok(())
}
