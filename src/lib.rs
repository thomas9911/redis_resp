//! In RESP, the first byte determines the data type:
//! For Simple Strings, the first byte of the reply is "+"
//! For Errors, the first byte of the reply is "-"
//! For Integers, the first byte of the reply is ":"
//! For Bulk Strings, the first byte of the reply is "$"
//! For Arrays, the first byte of the reply is "*"
//!
//!
//!
pub mod error;
pub mod formatter;
pub mod lexer;
pub mod parser;
pub mod resp_type;
pub mod value;

#[cfg(feature = "serde")]
pub mod serde;

pub use error::{OwnedParseError, ParseError, RespErrorType};
pub use lexer::Lexer;
pub use parser::Parser;
pub use resp_type::{RespType, RespTypeRef};
pub use value::Value;

/// Convenience function to parse Redis Resp format into Rust type
/// (maybe look at the serde module to parse specific Rust type).
/// 
/// ```rust
/// use redis_resp::{Value, bytes_to_value};
/// 
/// assert_eq!(Ok(Ok(Value::String("testing".to_string()))), bytes_to_value(b"$7\r\ntesting\r\n"));
/// assert_eq!(Ok(Err(Value::String("testing".to_string()))), bytes_to_value(b"-testing\r\n"));
/// ```
pub fn bytes_to_value(data: &[u8]) -> Result<Result<Value, Value>, ParseError> {
    Ok(bytes_to_resp_type(data)?.into_value())
}

/// Convenience function to parse Redis Resp format
/// 
/// ```rust
/// use redis_resp::{RespType, bytes_to_resp_type};
/// 
/// assert_eq!(Ok(RespType::BulkString(b"testing".to_vec())), bytes_to_resp_type(b"$7\r\ntesting\r\n"));
/// ```
pub fn bytes_to_resp_type(data: &[u8]) -> Result<RespType, ParseError> {
    Ok(Parser::new_from_bytes(data).parse()?.to_owned())
}


/// Convenience function to parse Redis Resp format
/// 
/// ```rust
/// use redis_resp::{RespTypeRef, bytes_to_resp_type_ref};
/// 
/// assert_eq!(Ok(RespTypeRef::BulkString(b"testing")), bytes_to_resp_type_ref(b"$7\r\ntesting\r\n"));
/// ```
pub fn bytes_to_resp_type_ref(data: &[u8]) -> Result<RespTypeRef<'_>, ParseError> {
    Parser::new_from_bytes(data).parse()
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
