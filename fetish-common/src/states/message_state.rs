use std::{
    fs,
    sync::{Arc, Mutex},
};

use async_trait::async_trait;
use log::{debug, error, info, trace};
use tdlib::{
    enums::{InputMessageContent, MessageContent, MessageSender, User},
    functions,
    types::{FormattedText, InputMessageText, Message, MessageSenderUser},
};
use unidecode::unidecode;

use crate::{
    application::ApplicationData,
    database::Database,
    error::{FetishError, FetishResult},
    models::scammer::Scammer,
};

use super::ApplicationState;

pub type SendMessageData = (Message, InputMessageContent, i32);

pub struct MessageState;

#[async_trait]
impl ApplicationState for MessageState {
    async fn run(&self, mut app_data: ApplicationData) -> FetishResult<ApplicationData> {
        let (message_to_send_tx, message_to_send_rx) = tokio::sync::mpsc::unbounded_channel();
        let shutdown_rx = app_data.shutdown_rx.resubscribe();
        debug!("Starting message sender");
        let message_sender_handle =
            tokio::spawn(message_sender::run(shutdown_rx, message_to_send_rx));

        info!("Start listening for messages");
        let User::User(me) = functions::get_me(app_data.client_id).await.unwrap();

        loop {
            tokio::select! {
                Some(message) = app_data.message_rx.recv() => {
                    let failsafe = {
                        // Skip messages from private chats
                        if message.chat_id >= 0 {
                            continue;
                        }

                        if let Some((user_id, is_scammer)) = is_scammer_account(app_data.db.clone(), &message)? {
                            // Skip messages from me
                            if user_id == me.id {
                                continue;
                            }

                            if is_scammer {
                                info!("Scammer detected: {user_id}");
                                let sanction = fs::read_to_string("res/scam_account.txt")?;
                                send_sanction(&message_to_send_tx, message, sanction, app_data.client_id).await?;
                                continue;
                            }
                        }

                        if is_scam_message(&message)? {
                            info!("Scam message detected");
                            let sanction = fs::read_to_string("res/message.txt")?;
                            send_sanction(&message_to_send_tx, message, sanction, app_data.client_id).await?;
                            continue;
                        }

                        Ok(())
                    } as FetishResult<()>;

                    if let Err(e) = failsafe {
                        error!("Message handling error: {e:#?}");
                    }
                },
                _ = app_data.shutdown_rx.recv() => {
                    debug!("Received shutdown signal");
                    break;
                }
            }
        }
        debug!("Waiting for message sender to finish");
        message_sender_handle.await?;
        info!("Stop listening for messages");
        Ok(app_data)
    }
}

fn is_scammer_account(
    db: Arc<Mutex<Database>>,
    message: &Message,
) -> FetishResult<Option<(i64, bool)>> {
    Ok(match message.sender_id {
        MessageSender::User(MessageSenderUser { user_id }) => Some((
            user_id,
            db.lock().unwrap().load::<Scammer>(user_id)?.is_some(),
        )),
        _ => None,
    })
}

fn is_scam_message(message: &Message) -> FetishResult<bool> {
    match &message.content {
        MessageContent::MessageText(message_text) => {
            let text = message_text.text.text.clone();
            info!(
                "{}: {}{}",
                message.chat_id,
                &text[0..30],
                if text.len() > 30 { "..." } else { "" }
            );
            let text = unidecode(text.to_uppercase().as_str());
            let is_scam = serde_json::from_str::<Vec<String>>(
                fs::read_to_string("res/keywords.json")?.as_str(),
            )?
            .into_iter()
            .any(|keyword| text.contains(&keyword));
            Ok(is_scam)
        }
        _ => {
            trace!("{:#?}", message.content);
            Ok(false)
        }
    }
}

async fn send_sanction(
    message_to_send_tx: &tokio::sync::mpsc::UnboundedSender<SendMessageData>,
    message: Message,
    sanction: String,
    client_id: i32,
) -> FetishResult<()> {
    message_to_send_tx
        .send((
            message,
            InputMessageContent::InputMessageText(InputMessageText {
                text: FormattedText {
                    text: sanction,
                    entities: vec![],
                },
                disable_web_page_preview: true,
                clear_draft: false,
            }),
            client_id,
        ))
        .map_err(|_| FetishError::MessageHandle)?;
    Ok(())
}

mod message_sender {
    use std::time;

    use log::{debug, error, info};
    use rand::Rng;
    use tdlib::{enums::MessageReplyTo, types::MessageReplyToMessage};
    use tokio::sync::{broadcast, mpsc};

    use super::SendMessageData;

    pub async fn run(
        mut shutdown_rx: broadcast::Receiver<()>,
        mut message_to_send_rx: mpsc::UnboundedReceiver<SendMessageData>,
    ) {
        debug!("Starting message sender");

        loop {
            tokio::select! {
                Some(send_message_data) = message_to_send_rx.recv() => {
                    let (message, input_message, client_id) = send_message_data;
                    info!("Sending message");
                    debug!("{:#?}", message.content);
                    let (min, max) = (3000, 6000);
                    let waiting_time = (rand::thread_rng().gen::<f64>() * (max - min) as f64) as u64 + min;
                    info!("Waiting for {waiting_time} ms");
                    tokio::time::sleep(time::Duration::from_millis(waiting_time)).await;
                    if let Err(e) = tdlib::functions::send_message(
                        message.chat_id,
                        message.message_thread_id,
                        Some(MessageReplyTo::Message(MessageReplyToMessage {
                            chat_id: message.chat_id,
                            message_id: message.id,
                        })),
                        None,
                        input_message,
                        client_id,
                    )
                    .await
                    {
                        error!("Failed to send message: {e:#?}");
                    }
                }
                _ = shutdown_rx.recv() => {
                    debug!("Shutting down message sender");
                    break;
                }
            }
        }
    }
}
