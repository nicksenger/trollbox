use iced::alignment::{Alignment, Horizontal, Vertical};
use iced::executor;
use iced::pure::{column, container, row, text, text_input, Application, Element};
use iced::{Command, Length, Settings, Subscription};

mod utilities;

use utilities::scrollable;
use utilities::Client;

pub fn main() -> iced::Result {
    State::run(Settings::default())
}

enum State {
    Disconnected,
    Connected {
        client: Client,
        input_value: String,
        messages: Vec<schema::Message>,
    },
}

#[derive(Clone, Debug)]
pub enum Message {
    Connected(Client),
    Disconnected,
    InputChanged(String),
    SendTrollBoxMessage(String),
    SendTrollBoxMessageSuccess,
    SendTrollBoxMessageFailure,
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
            Message::Disconnected => *self = Self::Disconnected,

            Message::Connected(client) => {
                *self = Self::Connected {
                    client,
                    input_value: String::new(),
                    messages: vec![],
                };
            }

            Message::InputChanged(s) => {
                if let Self::Connected { input_value, .. } = self {
                    *input_value = s;
                }
            }

            Message::SendTrollBoxMessage(text) => {
                if let Self::Connected {
                    client,
                    input_value,
                    ..
                } = self
                {
                    *input_value = String::new();

                    return Command::perform(client.send_message(text), |errors| {
                        if errors.is_empty() {
                            Message::SendTrollBoxMessageSuccess
                        } else {
                            Message::SendTrollBoxMessageFailure
                        }
                    });
                }
            }

            Message::TrollBoxMessageReceived(troll_box_message) => {
                if let Self::Connected { messages, .. } = self {
                    messages.push(troll_box_message);
                }
            }

            Message::SendTrollBoxMessageSuccess | Message::SendTrollBoxMessageFailure => {}
        }

        Command::none()
    }

    fn view(&self) -> Element<Message> {
        match self {
            Self::Disconnected => container(text("Connecting to the Trollbox.."))
                .style(style::Style)
                .height(Length::Fill)
                .width(Length::Fill)
                .align_x(Horizontal::Center)
                .align_y(Vertical::Center)
                .into(),
            Self::Connected {
                messages,
                input_value,
                ..
            } => container(
                column().spacing(10).push(chat_body(&messages)).push(
                    text_input("Message #trollbox", &input_value, Message::InputChanged)
                        .on_submit(Message::SendTrollBoxMessage(input_value.clone()))
                        .style(style::Style),
                ),
            )
            .style(style::Style)
            .padding(10)
            .into(),
        }
    }

    fn subscription(&self) -> Subscription<Message> {
        Client::connect()
    }
}

fn chat_body(messages: &Vec<schema::Message>) -> Element<Message> {
    scrollable(
        messages
            .iter()
            .fold(column().spacing(20), |col, m| col.push(trollbox_message(m)))
            .width(Length::Fill),
    )
    .height(Length::Fill)
    .style(style::Style)
    .into()
}

fn trollbox_message(message: &schema::Message) -> Element<Message> {
    column()
        .spacing(5)
        .push(
            row()
                .spacing(5)
                .align_items(Alignment::End)
                .push(container(text(&message.alias).size(24)).height(Length::Units(24)))
                .push(
                    container(
                        text(&message.sent_at.format("%A, %d %b %Y %l:%M %p").to_string()).size(14),
                    )
                    .height(Length::Units(16)),
                ),
        )
        .push(row().push(text(&message.text)))
        .into()
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
