use std::io::Write;

use crate::consts;
use crate::{FormatError, RespTypeRef};

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Protocol {
    V2,
    V3,
}

#[derive(Debug, PartialEq)]
pub struct Formatter<'a> {
    pub item: RespTypeRef<'a>,
    pub allow_nested: bool,
    pub protocol: Protocol,
}

impl<'a> Formatter<'a> {
    pub fn new_with_defaults(item: RespTypeRef<'a>) -> Formatter<'a> {
        Self::new_protocol_v3(item)
    }

    pub fn new_protocol_v3(item: RespTypeRef<'a>) -> Formatter<'a> {
        Formatter {
            item,
            allow_nested: true,
            protocol: Protocol::V3,
        }
    }

    pub fn new_protocol_v2(item: RespTypeRef<'a>) -> Formatter<'a> {
        Formatter {
            item,
            allow_nested: false,
            protocol: Protocol::V2,
        }
    }

    pub fn write<W: Write>(&self, output: &mut W) -> Result<(), FormatError> {
        self.inner_write(output, &self.item, 0)
    }

    pub fn set_protocol_v3(&mut self) -> &mut Formatter<'a> {
        self.protocol = Protocol::V3;
        self.allow_nested = true;
        self
    }

    pub fn set_protocol_v2(&mut self) -> &mut Formatter<'a> {
        self.protocol = Protocol::V2;
        self.allow_nested = false;
        self
    }

    pub fn is_protocol_v3(&self) -> bool {
        self.protocol == Protocol::V3
    }

    fn inner_write<W: Write>(
        &self,
        output: &mut W,
        item: &RespTypeRef<'a>,
        level: usize,
    ) -> Result<(), FormatError> {
        if !self.allow_nested && level >= 2 {
            return Err(FormatError::NestedDataNotAllowed);
        }

        use RespTypeRef::*;

        match item {
            SimpleString(data) => {
                output.write_all(&[consts::SIMPLE_STRING])?;
                output.write_all(data)?;
                output.write_all(&consts::NEWLINE)?;
                return Ok(());
            }
            Error(data) => {
                output.write_all(&[consts::ERROR])?;
                output.write_all(data)?;
                output.write_all(&consts::NEWLINE)?;
                return Ok(());
            }
            Integer(data) => {
                output.write_all(&[consts::INTEGER])?;
                output.write_all(data.to_string().as_bytes())?;
                output.write_all(&consts::NEWLINE)?;
                return Ok(());
            }
            BulkString(data) => {
                output.write_all(&[consts::BULK_STRING])?;
                output.write_all(data.len().to_string().as_bytes())?;
                output.write_all(&consts::NEWLINE)?;
                output.write_all(data)?;
                output.write_all(&consts::NEWLINE)?;
                return Ok(());
            }
            NullString => {
                output.write_all(b"$-1\r\n")?;
                return Ok(());
            }
            Array(data) => {
                output.write_all(&[consts::ARRAY])?;
                output.write_all(data.len().to_string().as_bytes())?;
                output.write_all(&consts::NEWLINE)?;
                for array_item in data {
                    self.inner_write(output, array_item, level + 1)?;
                }
                return Ok(());
            }
            NullArray => {
                output.write_all(b"*-1\r\n")?;
                return Ok(());
            }
            _ => (),
        };

        if self.is_protocol_v3() {
            match item {
                Null => {
                    output.write_all(&[consts::NULL])?;
                    output.write_all(&consts::NEWLINE)?;
                }
                Boolean(true) => {
                    output.write_all(&consts::TRUE)?;
                    output.write_all(&consts::NEWLINE)?;
                }
                Boolean(false) => {
                    output.write_all(&consts::FALSE)?;
                    output.write_all(&consts::NEWLINE)?;
                }
                Double(f) if f.is_nan() => {
                    output.write_all(&consts::NAN)?;
                    output.write_all(&consts::NEWLINE)?;
                }
                Double(f) if f.is_infinite() && f.is_sign_positive() => {
                    output.write_all(&consts::INFINITE)?;
                    output.write_all(&consts::NEWLINE)?;
                }
                Double(f) if f.is_infinite() && f.is_sign_negative() => {
                    output.write_all(&consts::NEG_INFINITE)?;
                    output.write_all(&consts::NEWLINE)?;
                }
                Double(f) => {
                    output.write_all(&[consts::DOUBLE])?;
                    output.write_all(f.to_string().as_bytes())?;
                    output.write_all(&consts::NEWLINE)?;
                }
                BlobError(data) => {
                    output.write_all(&[consts::BULK_ERROR])?;
                    output.write_all(data.len().to_string().as_bytes())?;
                    output.write_all(&consts::NEWLINE)?;
                    output.write_all(data)?;
                    output.write_all(&consts::NEWLINE)?;
                }
                VerbatimString(prefix, data) => {
                    output.write_all(&[consts::VERBATIM_STRING])?;
                    output.write_all((data.len() + 4).to_string().as_bytes())?;
                    output.write_all(&consts::NEWLINE)?;
                    output.write_all(prefix)?;
                    output.write_all(&[consts::VERBATIM_STRING_SEPARATOR])?;
                    output.write_all(data)?;
                    output.write_all(&consts::NEWLINE)?;
                }
                BigInteger(data) => {
                    output.write_all(&[consts::BIG_INTEGER])?;
                    output.write_all(data)?;
                    output.write_all(&consts::NEWLINE)?;
                }
                Map(data) => {
                    output.write_all(&[consts::MAP])?;
                    output.write_all(data.len().to_string().as_bytes())?;
                    output.write_all(&consts::NEWLINE)?;
                    for (key, value) in data {
                        self.inner_write(output, key, level + 1)?;
                        self.inner_write(output, value, level + 1)?;
                    }
                }
                Set(data) => {
                    output.write_all(&[consts::SET])?;
                    output.write_all(data.len().to_string().as_bytes())?;
                    output.write_all(&consts::NEWLINE)?;
                    for item in data {
                        self.inner_write(output, item, level + 1)?;
                    }
                }
                Attribute(data) => {
                    output.write_all(&[consts::ATTRIBUTE])?;
                    output.write_all(data.attributes.len().to_string().as_bytes())?;
                    output.write_all(&consts::NEWLINE)?;
                    for item in data.attributes.iter() {
                        self.inner_write(output, item, level + 1)?;
                    }
                    self.inner_write(output, &data.data, level + 1)?;
                }
                Push(data) => {
                    output.write_all(&[consts::PUSH])?;
                    output.write_all(data.len().to_string().as_bytes())?;
                    output.write_all(&consts::NEWLINE)?;
                    for array_item in data {
                        self.inner_write(output, array_item, level + 1)?;
                    }
                }
                Hello(data) => {
                    output.write_all(&consts::HELLO)?;
                    output.write_all(b" ")?;
                    output.write_all(data.protocol.as_bytes())?;
                    if let Some(auth) = &data.auth {
                        output.write_all(b" ")?;
                        output.write_all(&consts::AUTH)?;
                        output.write_all(b" ")?;
                        output.write_all(auth.username.as_bytes())?;
                        output.write_all(b" ")?;
                        output.write_all(auth.password.as_bytes())?;
                    }
                }
                _ => unreachable!(),
            };
            Ok(())
        } else {
            Err(FormatError::ProtocolError)
        }
    }
}

#[test]
fn formatter_simple_string() {
    let formatter = Formatter::new_with_defaults(RespTypeRef::SimpleString(b"just text"));
    let expected = b"+just text\r\n";

    let mut buffer = Vec::new();

    formatter.write(&mut buffer).unwrap();

    assert_eq!(buffer, expected);
}

#[test]
fn formatter_error() {
    let formatter = Formatter::new_with_defaults(RespTypeRef::Error(b"CRASH"));
    let expected = b"-CRASH\r\n";

    let mut buffer = Vec::new();

    formatter.write(&mut buffer).unwrap();

    assert_eq!(buffer, expected);
}

#[test]
fn formatter_integer() {
    let formatter = Formatter::new_with_defaults(RespTypeRef::Integer(12345));
    let expected = b":12345\r\n";

    let mut buffer = Vec::new();

    formatter.write(&mut buffer).unwrap();

    assert_eq!(buffer, expected);
}

#[test]
fn formatter_bulk_string() {
    let formatter = Formatter::new_with_defaults(RespTypeRef::BulkString(b"Just some text"));
    let expected = b"$14\r\nJust some text\r\n";

    let mut buffer = Vec::new();

    formatter.write(&mut buffer).unwrap();

    assert_eq!(buffer, expected);
}

#[test]
fn formatter_integer_array() {
    let formatter = Formatter::new_with_defaults(RespTypeRef::Array(vec![
        RespTypeRef::Integer(1),
        RespTypeRef::Integer(2),
        RespTypeRef::Integer(3),
    ]));
    let expected = b"*3\r\n:1\r\n:2\r\n:3\r\n";

    let mut buffer = Vec::new();

    formatter.write(&mut buffer).unwrap();

    assert_eq!(buffer, expected);
}

#[test]
fn formatter_mixed_array() {
    let formatter = Formatter::new_with_defaults(RespTypeRef::Array(vec![
        RespTypeRef::Integer(1),
        RespTypeRef::SimpleString(b"OK"),
        RespTypeRef::NullArray,
        RespTypeRef::BulkString(b"Just text"),
    ]));
    let expected = b"*4\r\n:1\r\n+OK\r\n*-1\r\n$9\r\nJust text\r\n";
    let mut buffer = Vec::new();

    formatter.write(&mut buffer).unwrap();

    assert_eq!(buffer, expected);
}

#[cfg(test)]
mod proptests {
    use proptest::prelude::*;

