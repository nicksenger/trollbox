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
                let scrollable = container(
                    scrollable(messages.iter().fold(column(), |column, message| {
                        column.push(text(format!("{}: {}", message.alias, message.text)))
                    }))
                    .style(style::Style),
                )
                .height(Length::Fill);

                let input = text_input("Enter a message", input_value, Message::InputValueChanged)
                    .on_submit(Message::SendTrollBoxMessage)
                    .style(style::Style);

                container(column().push(scrollable).push(input))
                    .style(style::Style)
                    .padding(5)
                    .into()
            }
            Self::Disconnected => text("Connecting to the trollbox...").into(),
        }
    }

    fn subscription(&self) -> Subscription<Message> {
        Client::connect()
    }
}

mod style {
    use iced::container::StyleSheet as ContainerStyleSheet;
    use iced::scrollable::{Scrollbar, Scroller, StyleSheet as ScrollableStyleSheet};
    use iced::text_input::{Style as InputStyle, StyleSheet as InputStyleSheet};
    use iced::Color;

    pub struct Style;

    impl ContainerStyleSheet for Style {
        fn style(&self) -> iced::container::Style {
            iced::container::Style {
                text_color: Some(Color::WHITE),
                background: Some(iced::Background::Color(Color::BLACK)),
                ..Default::default()
            }
        }
    }

    impl InputStyleSheet for Style {
        fn active(&self) -> InputStyle {
            InputStyle {
                background: iced::Background::Color(Color::BLACK),
                border_radius: 0.0,
                border_width: 1.0,
                border_color: Color::from_rgb(0.5, 0.5, 0.5),
            }
        }

        fn focused(&self) -> InputStyle {
            InputStyle {
                border_color: Color::WHITE,
                ..InputStyleSheet::active(self)
            }
        }

        fn placeholder_color(&self) -> Color {
            Color::from_rgb(0.5, 0.5, 0.5)
        }

        fn value_color(&self) -> Color {
            Color::WHITE
        }

        fn selection_color(&self) -> Color {
            Color::from_rgb(0.5, 0.5, 0.5)
        }
    }

    impl ScrollableStyleSheet for Style {
        /// Produces the style of an active scrollbar.
        fn active(&self) -> Scrollbar {
            Scrollbar {
                background: None,
                border_radius: 0.0,
                border_width: 0.0,
                border_color: Color::TRANSPARENT,
                scroller: Scroller {
                    color: [0.0, 0.62, 0.32, 1.0].into(),

                    border_radius: 5.0,
                    border_width: 0.0,
                    border_color: Color::TRANSPARENT,
                },
            }
        }

        /// Produces the style of an hovered scrollbar.
        fn hovered(&self) -> Scrollbar {
            Scrollbar {
                background: None,
                border_radius: 0.0,
                border_width: 0.0,
                border_color: Color::TRANSPARENT,
                scroller: Scroller {
                    color: [0.9, 0.0, 0.16, 1.0].into(),
                    border_radius: 5.0,
                    border_width: 0.0,
                    border_color: Color::TRANSPARENT,
                },
            }
        }

        /// Produces the style of a scrollbar that is being dragged.
        fn dragging(&self) -> Scrollbar {
            ScrollableStyleSheet::hovered(self)
        }
    }
}
