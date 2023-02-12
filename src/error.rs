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
