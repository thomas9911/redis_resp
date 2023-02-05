use std::io::Write;

use crate::RespTypeRef;

pub struct Formatter<'a> {
    item: RespTypeRef<'a>,
}

impl<'a> Formatter<'a> {
    pub fn new_with_defaults(item: RespTypeRef<'a>) -> Formatter<'a> {
        Formatter { item }
    }

    pub fn write<W: Write>(&self, output: &mut W) -> std::io::Result<()> {
        self.inner_write(output, &self.item)
    }

    fn inner_write<W: Write>(&self, output: &mut W, item: &RespTypeRef<'a>) -> std::io::Result<()> {
        use RespTypeRef::*;

        match item {
            SimpleString(data) => {
                output.write_all(b"+")?;
                output.write_all(data)?;
                output.write_all(b"\r\n")?;
            }
            Error(data) => {
                output.write_all(b"-")?;
                output.write_all(data)?;
                output.write_all(b"\r\n")?;
            }
            Integer(data) => {
                output.write_all(b":")?;
                output.write_all(data.to_string().as_bytes())?;
                output.write_all(b"\r\n")?;
            }
            BulkString(data) => {
                output.write_all(b"$")?;
                output.write_all(data.len().to_string().as_bytes())?;
                output.write_all(b"\r\n")?;
                output.write_all(data)?;
                output.write_all(b"\r\n")?;
            }
            NullString => output.write_all(b"$-1\r\n")?,
            Array(data) => {
                output.write_all(b"*")?;
                output.write_all(data.len().to_string().as_bytes())?;
                output.write_all(b"\r\n")?;
                for array_item in data {
                    self.inner_write(output, array_item)?;
                }
            }
            NullArray => output.write_all(b"*-1\r\n")?,
        };

        Ok(())
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
