use std::fs;

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
    error::{FetishError, FetishResult},
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
                    match message.sender_id {
                        MessageSender::User(MessageSenderUser { user_id }) => if user_id == me.id { continue; },
                        _ => ()
                    }
                    if message.chat_id >= 0 { continue; }
                    if let Err(e) = handle_message(&message_to_send_tx, message, app_data.client_id).await {
                        error!("Failed to handle message: {e:#?}");
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

async fn handle_message(
    message_to_send_tx: &tokio::sync::mpsc::UnboundedSender<SendMessageData>,
    message: Message,
    client_id: i32,
) -> FetishResult<()> {
    match &message.content {
        MessageContent::MessageText(message_text) => {
            // Debug message.chat_id == -4162067427
            let text = message_text.text.text.clone();
            info!("{}: {text}", message.chat_id);
            let text = unidecode(text.to_uppercase().as_str());
            let sanction = fs::read_to_string("res/message.txt")?;
            if serde_json::from_str::<Vec<String>>(
                fs::read_to_string("res/keywords.json")?.as_str(),
            )?
            .into_iter()
            .any(|keyword| text.contains(&keyword))
            {
                message_to_send_tx
                    .send((
                        message,
                        InputMessageContent::InputMessageText(InputMessageText {
                            text: FormattedText {
                                text: sanction.into(),
                                entities: vec![],
                            },
                            disable_web_page_preview: true,
                            clear_draft: false,
                        }),
                        client_id,
                    ))
                    .map_err(|_| FetishError::MessageHandle)?;
            }
        }
        _ => {
            trace!("{:#?}", message.content);
        }
    }
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
