use crate::RespType;

#[derive(Debug, PartialEq, Clone)]
pub enum Value {
    Bytes(Vec<u8>),
    String(String),
    Int(i64),
    Array(Vec<Value>),
    Null,
}

impl From<Value> for RespType {
    fn from(val: Value) -> Self {
        use Value::*;

        match val {
            Bytes(data) => RespType::BulkString(data),
            String(data) => RespType::BulkString(data.into()),
            Int(data) => RespType::Integer(data),
            Array(data) => RespType::Array(data.into_iter().map(|x| x.into()).collect()),
            Null => RespType::NullString,
        }
    }
}

impl From<RespType> for Result<Value, Value> {
    fn from(val: RespType) -> Self {
        val.into_value()
    }
}