    use crate::formatter::Formatter;
    use crate::resp_type::Attribute;
    use crate::RespType;
    use ordered_float::OrderedFloat;

    fn arb_resp_type() -> impl Strategy<Value = RespType> {
        let leaf = prop_oneof![
            Just(RespType::Null),
            Just(RespType::NullString),
            Just(RespType::NullArray),
            prop::bool::ANY.prop_map(RespType::Boolean),
            prop::num::f64::ANY.prop_map(|x| RespType::Double(OrderedFloat(x))),
            prop::string::bytes_regex(".*")
                .unwrap()
                .prop_map(RespType::SimpleString),
            prop::string::bytes_regex(".*")
                .unwrap()
                .prop_map(RespType::Error),
            prop::string::bytes_regex(".*")
                .unwrap()
                .prop_map(RespType::BulkString),
            prop::string::bytes_regex(".*")
                .unwrap()
                .prop_map(RespType::BlobError),
            prop::string::bytes_regex(".*")
                .unwrap()
                .prop_map(|x| RespType::VerbatimString(b"txt".to_vec(), x)),
            prop::string::string_regex("-?[0-9]+")
                .unwrap()
                .prop_map(|x| RespType::BigInteger(x.parse().unwrap())),
        ];
        leaf.prop_recursive(8, 256, 10, |inner| {
            prop_oneof![
                prop::collection::vec(inner.clone(), 0..10).prop_map(|x| RespType::Attribute(
                    Attribute {
                        attributes: x,
                        data: Box::new(RespType::Null)
                    }
                )),
                prop::collection::vec(inner.clone(), 0..10).prop_map(RespType::Array),
                prop::collection::vec(inner.clone(), 0..10).prop_map(RespType::Push),
                im::proptest::hash_map(inner.clone(), inner.clone(), 0..10).prop_map(RespType::Map),
                im::proptest::hash_set(inner, 0..10).prop_map(RespType::Set),
            ]
        })
    }

    proptest! {
        #[test]
        fn xd(x in arb_resp_type()) {
            let formatter = Formatter::new_protocol_v3(x.as_referenced());

            let mut buffer = Vec::new();
            prop_assert!(formatter.write(&mut buffer).is_ok());
            prop_assert!(!buffer.is_empty())
        }
    }
}
