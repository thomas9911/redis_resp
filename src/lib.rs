/// In RESP, the first byte determines the data type:
/// For Simple Strings, the first byte of the reply is "+"
/// For Errors, the first byte of the reply is "-"
/// For Integers, the first byte of the reply is ":"
/// For Bulk Strings, the first byte of the reply is "$"
/// For Arrays, the first byte of the reply is "*"
///
///
///
pub mod formatter;
pub mod lexer;
pub mod parser;
pub mod resp_type;
pub mod value;

use std::fmt::Display;

pub use lexer::Lexer;
pub use parser::Parser;
pub use resp_type::{RespType, RespTypeRef};
pub use value::Value;

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum RespErrorType {
    None,
    Other,
    NewLineMissing,
    InvalidStart,
    InvalidData,
    InvalidInteger,
    InvalidSize,
}

#[derive(Debug, PartialEq, Clone)]
pub struct ParseError<'a> {
    token: Option<lexer::Token<'a>>,
    error_type: RespErrorType,
}

impl<'a> Display for ParseError<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(f, "{:?}", self.error_type)
    }
}

impl<'a> std::error::Error for ParseError<'a> {}

pub fn bytes_to_value(data: &[u8]) -> Result<Result<Value, Value>, ParseError> {
    Ok(bytes_to_resp_type(data)?.into_value())
}

pub fn bytes_to_resp_type(data: &[u8]) -> Result<RespType, ParseError> {
    Ok(Parser::new_from_bytes(data.as_ref()).parse()?.to_owned())
}

pub fn bytes_to_resp_type_ref<'a>(data: &'a [u8]) -> Result<RespTypeRef<'a>, ParseError> {
    Ok(Parser::new_from_bytes(data.as_ref()).parse()?)
}

#[test]
fn convert_text_to_value() {
    let result = bytes_to_value(b"$14\r\njust some text\r\n").unwrap();
    assert_eq!(result, Ok(Value::String("just some text".to_string())))
}

#[test]
fn convert_text_to_resp_type() {
    let result = bytes_to_resp_type(b"$14\r\njust some text\r\n").unwrap();
    assert_eq!(result, RespType::BulkString(b"just some text".to_vec()))
}

#[test]
fn convert_text_to_resp_type_ref() {
    let result = bytes_to_resp_type_ref(b"$14\r\njust some text\r\n").unwrap();
    assert_eq!(result, RespTypeRef::BulkString(b"just some text"))
}

#[test]
fn handle_error() {
    let data = b"$100\r\nTesting\r\n";
    let result = bytes_to_resp_type_ref(data).unwrap_err();
    let token = result.token.unwrap();

    assert_eq!(RespErrorType::InvalidSize, result.error_type);
    assert_eq!(b"$100\r\n", &data[0..token.start]);
    assert_eq!(b"Testing", token.data);
}
