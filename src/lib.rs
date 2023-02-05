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

pub use lexer::Lexer;
pub use parser::Parser;

#[derive(Debug, PartialEq, Clone)]
pub enum Value {
    Bytes(Vec<u8>),
    String(String),
    Int(i64),
    Array(Vec<Value>),
    Null,
}

impl Into<RespType> for Value {
    fn into(self) -> RespType {
        use Value::*;

        match self {
            Bytes(data) => RespType::BulkString(data),
            String(data) => RespType::BulkString(data.into()),
            Int(data) => RespType::Integer(data),
            Array(data) => RespType::Array(data.into_iter().map(|x| x.into()).collect()),
            Null => RespType::NullString,
        }
    }
}

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

#[derive(Debug, PartialEq)]
pub enum RespTypeRef<'a> {
    SimpleString(&'a [u8]),
    Error(&'a [u8]),
    Integer(i64),
    BulkString(&'a [u8]),
    NullString,
    Array(Vec<RespTypeRef<'a>>),
    NullArray,
}

impl<'a> RespTypeRef<'a> {
    pub fn to_owned(&'a self) -> RespType {
        match self {
            RespTypeRef::SimpleString(x) => RespType::SimpleString(x.to_vec()),
            RespTypeRef::Error(x) => RespType::Error(x.to_vec()),
            RespTypeRef::Integer(x) => RespType::Integer(*x),
            RespTypeRef::BulkString(x) => RespType::BulkString(x.to_vec()),
            RespTypeRef::NullString => RespType::NullString,
            RespTypeRef::Array(x) => RespType::Array(x.into_iter().map(|y| y.to_owned()).collect()),
            RespTypeRef::NullArray => RespType::NullArray,
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum RespType {
    SimpleString(Vec<u8>),
    Error(Vec<u8>),
    Integer(i64),
    BulkString(Vec<u8>),
    NullString,
    Array(Vec<RespType>),
    NullArray,
}

impl RespType {
    pub fn is_null(&self) -> bool {
        use RespType::*;

        match self {
            NullString => true,
            NullArray => true,
            _ => false,
        }
    }

    pub fn as_bytes(&self) -> Option<&[u8]> {
        use RespType::*;

        match self {
            SimpleString(data) => Some(data),
            BulkString(data) => Some(data),
            _ => None,
        }
    }

    pub fn as_error_bytes(&self) -> Option<&[u8]> {
        if let RespType::Error(error) = self {
            Some(error)
        } else {
            None
        }
    }

    pub fn as_string(&self) -> Option<&str> {
        self.as_bytes()
            .map(|x| std::str::from_utf8(x).ok())
            .flatten()
    }

    pub fn as_error_string(&self) -> Option<&str> {
        self.as_error_bytes()
            .map(|x| std::str::from_utf8(x).ok())
            .flatten()
    }

    pub fn into_bytes(self) -> Option<Vec<u8>> {
        use RespType::*;

        match self {
            SimpleString(data) => Some(data),
            BulkString(data) => Some(data),
            _ => None,
        }
    }

    pub fn into_error_bytes(self) -> Option<Vec<u8>> {
        if let RespType::Error(error) = self {
            Some(error)
        } else {
            None
        }
    }

    pub fn into_string(self) -> Option<String> {
        self.into_bytes()
            .map(|x| String::from_utf8(x).ok())
            .flatten()
    }

    pub fn into_error_string(self) -> Option<String> {
        self.into_error_bytes()
            .map(|x| String::from_utf8(x).ok())
            .flatten()
    }
}

impl Into<Result<Value, Value>> for RespType {
    fn into(self) -> Result<Value, Value> {
        use RespType::*;

        if self.is_null() {
            return Ok(Value::Null);
        }

        if self.as_error_string().is_some() {
            return Err(Value::String(self.into_error_string().unwrap()));
        };

        if self.as_error_bytes().is_some() {
            return Err(Value::Bytes(self.into_error_bytes().unwrap()));
        };

        if self.as_string().is_some() {
            return Ok(Value::String(self.into_string().unwrap()));
        };

        if self.as_bytes().is_some() {
            return Ok(Value::Bytes(self.into_bytes().unwrap()));
        };

        match self {
            Integer(data) => Ok(Value::Int(data)),
            Array(data) => {
                let converted: Result<Vec<Value>, Value> =
                    data.into_iter().map(|x| x.into()).collect();
                Ok(Value::Array(converted?))
            }
            _ => unreachable!(),
        }
    }
}

#[test]
fn resp_type_into_value_error_string() {
    let result: Result<Value, Value> = RespType::Error(b"error".to_vec()).into();

    assert_eq!(result, Err(Value::String("error".to_string())))
}

#[test]
fn resp_type_into_value_error_bytes() {
    let result: Result<Value, Value> = RespType::Error(b"\xfe\xfe\xff\xff".to_vec()).into();
    assert_eq!(result, Err(Value::Bytes([254, 254, 255, 255].to_vec())))
}

#[test]
fn resp_type_into_value_bulkstring_string() {
    let result: Result<Value, Value> = RespType::BulkString(b"text".to_vec()).into();

    assert_eq!(result, Ok(Value::String("text".to_string())))
}

#[test]
fn resp_type_into_value_bulkstring_bytes() {
    let result: Result<Value, Value> =
        RespType::BulkString([254, 254, 255, 255, 1, 2, 3, 4].to_vec()).into();

    assert_eq!(
        result,
        Ok(Value::Bytes([254, 254, 255, 255, 1, 2, 3, 4].to_vec()))
    )
}

#[test]
fn resp_type_into_value_null() {
    let result: Result<Value, Value> = RespType::NullString.into();

    assert_eq!(result, Ok(Value::Null));

    let result: Result<Value, Value> = RespType::NullArray.into();

    assert_eq!(result, Ok(Value::Null));
}

#[test]
fn resp_type_into_value_array() {
    let result: Result<Value, Value> = RespType::Array(vec![
        RespType::SimpleString(b"OK".to_vec()),
        RespType::BulkString(b"just some text".to_vec()),
        RespType::NullString,
        RespType::Array(vec![]),
    ])
    .into();

    assert_eq!(
        result,
        Ok(Value::Array(vec![
            Value::String("OK".to_string()),
            Value::String("just some text".to_string()),
            Value::Null,
            Value::Array(vec![])
        ]))
    )
}

#[test]
fn convert_text_to_value() {
    let mut parser = Parser::new_from_bytes(b"$14\r\njust some text\r\n");
    let item = parser.parse().unwrap().to_owned();
    let result: Result<Value, Value> = item.into();

    assert_eq!(result, Ok(Value::String("just some text".to_string())))
}
