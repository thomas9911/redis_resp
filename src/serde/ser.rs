use std::io::Write;

use serde::ser::{Error, Impossible};
use serde::{ser, Serialize};

use crate::formatter::Formatter;
use crate::serde::error::{SerializerError, SerializerResult};
use crate::RespTypeRef;

pub struct Serializer {
    output: Vec<u8>,
}

pub fn to_bytes<T>(value: &T) -> SerializerResult<Vec<u8>>
where
    T: Serialize,
{
    let mut serializer = Serializer { output: Vec::new() };
    value.serialize(&mut serializer)?;
    Ok(serializer.output)
}

impl<'a> ser::Serializer for &'a mut Serializer {
    type Ok = ();

    type Error = SerializerError;

    type SerializeSeq = Self;
    type SerializeTuple = Self;
    type SerializeTupleStruct = Self;
    type SerializeTupleVariant = Self;
    type SerializeMap = Impossible<(), SerializerError>;
    type SerializeStruct = Impossible<(), SerializerError>;
    type SerializeStructVariant = Impossible<(), SerializerError>;

    fn serialize_bool(self, v: bool) -> Result<Self::Ok, Self::Error> {
        let int = if v { 1 } else { 0 };

        self.serialize_i64(int)
    }

    fn serialize_i8(self, v: i8) -> Result<Self::Ok, Self::Error> {
        self.serialize_i64(v as i64)
    }

    fn serialize_i16(self, v: i16) -> Result<Self::Ok, Self::Error> {
        self.serialize_i64(v as i64)
    }

    fn serialize_i32(self, v: i32) -> Result<Self::Ok, Self::Error> {
        self.serialize_i64(v as i64)
    }

    fn serialize_i64(self, v: i64) -> Result<Self::Ok, Self::Error> {
        Formatter::new_with_defaults(RespTypeRef::Integer(v)).write(&mut self.output)?;
        Ok(())
    }

    fn serialize_u8(self, v: u8) -> Result<Self::Ok, Self::Error> {
        self.serialize_i64(v as i64)
    }

    fn serialize_u16(self, v: u16) -> Result<Self::Ok, Self::Error> {
        self.serialize_i64(v as i64)
    }

    fn serialize_u32(self, v: u32) -> Result<Self::Ok, Self::Error> {
        self.serialize_i64(v as i64)
    }

    fn serialize_u64(self, v: u64) -> Result<Self::Ok, Self::Error> {
        let integer = i64::try_from(v).map_err(|e| Self::Error::custom(e))?;
        Formatter::new_with_defaults(RespTypeRef::Integer(integer)).write(&mut self.output)?;
        Ok(())
    }

    fn serialize_f32(self, v: f32) -> Result<Self::Ok, Self::Error> {
        Formatter::new_with_defaults(RespTypeRef::SimpleString(v.to_string().as_bytes()))
            .write(&mut self.output)?;
        Ok(())
    }

    fn serialize_f64(self, v: f64) -> Result<Self::Ok, Self::Error> {
        Formatter::new_with_defaults(RespTypeRef::SimpleString(v.to_string().as_bytes()))
            .write(&mut self.output)?;
        Ok(())
    }

    fn serialize_char(self, v: char) -> Result<Self::Ok, Self::Error> {
        let mut buffer = [0; 4];
        Formatter::new_with_defaults(RespTypeRef::SimpleString(
            v.encode_utf8(&mut buffer).as_bytes(),
        ))
        .write(&mut self.output)?;
        Ok(())
    }

    fn serialize_str(self, v: &str) -> Result<Self::Ok, Self::Error> {
        Formatter::new_with_defaults(RespTypeRef::BulkString(v.as_bytes()))
            .write(&mut self.output)?;
        Ok(())
    }

    fn serialize_bytes(self, v: &[u8]) -> Result<Self::Ok, Self::Error> {
        Formatter::new_with_defaults(RespTypeRef::BulkString(v)).write(&mut self.output)?;
        Ok(())
    }

    fn serialize_none(self) -> Result<Self::Ok, Self::Error> {
        self.serialize_unit()
    }

    fn serialize_some<T: ?Sized>(self, value: &T) -> Result<Self::Ok, Self::Error>
    where
        T: Serialize,
    {
        value.serialize(self)
    }

    fn serialize_unit(self) -> Result<Self::Ok, Self::Error> {
        Formatter::new_with_defaults(RespTypeRef::NullArray).write(&mut self.output)?;
        Ok(())
    }

    fn serialize_unit_struct(self, _name: &'static str) -> Result<Self::Ok, Self::Error> {
        self.serialize_unit()
    }

