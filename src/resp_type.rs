use crate::Hello;
use crate::Value;
use crate::{BigInt, HashMap, HashSet, OrderedFloat};
use std::borrow::Cow;

#[derive(Debug, PartialEq, Hash, Eq, Clone)]
pub struct AttributeRef<'a> {
    pub attributes: Vec<RespTypeRef<'a>>,
    pub data: Box<RespTypeRef<'a>>,
}

impl Attribute {
    pub fn as_referenced(&'_ self) -> AttributeRef<'_> {
        AttributeRef {
            attributes: self.attributes.iter().map(|y| y.as_referenced()).collect(),
            data: Box::new(self.data.as_referenced()),
        }
    }
}

impl<'a> AttributeRef<'a> {
    /// claims the reference as the actual object, sort of like to_owned but returns a different type
    fn claim(&self) -> Attribute {
        Attribute {
            attributes: self.attributes.iter().map(|y| y.claim()).collect(),
            data: Box::new(self.data.claim()),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Hash, Eq)]
pub enum RespTypeRef<'a> {
    SimpleString(&'a [u8]),
    Error(&'a [u8]),
    Integer(i64),
    BulkString(&'a [u8]),
    NullString,
    Array(Vec<RespTypeRef<'a>>),
    NullArray,
    Null,
    Double(OrderedFloat<f64>),
    Boolean(bool),
    BlobError(&'a [u8]),
    VerbatimString(&'a [u8], &'a [u8]),
    Map(Vec<(RespTypeRef<'a>, RespTypeRef<'a>)>),
    Set(Vec<RespTypeRef<'a>>),
    Attribute(AttributeRef<'a>),
    Push(Vec<RespTypeRef<'a>>),
    Hello(Hello),
    BigInteger(Cow<'a, [u8]>),
}

impl<'a> RespTypeRef<'a> {
    pub fn claim(&'a self) -> RespType {
        match self {
            RespTypeRef::SimpleString(x) => RespType::SimpleString(x.to_vec()),
            RespTypeRef::Error(x) => RespType::Error(x.to_vec()),
            RespTypeRef::Integer(x) => RespType::Integer(*x),
            RespTypeRef::BulkString(x) => RespType::BulkString(x.to_vec()),
            RespTypeRef::NullString => RespType::NullString,
            RespTypeRef::Array(x) => RespType::Array(x.iter().map(|y| y.claim()).collect()),
            RespTypeRef::NullArray => RespType::NullArray,
            RespTypeRef::Null => RespType::Null,
            RespTypeRef::Double(x) => RespType::Double(*x),
            RespTypeRef::Boolean(x) => RespType::Boolean(*x),
            RespTypeRef::BlobError(x) => RespType::BlobError(x.to_vec()),
            RespTypeRef::VerbatimString(x, y) => RespType::VerbatimString(x.to_vec(), y.to_vec()),
            RespTypeRef::Map(x) => {
                RespType::Map(x.iter().map(|(k, v)| (k.claim(), v.claim())).collect())
            }
            RespTypeRef::Set(x) => RespType::Set(x.iter().map(|y| y.claim()).collect()),
            RespTypeRef::Attribute(x) => RespType::Attribute(x.claim()),
            RespTypeRef::Push(x) => RespType::Push(x.iter().map(|y| y.claim()).collect()),
            RespTypeRef::Hello(x) => RespType::Hello(x.clone()),
            RespTypeRef::BigInteger(x) => RespType::BigInteger(
                std::str::from_utf8(x)
                    .expect("invalid bytes")
                    .parse()
                    .expect("invalid bytes"),
            ),
        }
    }

    pub fn as_type(&'a self) -> RespTypeRefType {
        match self {
            RespTypeRef::SimpleString(_) => RespTypeRefType::SimpleString,
            RespTypeRef::Error(_) => RespTypeRefType::Error,
            RespTypeRef::Integer(_) => RespTypeRefType::Integer,
            RespTypeRef::BulkString(_) => RespTypeRefType::BulkString,
            RespTypeRef::NullString => RespTypeRefType::NullString,
            RespTypeRef::Array(_) => RespTypeRefType::Array,
            RespTypeRef::NullArray => RespTypeRefType::NullArray,
            RespTypeRef::Null => RespTypeRefType::Null,
            RespTypeRef::Double(_) => RespTypeRefType::Double,
            RespTypeRef::Boolean(_) => RespTypeRefType::Boolean,
            RespTypeRef::BlobError(_) => RespTypeRefType::BlobError,
            RespTypeRef::VerbatimString(_, _) => RespTypeRefType::VerbatimString,
            RespTypeRef::Map(_) => RespTypeRefType::Map,
            RespTypeRef::Set(_) => RespTypeRefType::Set,
            RespTypeRef::Attribute(_) => RespTypeRefType::Attribute,
            RespTypeRef::Push(_) => RespTypeRefType::Push,
            RespTypeRef::Hello(_) => RespTypeRefType::Hello,
            RespTypeRef::BigInteger(_) => RespTypeRefType::BigInteger,
        }
    }

    pub fn is_null(&self) -> bool {
        use RespTypeRef::*;

        matches!(self, NullString | NullArray)
    }

    pub fn as_bytes(&self) -> Option<&[u8]> {
        use RespTypeRef::*;

        match self {
            SimpleString(data) => Some(data),
            BulkString(data) => Some(data),
            _ => None,
        }
    }

    pub fn as_error_bytes(&self) -> Option<&[u8]> {
        if let RespTypeRef::Error(error) = self {
            Some(error)
        } else {
            None
        }
    }

    pub fn as_string(&self) -> Option<&str> {
        self.as_bytes().and_then(|x| std::str::from_utf8(x).ok())
    }

    pub fn as_error_string(&self) -> Option<&str> {
        self.as_error_bytes()
            .and_then(|x| std::str::from_utf8(x).ok())
    }
}

#[derive(Debug, PartialEq, Hash, Eq, Clone)]
pub struct Attribute {
    pub attributes: Vec<RespType>,
    pub data: Box<RespType>,
}

#[derive(Debug, PartialEq, Hash, Eq, Clone)]
pub enum RespType {
    SimpleString(Vec<u8>),
    Error(Vec<u8>),
    Integer(i64),
    BulkString(Vec<u8>),
    NullString,
    Array(Vec<RespType>),
    NullArray,
    Null,
    Double(OrderedFloat<f64>),
    Boolean(bool),
    BlobError(Vec<u8>),
    VerbatimString(Vec<u8>, Vec<u8>),
    Map(HashMap<RespType, RespType>),
    Set(HashSet<RespType>),
    Attribute(Attribute),
    Push(Vec<RespType>),
    Hello(Hello),
    BigInteger(BigInt),
}

impl RespType {
    pub fn as_referenced(&self) -> RespTypeRef<'_> {
        match self {
            RespType::SimpleString(x) => RespTypeRef::SimpleString(x),
            RespType::Error(x) => RespTypeRef::Error(x),
            RespType::Integer(x) => RespTypeRef::Integer(*x),
            RespType::BulkString(x) => RespTypeRef::BulkString(x),
            RespType::NullString => RespTypeRef::NullString,
            RespType::Array(x) => RespTypeRef::Array(x.iter().map(|y| y.as_referenced()).collect()),
            RespType::NullArray => RespTypeRef::NullArray,
            RespType::Null => RespTypeRef::Null,
            RespType::Double(x) => RespTypeRef::Double(*x),
            RespType::Boolean(x) => RespTypeRef::Boolean(*x),
            RespType::BlobError(x) => RespTypeRef::BlobError(x),
            RespType::VerbatimString(x, y) => RespTypeRef::VerbatimString(x, y),
            RespType::Map(x) => RespTypeRef::Map(
                x.iter()
                    .map(|(k, v)| (k.as_referenced(), v.as_referenced()))
                    .collect(),
            ),
            RespType::Set(x) => RespTypeRef::Set(x.iter().map(|y| y.as_referenced()).collect()),
            RespType::Attribute(x) => RespTypeRef::Attribute(x.as_referenced()),
            RespType::Push(x) => RespTypeRef::Push(x.iter().map(|y| y.as_referenced()).collect()),
            RespType::Hello(x) => RespTypeRef::Hello(x.clone()),
            RespType::BigInteger(x) => {
                RespTypeRef::BigInteger(Cow::from(x.to_str_radix(10).as_bytes().to_vec()))
            }
        }
    }

    pub fn is_null(&self) -> bool {
        use RespType::*;

        matches!(self, NullString | NullArray)
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
        self.as_bytes().and_then(|x| std::str::from_utf8(x).ok())
    }

    pub fn as_error_string(&self) -> Option<&str> {
        self.as_error_bytes()
            .and_then(|x| std::str::from_utf8(x).ok())
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
        self.into_bytes().and_then(|x| String::from_utf8(x).ok())
    }

    pub fn into_error_string(self) -> Option<String> {
        self.into_error_bytes()
            .and_then(|x| String::from_utf8(x).ok())
    }

    pub fn into_value(self) -> Result<Value, Value> {
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

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum RespTypeRefType {
    SimpleString,
    Error,
    Integer,
    BulkString,
    NullString,
    Array,
    NullArray,
    Null,
    Double,
    Boolean,
    BlobError,
    VerbatimString,
    Map,
    Set,
    Attribute,
    Push,
    Hello,
    BigInteger,
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
