use async_mutex::Mutex;
use futures::channel::mpsc;
use futures::stream::select;
use futures::{SinkExt, Stream, StreamExt};
use std::collections::{HashMap, VecDeque};
use std::pin::Pin;
use std::sync::Arc;
use tonic::{transport::Server, Request, Response, Status};
use uuid::Uuid;

use schema::trollbox::troll_box_server::{TrollBox, TrollBoxServer};
use schema::trollbox::{self, SendMessageRequest, SendMessageResponse, StreamMessagesRequest};
use schema::Message;

const MESSAGE_CAPACITY: usize = 100;

enum Event {
    Message(Message),
    Subscribe((Uuid, mpsc::Sender<Message>)),
    Unsubscribe(Uuid),
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr = "[::0]:50051".parse()?;

    let mut message_buffer = MessageBuffer::default();
    let (message_sender, mut message_receiver) = mpsc::channel(128);
    let (subscribe_sender, mut subscribe_receiver) = mpsc::channel(128);
    let (unsubscribe_sender, mut unsubscribe_receiver) = mpsc::channel(128);

    let service = TrollBoxService {
        message_buffer: message_buffer.clone(),
        message_sender,
        subscribe_sender,
        unsubscribe_sender,
    };

    let (_, _) = futures::join!(
        Server::builder()
            .add_service(TrollBoxServer::new(service))
            .serve(addr),
        async move {
            let mut subscriptions: HashMap<Uuid, mpsc::Sender<Message>> = HashMap::new();

            loop {
                let event = {
                    let message = message_receiver.by_ref().map(Event::Message);
                    let subscribe = subscribe_receiver.by_ref().map(Event::Subscribe);
                    let unsubscribe = unsubscribe_receiver.by_ref().map(Event::Unsubscribe);

                    let mut select = select(select(subscribe, unsubscribe), message);

                    select.next().await.expect("event input")
                };

                match event {
                    Event::Message(message) => {
                        for sender in subscriptions.values_mut() {
                            let _ = sender.try_send(message.clone());
                        }

                        message_buffer.push(message.clone()).await;
                    }
                    Event::Subscribe((id, sender)) => {
                        subscriptions.insert(id, sender);
                    }
                    Event::Unsubscribe(id) => {
                        subscriptions.remove(&id);
                    }
                }
            }
        }
    );

    Ok(())
}

pub struct TrollBoxService {
    message_buffer: MessageBuffer,
    message_sender: mpsc::Sender<Message>,
    subscribe_sender: mpsc::Sender<(Uuid, mpsc::Sender<Message>)>,
    unsubscribe_sender: mpsc::Sender<Uuid>,
}

#[tonic::async_trait]
impl TrollBox for TrollBoxService {
    type MessagesStream = Pin<Box<dyn Stream<Item = Result<trollbox::Message, Status>> + Send>>;

    async fn send_message(
        &self,
        request: Request<SendMessageRequest>,
    ) -> Result<Response<SendMessageResponse>, Status> {
        match Message::try_from(request.into_inner()) {
            Err(e) => Ok(Response::new(SendMessageResponse {
                errors: vec![e.into()],
            })),
            Ok(message) => {
                let errors: Vec<schema::trollbox::SendMessageError> =
                    match self.message_sender.clone().send(message.clone()).await {
                        Ok(_) => vec![],
                        Err(_) => {
                            vec![
                                schema::SendMessageError::Unknown("internal error".to_string())
                                    .into(),
                            ]
                        }
                    };

                Ok(Response::new(SendMessageResponse { errors }))
            }
        }
    }

    async fn messages(
        &self,
        _request: Request<StreamMessagesRequest>,
    ) -> Result<Response<Self::MessagesStream>, Status> {
        let (subscription_sender, mut subscription_receiver) = mpsc::channel(128);

        let messages = self.message_buffer.messages().await;
        for message in messages {
            let _ = subscription_sender.clone().try_send(message.clone());
        }

        let subscription_id = Uuid::new_v4();
        let _ = self
            .subscribe_sender
            .clone()
            .try_send((subscription_id, subscription_sender));

        let (mut tx, rx) = mpsc::channel(128);
        let mut unsubscribe_sender = self.unsubscribe_sender.clone();

        tokio::spawn(async move {
            while let Some(message) = subscription_receiver.next().await {
                match tx
                    .send(Result::<_, Status>::Ok(trollbox::Message::from(message)))
                    .await
                {
                    Ok(_) => {}
                    Err(_item) => {
                        break;
                    }
                }
            }

            let _ = unsubscribe_sender.try_send(subscription_id);
        });

        Ok(Response::new(Box::pin(rx) as Self::MessagesStream))
    }
}

#[derive(Clone, Default)]
struct MessageBuffer {
    inner: Arc<Mutex<VecDeque<Message>>>,
}

impl MessageBuffer {
    async fn push(&mut self, message: Message) {
        let mut inner = self.inner.lock().await;

        if inner.len() >= MESSAGE_CAPACITY {
            inner.pop_front();
        }

        inner.push_back(message);
    }

    async fn messages(&self) -> Vec<Message> {
        self.inner.lock().await.iter().cloned().collect()
    }
}
