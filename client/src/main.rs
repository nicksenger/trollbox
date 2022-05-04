use iced::executor;
use iced::pure::{column, container, scrollable, text, text_input, Application, Element};
use iced::{Command, Length, Settings, Subscription};

mod client;

use client::Client;

pub fn main() -> iced::Result {
    State::run(Settings::default())
}

enum State {
    Disconnected,
    Connected {
        client: Client,
        messages: Vec<schema::Message>,
        input_value: String,
    },
}

#[derive(Clone, Debug)]
pub enum Message {
    Connected(Client),
    Disconnected,
    TrollBoxMessageReceived(schema::Message),
    InputValueChanged(String),
    SendTrollBoxMessage,
    SendMessageSuccess,
    SendMessageFailure,
}

impl Application for State {
    type Executor = executor::Default;
    type Message = Message;
    type Flags = ();

    fn new(_flags: ()) -> (Self, Command<Message>) {
        (Self::Disconnected, Command::none())
    }

    fn title(&self) -> String {
        String::from("Trollbox - Iced Workshop")
    }

    fn update(&mut self, message: Message) -> Command<Message> {
        match message {
            Message::Connected(client) => {
                *self = State::Connected {
                    client,
                    messages: vec![],
                    input_value: String::new(),
                }
            }
            Message::Disconnected => *self = State::Disconnected,
            Message::TrollBoxMessageReceived(message) => {
                if let State::Connected { messages, .. } = self {
                    messages.push(message);
                }
            }
            Message::InputValueChanged(value) => {
                if let State::Connected { input_value, .. } = self {
                    *input_value = value;
                }
            }
            Message::SendTrollBoxMessage => {
                if let State::Connected {
                    client,
                    input_value,
                    ..
                } = self
                {
                    let future = client.send_message(input_value.to_string());

                    *input_value = String::new();

                    return Command::perform(future, |errors| {
                        if errors.is_empty() {
                            Message::SendMessageSuccess
                        } else {
                            Message::SendMessageFailure
                        }
                    });
                }
            }
            Message::SendMessageSuccess | Message::SendMessageFailure => {}
        }

        Command::none()
    }

    fn view(&self) -> Element<Message> {
        match self {
            Self::Connected {
                messages,
                input_value,
                ..
            } => {
                let scrollable = container(scrollable(messages.iter().fold(
                    column(),
                    |column, message| {
                        column.push(text(format!("{}: {}", message.alias, message.text)))
                    },
                )))
                .height(Length::Fill);

                let input = text_input("Enter a message", input_value, Message::InputValueChanged)
                    .on_submit(Message::SendTrollBoxMessage);

                column().push(scrollable).push(input).padding(5).into()
            }
            Self::Disconnected => text("Connecting to the trollbox...").into(),
        }
    }

    fn subscription(&self) -> Subscription<Message> {
        Client::connect()
    }
}