    fn serialize_unit_variant(
        self,
        name: &'static str,
        variant_index: u32,
        variant: &'static str,
    ) -> Result<Self::Ok, Self::Error> {
        Err(Self::Error::custom("unable to serializer value"))
    }

    fn serialize_newtype_struct<T: ?Sized>(
        self,
        name: &'static str,
        value: &T,
    ) -> Result<Self::Ok, Self::Error>
    where
        T: Serialize,
    {
        Err(Self::Error::custom("unable to serializer value"))
    }

    fn serialize_newtype_variant<T: ?Sized>(
        self,
        name: &'static str,
        variant_index: u32,
        variant: &'static str,
        value: &T,
    ) -> Result<Self::Ok, Self::Error>
    where
        T: Serialize,
    {
        Err(Self::Error::custom("unable to serializer value"))
    }

    fn serialize_seq(self, len: Option<usize>) -> Result<Self::SerializeSeq, Self::Error> {
        if let Some(len) = len {
            self.output.write_all(b"*")?;
            self.output.write_all(len.to_string().as_bytes())?;
            self.output.write_all(b"\r\n")?;
            Ok(self)
        } else {
            Err(Self::Error::custom("value is missing size"))
        }
    }

    fn serialize_tuple(self, len: usize) -> Result<Self::SerializeTuple, Self::Error> {
        self.serialize_seq(Some(len))
    }

    fn serialize_tuple_struct(
        self,
        name: &'static str,
        len: usize,
    ) -> Result<Self::SerializeTupleStruct, Self::Error> {
        self.serialize_seq(Some(len))
    }

    fn serialize_tuple_variant(
        self,
        name: &'static str,
        variant_index: u32,
        variant: &'static str,
        len: usize,
    ) -> Result<Self::SerializeTupleVariant, Self::Error> {
        Err(Self::Error::custom("unable to serializer value"))
    }

    fn serialize_map(self, len: Option<usize>) -> Result<Self::SerializeMap, Self::Error> {
        Err(Self::Error::custom("unable to serializer value"))
    }

    fn serialize_struct(
        self,
        name: &'static str,
        len: usize,
    ) -> Result<Self::SerializeStruct, Self::Error> {
        Err(Self::Error::custom("unable to serializer value"))
    }

    fn serialize_struct_variant(
        self,
        name: &'static str,
        variant_index: u32,
        variant: &'static str,
        len: usize,
    ) -> Result<Self::SerializeStructVariant, Self::Error> {
        Err(Self::Error::custom("unable to serializer value"))
    }
}

impl<'a> ser::SerializeSeq for &'a mut Serializer {
    type Ok = ();
    type Error = SerializerError;

    fn serialize_element<T>(&mut self, value: &T) -> SerializerResult<()>
    where
        T: ?Sized + Serialize,
    {
        value.serialize(&mut **self)
    }

    fn end(self) -> SerializerResult<()> {
        Ok(())
    }
}

// Same thing but for tuples.
impl<'a> ser::SerializeTuple for &'a mut Serializer {
    type Ok = ();
    type Error = SerializerError;

    fn serialize_element<T>(&mut self, value: &T) -> SerializerResult<()>
    where
        T: ?Sized + Serialize,
    {
        value.serialize(&mut **self)
    }

    fn end(self) -> SerializerResult<()> {
        Ok(())
    }
}

impl<'a> ser::SerializeTupleStruct for &'a mut Serializer {
    type Ok = ();
    type Error = SerializerError;

    fn serialize_field<T>(&mut self, value: &T) -> SerializerResult<()>
    where
        T: ?Sized + Serialize,
    {
        value.serialize(&mut **self)
    }

    fn end(self) -> SerializerResult<()> {
        Ok(())
    }
}

impl<'a> ser::SerializeTupleVariant for &'a mut Serializer {
    type Ok = ();
    type Error = SerializerError;

    fn serialize_field<T>(&mut self, value: &T) -> SerializerResult<()>
    where
        T: ?Sized + Serialize,
    {
        value.serialize(&mut **self)
    }

    fn end(self) -> SerializerResult<()> {
        Ok(())
    }
}

#[test]
fn serialize_string_test() {
    assert_eq!(
        to_bytes(&String::from("hello")).unwrap(),
        b"$5\r\nhello\r\n"
    )
}

#[test]
fn serialize_char_test() {
    assert_eq!(to_bytes(&'a').unwrap(), b"+a\r\n")
}

#[test]
fn serialize_integer_test() {
    assert_eq!(to_bytes(&12345).unwrap(), b":12345\r\n")
}

#[test]
fn serialize_list_integer_test() {
    assert_eq!(
        to_bytes(&[1, 2, 3, 4, 5]).unwrap(),
        b"*5\r\n:1\r\n:2\r\n:3\r\n:4\r\n:5\r\n"
    )
}
