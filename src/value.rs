use crate::resp_type::Attribute;
use crate::{BigInt, HashMap, HashSet, OrderedFloat};
use crate::{Hello, RespType};

/// Rust types that can be expressed in Resp Protocol
#[derive(Debug, PartialEq, Clone, Hash, Eq)]
pub enum Value {
    Bytes(Vec<u8>),
    String(String),
    Bool(bool),
    Int(i64),
    Double(OrderedFloat<f64>),
    BigInt(BigInt),
    Array(Vec<Value>),
    Map(HashMap<Value, Value>),
    Set(HashSet<Value>),
    AttributedValue(Vec<Value>, Box<Value>),
    Hello(Hello),
    Null,
}

impl From<Value> for RespType {
    fn from(val: Value) -> Self {
        use Value::*;

        match val {
            Bytes(data) => RespType::BulkString(data),
            String(data) => RespType::BulkString(data.into()),
            Bool(data) => RespType::Boolean(data),
            Double(data) => RespType::Double(data),
            BigInt(data) => RespType::BigInteger(data),
            Int(data) => RespType::Integer(data),
            Array(data) => RespType::Array(convert(data)),
            Map(data) => RespType::Map(
                data.into_iter()
                    .map(|(k, v)| (k.into(), v.into()))
                    .collect(),
            ),
            Set(data) => RespType::Set(data.into_iter().map(RespType::from).collect()),
            AttributedValue(attrs, data) => RespType::Attribute(Attribute {
                attributes: convert(attrs),
                data: Box::new(RespType::from(*data)),
            }),
            Hello(data) => RespType::Hello(data),
            Null => RespType::NullString,
        }
    }
}

fn convert(data: Vec<Value>) -> Vec<RespType> {
    data.into_iter().map(RespType::from).collect()
}

impl From<RespType> for Result<Value, Value> {
    fn from(val: RespType) -> Self {
        val.into_value()
    }
}
