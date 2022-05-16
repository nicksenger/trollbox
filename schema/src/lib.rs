use chrono::{DateTime, Local, TimeZone};
use std::time::{SystemTime, UNIX_EPOCH};
use uuid::Uuid;

pub use gen::trollbox;

mod gen {
    pub mod trollbox;
}

#[derive(Clone, Debug)]
pub struct Message {
    pub id: Uuid,
    pub alias: String,
    pub text: String,
    pub sent_at: DateTime<Local>,
}

impl From<trollbox::Message> for Message {
    fn from(message: trollbox::Message) -> Self {
        let (seconds, nanos) = message
            .timestamp
            .map(|ts| (ts.seconds, ts.nanos as u32))
            .expect("missing timestamp");

        Message {
            id: Uuid::parse_str(&message.id).expect("bad uuid"),
            alias: message.alias,
            text: message.text,
            sent_at: Local.timestamp(seconds, nanos),
        }
    }
}

impl From<Message> for trollbox::Message {
    fn from(message: Message) -> Self {
        trollbox::Message {
            id: message.id.to_string(),
            alias: message.alias,
            text: message.text,
            timestamp: Some(prost_types::Timestamp {
                seconds: message.sent_at.timestamp(),
                nanos: message.sent_at.timestamp_subsec_nanos() as i32,
            }),
        }
    }
}

impl TryFrom<trollbox::SendMessageRequest> for Message {
    type Error = SendMessageError;

    fn try_from(request: trollbox::SendMessageRequest) -> Result<Self, Self::Error> {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("time went backwards");

        if request.message.is_empty() {
            Err(SendMessageError::MissingMessage)
        } else if request.message.len() > 256 {
            Err(SendMessageError::MessageTooLong)
        } else if request.alias.is_empty() {
            Err(SendMessageError::MissingAlias)
        } else if request.alias.len() > 32 {
            Err(SendMessageError::AliasTooLong)
        } else {
            Ok(Message {
                id: Uuid::new_v4(),
                alias: request.alias,
                text: request.message,
                sent_at: Local.timestamp(timestamp.as_secs() as i64, timestamp.subsec_nanos()),
            })
        }
    }
}

pub enum SendMessageError {
    Unknown(String),
    MissingAlias,
    MissingMessage,
    AliasTooLong,
    MessageTooLong,
}

impl From<SendMessageError> for trollbox::SendMessageError {
    fn from(e: SendMessageError) -> Self {
        use trollbox::send_message_error::Kind::*;

        trollbox::SendMessageError {
            kind: Some(match e {
                SendMessageError::MessageTooLong => MessageTooLong(()),
                SendMessageError::MissingMessage => MissingMessage(()),
                SendMessageError::AliasTooLong => AliasTooLong(()),
                SendMessageError::MissingAlias => MissingAlias(()),
                SendMessageError::Unknown(message) => Unknown(trollbox::UnknownError { message }),
            }),
        }
    }
}

impl From<trollbox::SendMessageError> for SendMessageError {
    fn from(e: trollbox::SendMessageError) -> Self {
        use trollbox::send_message_error::Kind::*;

        match e.kind {
            Some(MessageTooLong(())) => Self::MessageTooLong,
            Some(MissingMessage(())) => Self::MissingMessage,
            Some(AliasTooLong(())) => Self::AliasTooLong,
            Some(MissingAlias(())) => Self::MissingAlias,
            Some(Unknown(trollbox::UnknownError { message })) => Self::Unknown(message),
            None => Self::Unknown("Failed".to_string()),
        }
    }
}
