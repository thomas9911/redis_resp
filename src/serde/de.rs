use crate::{Parser, RespTypeRef};

use crate::serde::{DeserializerError, DeserializerResult};
use serde::de::{self, Deserialize, DeserializeSeed, SeqAccess, Visitor};

pub fn from_bytes<'de: 'a, 'a, T>(input: &'de [u8]) -> Result<T, String>
where
    T: Deserialize<'de>,
{
    let mut deserializer = Deserializer::from_bytes(input);
    // let t = T::deserialize(&mut deserializer)?;
    // Ok(t)

    T::deserialize(&mut deserializer).map_err(|e| e.to_string())
}

pub struct Deserializer<'de> {
    input: Parser<'de>,
    item: Option<RespTypeRef<'de>>,
}

impl<'de> Deserializer<'de> {
    pub fn from_bytes(input: &'de [u8]) -> Self {
        Deserializer {
            input: Parser::new_from_bytes(input),
            item: None,
        }
    }

    fn set_item(&mut self) -> DeserializerResult<'de, ()> {
        if self.item.is_none() {
            let item = self.input.parse()?;
            self.item = Some(item);
        }

        Ok(())
    }
}

impl<'de, 'a> de::Deserializer<'de> for &'a mut Deserializer<'de> {
    type Error = DeserializerError<'a>;

    fn deserialize_any<V>(self, visitor: V) -> DeserializerResult<'a, V::Value>
    where
        V: Visitor<'de>,
    {

        self.set_item()?;

        match self.item {
            Some(RespTypeRef::Array(_)) => self.deserialize_seq(visitor),
            Some(RespTypeRef::SimpleString(_)) => self.deserialize_byte_buf(visitor),
            Some(RespTypeRef::BulkString(_)) => self.deserialize_byte_buf(visitor),
            Some(RespTypeRef::Error(_)) => self.deserialize_byte_buf(visitor),
            Some(RespTypeRef::NullArray) => self.deserialize_unit(visitor),
            Some(RespTypeRef::NullString) => self.deserialize_unit(visitor),
            Some(RespTypeRef::Integer(_)) => self.deserialize_i64(visitor),
            None => Err(Self::Error::message("Invalid".to_string())),
        }
    }

    fn deserialize_bool<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        todo!()
    }

    fn deserialize_i8<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.deserialize_i64(visitor)
    }

    fn deserialize_i16<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.deserialize_i64(visitor)
    }

    fn deserialize_i32<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.deserialize_i64(visitor)
    }

    fn deserialize_i64<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        match &self.item {
            Some(RespTypeRef::Integer(data)) => visitor.visit_i64(*data),
            None => self.deserialize_any(visitor),
            e => {
                dbg!(&e);
                Err(Self::Error::message("invalid input".to_string()))
            }
        }
    }

    fn deserialize_u8<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.deserialize_i64(visitor)
    }

    fn deserialize_u16<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.deserialize_i64(visitor)
    }

    fn deserialize_u32<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.deserialize_i64(visitor)
    }

    fn deserialize_u64<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.deserialize_i64(visitor)
    }

    fn deserialize_f32<V>(self, _visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        Err(Self::Error::message("Not supported type".to_string()))
    }

    fn deserialize_f64<V>(self, _visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        Err(Self::Error::message("Not supported type".to_string()))
    }

    fn deserialize_char<V>(self, _visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        Err(Self::Error::message("Not supported type".to_string()))
    }

    fn deserialize_str<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.deserialize_string(visitor)
    }

    fn deserialize_string<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        match &self.item {
            Some(RespTypeRef::BulkString(data)) => visitor.visit_bytes(data),
            Some(RespTypeRef::SimpleString(data)) => visitor.visit_bytes(data),
            Some(RespTypeRef::Error(data)) => visitor.visit_bytes(data),
            None => self.deserialize_any(visitor),
            e => {
                dbg!(&e);
                Err(Self::Error::message("invalid input".to_string()))
            }
        }
    }

    fn deserialize_bytes<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.deserialize_string(visitor)
    }

    fn deserialize_byte_buf<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.deserialize_string(visitor)
    }

    fn deserialize_option<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        match &self.item {
            Some(RespTypeRef::NullArray) => visitor.visit_none(),
            Some(RespTypeRef::NullString) => visitor.visit_none(),
            None => {
                self.set_item()?;
                self.deserialize_option(visitor)
            },
            Some(_) => {
                visitor.visit_some(self)
            }
        }
    }

    fn deserialize_unit<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        match &self.item {
            Some(RespTypeRef::NullArray) => visitor.visit_unit(),
            Some(RespTypeRef::NullString) => visitor.visit_unit(),
            None => self.deserialize_any(visitor),
            e => {
                dbg!(&e);
                Err(Self::Error::message("invalid input".to_string()))
            }
        }
    }

    fn deserialize_unit_struct<V>(
        self,
        _name: &'static str,
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.deserialize_unit(visitor)
    }

    fn deserialize_newtype_struct<V>(
        self,
        _name: &'static str,
        _visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        Err(Self::Error::message("Not supported type".to_string()))
    }

    fn deserialize_seq<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        match self.item {
            Some(RespTypeRef::BulkString(data)) => visitor.visit_bytes(data),
            None => {
                self.set_item()?;
                self.deserialize_seq(visitor)
            },
            _ => Err(Self::Error::message("invalid input".to_string())),
        }
    }

    fn deserialize_tuple<V>(self, _len: usize, _visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        Err(Self::Error::message("Not supported type".to_string()))
    }

    fn deserialize_tuple_struct<V>(
        self,
        _name: &'static str,
        _len: usize,
        _visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        Err(Self::Error::message("Not supported type".to_string()))
    }

    fn deserialize_map<V>(self, _visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        Err(Self::Error::message("Not supported type".to_string()))
    }

    fn deserialize_struct<V>(
        self,
        _name: &'static str,
        _fields: &'static [&'static str],
        _visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        Err(Self::Error::message("Not supported type".to_string()))
    }

    fn deserialize_enum<V>(
        self,
        _name: &'static str,
        _variants: &'static [&'static str],
        _visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        Err(Self::Error::message("Not supported type".to_string()))
    }

    fn deserialize_identifier<V>(self, _visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        Err(Self::Error::message("Not supported type".to_string()))
    }

    fn deserialize_ignored_any<V>(self, _visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        Err(Self::Error::message("Not supported type".to_string()))
    }
}

