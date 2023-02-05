use crate::RespType;

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

impl Into<Result<Value, Value>> for RespType {
    fn into(self) -> Result<Value, Value> {
        self.into_value()
    }
}
