use iced::executor;
use iced::pure::{text, Application, Element};
use iced::{Command, Settings, Subscription};

mod client;

use client::Client;

pub fn main() -> iced::Result {
    State::run(Settings::default())
}

enum State {
    Disconnected,
    Connected,
}

#[derive(Clone, Debug)]
pub enum Message {
    Connected(Client),
    Disconnected,
    TrollBoxMessageReceived(schema::Message),
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
            Message::Connected(_) => *self = State::Connected,
            Message::Disconnected => *self = State::Disconnected,
            Message::TrollBoxMessageReceived(message) => {
                println!("{:?}", message);
            }
        }

        Command::none()
    }

    fn view(&self) -> Element<Message> {
        match self {
            Self::Connected => text("Connected to the trollbox!"),
            Self::Disconnected => text("Connecting to the trollbox..."),
        }
        .into()
    }

    fn subscription(&self) -> Subscription<Message> {
        Client::connect()
    }
}