#[test]
fn deserialize_string_test() {
    let out: String = from_bytes(b"$14\r\njust some text\r\n").unwrap();
    assert_eq!(out, "just some text");
}

#[test]
fn deserialize_bytes_test() {
    let out: serde_bytes::ByteBuf = from_bytes(b"$4\r\n\xfe\xfe\xff\xff\r\n").unwrap();
    assert_eq!(&out, b"\xfe\xfe\xff\xff");
}

#[test]
fn deserialize_i64_test() {
    assert_eq!(1, from_bytes::<i64>(b":1\r\n").unwrap());
    assert_eq!(-1, from_bytes::<i64>(b":-1\r\n").unwrap());
    assert_eq!(12345, from_bytes::<i64>(b":12345\r\n").unwrap());
    assert!(from_bytes::<u64>(format!(":{}\r\n", i128::MAX).as_bytes()).is_err());
}

#[test]
fn deserialize_u64_test() {
    assert_eq!(1, from_bytes::<u64>(b":1\r\n").unwrap());
    assert!(from_bytes::<u64>(b":-1\r\n").is_err());
    assert_eq!(12345, from_bytes::<u64>(b":12345\r\n").unwrap());
}


#[test]
fn deserialize_unit_test() {
    assert_eq!((), from_bytes::<()>(b"*-1\r\n").unwrap());
    assert_eq!((), from_bytes::<()>(b"$-1\r\n").unwrap());
    assert!(from_bytes::<()>(b":-1\r\n").is_err());
}

#[test]
fn deserialize_option_test() {
    assert_eq!(None, from_bytes::<Option<String>>(b"$-1\r\n").unwrap());
    assert_eq!(Some("t".to_string()), from_bytes::<Option<String>>(b"$1\r\nt\r\n").unwrap());
}
