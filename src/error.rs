use std::fmt::Display;

use crate::lexer;

#[derive(Debug, PartialEq, Clone)]
pub enum RespErrorType {
    None,
    Other,
    NewLineMissing,
    InvalidStart,
    InvalidData,
    InvalidInteger,
    InvalidSize,
    Message(String),
}

#[derive(Debug, PartialEq, Clone)]
pub struct OwnedParseError {
    pub token: Option<lexer::OwnedToken>,
    pub error_type: RespErrorType,
}

impl OwnedParseError {
    pub fn message(input: String) -> OwnedParseError {
        OwnedParseError {
            token: None,
            error_type: RespErrorType::Message(input),
        }
    }
}

impl Display for OwnedParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(f, "{:?}", self.error_type)
    }
}

impl std::error::Error for OwnedParseError {}

#[derive(Debug, PartialEq, Clone)]
pub struct ParseError<'a> {
    pub token: Option<lexer::Token<'a>>,
    pub error_type: RespErrorType,
}

impl<'a> ParseError<'a> {
    pub fn message(input: String) -> ParseError<'a> {
        ParseError {
            token: None,
            error_type: RespErrorType::Message(input),
        }
    }

    pub fn to_owned(&self) -> OwnedParseError {
        let mut error = OwnedParseError {
            token: None,
            error_type: self.error_type.clone(),
        };

        if let Some(token) = &self.token {
            error.token = Some(token.to_owned())
        };

        error
    }
}

impl<'a> Display for ParseError<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(f, "{:?}", self.error_type)
    }
}

impl<'a> std::error::Error for ParseError<'a> {}

#[derive(Debug)]
pub enum FormatError {
    NestedDataNotAllowed,
    ProtocolError,
    Custom(String),
}

impl Display for FormatError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            FormatError::NestedDataNotAllowed => formatter.write_str(
                "data cannot be encoded by given protocol, because it doesnt allow nested data",
            ),
            FormatError::ProtocolError => {
                formatter.write_str("data cannot be encoded by given protocol")
            }
            FormatError::Custom(message) => formatter.write_str(message),
        }
    }
}

impl std::error::Error for FormatError {}

impl From<std::io::Error> for FormatError {
    fn from(error: std::io::Error) -> FormatError {
        FormatError::Custom(error.to_string())
    }
}
