use serde::{de, ser};
use std::fmt::Display;

pub type DeserializerResult<'a, T> = std::result::Result<T, DeserializerError<'a>>;
pub type SerializerResult<T> = std::result::Result<T, SerializerError>;

pub type DeserializerError<'a> = crate::ParseError<'a>;
// pub type SerializerError = String;

#[derive(Debug)]
pub struct SerializerError(String);

impl ser::Error for SerializerError {
    fn custom<T: Display>(msg: T) -> Self {
        SerializerError(msg.to_string())
    }
}

impl<'a> de::Error for DeserializerError<'a> {
    fn custom<T: Display>(msg: T) -> Self {
        DeserializerError::message(msg.to_string())
    }
}

impl Display for SerializerError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str(&self.0)
    }
}

impl std::error::Error for SerializerError {}
