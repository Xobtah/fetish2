use async_trait::async_trait;
use log::{debug, error, info, trace};
use tdlib::{
    enums::{InputMessageContent, MessageContent, MessageSender, User},
    functions,
    types::{FormattedText, InputMessageText, Message, MessageSenderUser},
};

use crate::{
    app::closing_state::ClosingState,
    error::{FetishError, FetishResult},
};

use super::{ApplicationData, ApplicationState};

pub type SendMessageData = (Message, InputMessageContent, i32);

pub struct MessageState {
    client_data: ApplicationData,
    message_sender_handle: tokio::task::JoinHandle<()>,
    message_to_send_tx: tokio::sync::mpsc::UnboundedSender<SendMessageData>,
}

#[async_trait]
impl ApplicationState for MessageState {
    async fn run(mut self: Box<Self>) -> FetishResult<Option<Box<dyn ApplicationState>>> {
        info!("Start listening for messages");
        let User::User(me) = functions::get_me(self.client_data.client_id).await.unwrap();

        loop {
            tokio::select! {
                Some(message) = self.client_data.message_rx.recv() => {
                    match message.sender_id {
                        MessageSender::User(MessageSenderUser { user_id }) => if user_id == me.id { continue; },
                        _ => ()
                    }
                    if let Err(e) = self.handle_message(message).await {
                        error!("Failed to handle message: {e:#?}");
                    }
                },
                _ = self.client_data.shutdown_rx.recv() => {
                    debug!("Received shutdown signal");
                    break;
                }
            }
        }
        debug!("Waiting for message sender to finish");
        self.message_sender_handle.await?;
        info!("Stop listening for messages");
        Ok(Some(Box::new(ClosingState::new(self.client_data))))
    }
}

impl MessageState {
    pub fn new(client_data: ApplicationData) -> Self {
        let (message_to_send_tx, message_to_send_rx) = tokio::sync::mpsc::unbounded_channel();
        let shutdown_rx = client_data.shutdown_rx.resubscribe();
        Self {
            client_data,
            message_sender_handle: tokio::spawn(message_sender::run(
                shutdown_rx,
                message_to_send_rx,
            )),
            message_to_send_tx,
        }
    }

    async fn handle_message(&self, message: Message) -> FetishResult<()> {
        match &message.content {
            MessageContent::MessageText(message_text) => {
                info!("{}: {}", message.chat_id, message_text.text.text);
                if message.chat_id == -4162067427 {
                    self.message_to_send_tx
                        .send((
                            message,
                            InputMessageContent::InputMessageText(InputMessageText {
                                text: FormattedText {
                                    text: "This message is sent from a bot".into(),
                                    entities: vec![],
                                },
                                disable_web_page_preview: true,
                                clear_draft: false,
                            }),
                            self.client_data.client_id,
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
}

mod message_sender {
    use std::time;

    use log::{debug, error, info, trace};
    use tdlib::{enums::MessageReplyTo, types::MessageReplyToMessage};
    use rand::Rng;
    use tokio::sync::{broadcast, mpsc};

    use super::SendMessageData;

    // struct Chrono {
    //     start: time::Instant,
    //     duration: time::Duration,
    // }

    // impl Chrono {
    //     fn init() -> Self {
    //         Self {
    //             start: time::Instant::now(),
    //             duration: Default::default(),
    //         }
    //     }

    //     fn start(&mut self, duration: time::Duration) -> Self {
    //         trace!("Starting chrono for {duration:#?}");
    //         Self {
    //             start: time::Instant::now(),
    //             duration,
    //         }
    //     }

    //     fn is_elapsed(&self) -> bool {
    //         trace!(":) {:#?}", self.start.elapsed());
    //         self.start.elapsed() >= self.duration
    //     }
    // }

    pub async fn run(
        mut shutdown_rx: broadcast::Receiver<()>,
        mut message_to_send_rx: mpsc::UnboundedReceiver<SendMessageData>,
    ) {
        debug!("Starting message sender");
        // let mut chrono = Chrono::init();

        loop {
            tokio::select! {
                Some(send_message_data) = message_to_send_rx.recv()/*, if chrono.is_elapsed()*/ => {
                    let (message, input_message, client_id) = send_message_data;
                    info!("Sending message");
                    debug!("{:#?}", message.content);
                    let (min, max) = (1000, 3000);
                    let waiting_time = (rand::thread_rng().gen::<f64>() * (max - min) as f64) as u64 + min;
                    trace!("Waiting for {waiting_time} ms");
                    tokio::time::sleep(time::Duration::from_millis(waiting_time)).await;
                    if let Err(e) = tdlib::functions::send_message(
                        message.chat_id,
                        message.message_thread_id,
                        Some(MessageReplyTo::Message(MessageReplyToMessage {
                            chat_id: message.chat_id,
                            message_id: message.id,
                        })),
                        None,
                        None,
                        input_message,
                        client_id,
                    )
                    .await
                    {
                        error!("Failed to send message: {e:#?}");
                    }
                    // chrono = chrono.start(time::Duration::from_millis(waiting_time));
                }
                _ = shutdown_rx.recv() => {
                    debug!("Shutting down message sender");
                    break;
                }
            }
        }
    }
}
