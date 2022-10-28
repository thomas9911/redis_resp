use std::path::Iter;

/// In RESP, the first byte determines the data type:
/// For Simple Strings, the first byte of the reply is "+"
/// For Errors, the first byte of the reply is "-"
/// For Integers, the first byte of the reply is ":"
/// For Bulk Strings, the first byte of the reply is "$"
/// For Arrays, the first byte of the reply is "*"
///
///
///

type ResultXd = Result<RespType, ()>;
type ResultXp<'a> = Result<RespTypeScan<'a>, ()>;

#[derive(Debug, PartialEq)]
pub enum RespType {
    SimpleString(Vec<u8>),
    Error(Vec<u8>),
    Integer(i64),
    BulkString(Vec<u8>),
    Array(Vec<RespType>),
}

impl RespType {
    pub fn parse(input: &[u8]) -> ResultXd {
        if input.len() == 0 {
            return Err(());
        }

        match input[0] {
            b'+' => parse_simple_string(&input[1..]),
            _ => return Err(()),
        }
    }
}

fn parse_simple_string(input: &[u8]) -> ResultXd {
    let mut iter = input.iter().peekable();
    let mut data = Vec::with_capacity(input.len());
    let mut closed = false;

    while let Some(a) = iter.next() {
        if a == &b'\r' && iter.peek() == Some(&&b'\n') {
            closed = true;
            break;
        }

        data.push(*a)
    }

    if closed {
        Ok(RespType::SimpleString(data))
    } else {
        Err(())
    }
}

#[derive(Debug, PartialEq)]
pub enum RespTypeScan<'a> {
    SimpleString(&'a [u8]),
    Error(&'a [u8]),
    Integer(i64),
    BulkString(&'a [u8]),
}

impl<'a> RespTypeScan<'a> {
    pub fn iter(data: &'a [u8]) -> RespTypeIter<'a> {
        RespTypeIter::new(data)
    }

    fn get_line_range(input: &[u8], start_at: usize) -> Result<(usize, usize), ()> {
        let mut iter = input[start_at..].iter().peekable();
        let mut closed = false;
        let mut end = start_at;

        while let Some(a) = iter.next() {
            if a == &b'\r' && iter.peek() == Some(&&b'\n') {
                closed = true;
                end += 2;
                break;
            }

            end += 1;
        }

        if closed {
            Ok((start_at, end))
        } else {
            Err(())
        }
    }
}

pub struct RespTypeIter<'a> {
    data: &'a [u8],
    closed: bool,
    scan_start: usize,
    is_list: bool,
}

impl<'a> RespTypeIter<'a> {
    pub fn new(data: &'a [u8]) -> RespTypeIter<'a> {
        RespTypeIter {
            data,
            closed: false,
            scan_start: 0,
            is_list: false,
        }
    }

    fn inner_loop(&mut self) -> ResultXp<'a> {
        let out = match self.data[self.scan_start] {
            b'+' => {
                let (start, end) = RespTypeScan::get_line_range(self.data, self.scan_start + 1)?;
                self.scan_start = end + 1;
                self.closed = true;
                RespTypeScan::SimpleString(&self.data[start..end - 2])
            }
            b'-' => {
                let (start, end) = RespTypeScan::get_line_range(self.data, self.scan_start + 1)?;
                self.scan_start = end + 1;
                self.closed = true;
                RespTypeScan::Error(&self.data[start..end - 2])
            }
            b':' => {
                let (start, end) = RespTypeScan::get_line_range(self.data, self.scan_start + 1)?;
                self.scan_start = end + 1;
                self.closed = true;
                let s = std::str::from_utf8(&self.data[start..end - 2]).map_err(|_| ())?;
                RespTypeScan::Integer(s.parse().map_err(|_| ())?)
            }
            // b'$' => (),
            // b'*' => (),
            _ => return Err(()),
        };

        Ok(out)
    }
}

impl<'a> Iterator for RespTypeIter<'a> {
    type Item = ResultXp<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.closed {
            return None;
        }

        if self.data.get(self.scan_start).is_none() {
            return None;
        }

        if self.scan_start == 0 {
            self.is_list = self.data[0] == b'*'
        }

        Some(self.inner_loop())
    }
}

#[test]
fn parse_simple_string_test() {
    assert_eq!(
        RespType::SimpleString(b"OK".to_vec()),
        RespType::parse(b"+OK\r\n").unwrap()
    );
    assert!(RespType::parse(b"+OK\r").is_err());
}

#[test]
fn iter_test_1() {
    let mut a = RespTypeIter::new(b"+OK\r\n");
    let item = a.next().unwrap().unwrap();

    assert_eq!(RespTypeScan::SimpleString(b"OK"), item);
    assert_eq!(None, a.next());
}

#[test]
fn iter_test_2() {
    let mut a = RespTypeIter::new(b"-ERROR\r\n");
    let item = a.next().unwrap().unwrap();

    assert_eq!(RespTypeScan::Error(b"ERROR"), item);
    assert_eq!(None, a.next());
}

#[test]
fn iter_test_3() {
    let mut a = RespTypeIter::new(b":1234\r\n");
    let item = a.next().unwrap().unwrap();

    assert_eq!(RespTypeScan::Integer(1234), item);
    assert_eq!(None, a.next());
}
