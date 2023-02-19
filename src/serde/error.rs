use serde::{de, ser};
use std::fmt::Display;

pub type DeserializerResult<'a, T> = std::result::Result<T, DeserializerError>;
pub type SerializerResult<T> = std::result::Result<T, SerializerError>;

pub type DeserializerError = crate::OwnedParseError;
pub type SerializerError = crate::FormatError;

impl ser::Error for SerializerError {
    fn custom<T: Display>(msg: T) -> Self {
        crate::FormatError::Custom(msg.to_string())
    }
}

impl de::Error for DeserializerError {
    fn custom<T: Display>(msg: T) -> Self {
        DeserializerError::message(msg.to_string())
    }
}
