use serde::{
    de,
    ser::{self, Error},
};
use std::fmt::Display;

pub type DeserializerResult<'a, T> = std::result::Result<T, DeserializerError>;
pub type SerializerResult<T> = std::result::Result<T, SerializerError>;

pub type DeserializerError = crate::OwnedParseError;
// pub type SerializerError = String;

#[derive(Debug)]
pub struct SerializerError(String);

impl ser::Error for SerializerError {
    fn custom<T: Display>(msg: T) -> Self {
        SerializerError(msg.to_string())
    }
}

impl de::Error for DeserializerError {
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

impl From<std::io::Error> for SerializerError {
    fn from(error: std::io::Error) -> SerializerError {
        SerializerError::custom(error)
    }
}
