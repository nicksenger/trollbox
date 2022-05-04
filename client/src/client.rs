use futures::StreamExt;
use iced::Subscription;
use std::time::{Duration, Instant};
use tonic::transport::Channel;

use schema::trollbox::troll_box_client::TrollBoxClient;
use schema::trollbox::{SendMessageRequest, StreamMessagesRequest};
use schema::SendMessageError;

use crate::Message;

const RECONNECT_DELAY: Duration = Duration::from_secs(5);

#[cfg(feature = "local")]
const SERVER_ADDRESS: &str = "http://[::0]:50051";
#[cfg(not(feature = "local"))]
const SERVER_ADDRESS: &str = "http://137.184.212.135:50051";

#[derive(Clone, Debug)]
pub struct Client {
    client: TrollBoxClient<Channel>,
}

impl Client {
    pub fn connect() -> Subscription<Message> {
        Subscription::from_recipe(Worker)
    }

    pub fn send_message(
        &mut self,
        text: String,
    ) -> impl futures::Future<Output = Vec<schema::SendMessageError>> {
        let mut client = self.client.clone();

        async move {
            let response = client
                .send_message(SendMessageRequest {
                    alias: std::env::var("ALIAS").unwrap_or_else(|_| "Anonymous".to_string()),
                    message: text,
                })
                .await;

            match response {
                Ok(response) => {
                    let response = response.into_inner();
                    response
                        .errors
                        .into_iter()
                        .map(SendMessageError::from)
                        .collect::<Vec<_>>()
                }
                Err(status) => vec![SendMessageError::Unknown(status.message().to_string())],
            }
        }
    }
}

pub struct Worker;

impl<I> iced_native::subscription::Recipe<iced_native::Hasher, I> for Worker {
    type Output = Message;

    fn hash(&self, state: &mut iced_native::Hasher) {
        use std::hash::Hash;

        struct Marker;
        std::any::TypeId::of::<Marker>().hash(state);
    }

    fn stream(
        self: Box<Self>,
        _input: futures::stream::BoxStream<'static, I>,
    ) -> futures::stream::BoxStream<'static, Self::Output> {
        futures::stream::unfold(
            State::Disconnected {
                last_connection_attempt_at: None,
            },
            |state| async { state.run().await },
        )
        .filter_map(|message| async { message })
        .boxed()
    }
}

enum State {
    Connected {
        message_stream: tonic::Streaming<schema::trollbox::Message>,
    },
    Disconnected {
        last_connection_attempt_at: Option<std::time::Instant>,
    },
}

impl State {
    fn disconnected() -> Self {
        Self::Disconnected {
            last_connection_attempt_at: Some(Instant::now()),
        }
    }
}

impl State {
    async fn run(self) -> Option<(Option<Message>, State)> {
        match self {
            Self::Connected { mut message_stream } => {
                match message_stream.next().await.expect("await message stream") {
                    Ok(message) => Some((
                        Some(Message::TrollBoxMessageReceived(message.into())),
                        State::Connected { message_stream },
                    )),
                    Err(_) => Some((Some(Message::Disconnected), State::disconnected())),
                }
            }

            Self::Disconnected {
                last_connection_attempt_at,
            } => {
                if let Some(time) = last_connection_attempt_at {
                    tokio::time::sleep(
                        RECONNECT_DELAY.saturating_sub(Instant::now().duration_since(time)),
                    )
                    .await
                }

                match TrollBoxClient::connect(SERVER_ADDRESS).await {
                    Ok(mut client) => {
                        if let Ok(response) =
                            client.messages(StreamMessagesRequest::default()).await
                        {
                            Some((
                                Some(Message::Connected(Client { client })),
                                State::Connected {
                                    message_stream: response.into_inner(),
                                },
                            ))
                        } else {
                            Some((None, State::disconnected()))
                        }
                    }
                    Err(_error) => {
                        #[cfg(feature = "debug")]
                        println!("Connection failed: {:?}", _error);

                        Some((None, State::disconnected()))
                    }
                }
            }
        }
    }
}
