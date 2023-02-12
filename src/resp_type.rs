use crate::Value;

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
            RespTypeRef::Array(x) => RespType::Array(x.iter().map(|y| y.to_owned()).collect()),
            RespTypeRef::NullArray => RespType::NullArray,
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
    pub fn as_referenced(&self) -> RespTypeRef<'_> {
        match self {
            RespType::SimpleString(x) => RespTypeRef::SimpleString(x),
            RespType::Error(x) => RespTypeRef::Error(x),
            RespType::Integer(x) => RespTypeRef::Integer(*x),
            RespType::BulkString(x) => RespTypeRef::BulkString(x),
            RespType::NullString => RespTypeRef::NullString,
            RespType::Array(x) => RespTypeRef::Array(x.iter().map(|y| y.as_referenced()).collect()),
            RespType::NullArray => RespTypeRef::NullArray,
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
