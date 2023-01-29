/// In RESP, the first byte determines the data type:
/// For Simple Strings, the first byte of the reply is "+"
/// For Errors, the first byte of the reply is "-"
/// For Integers, the first byte of the reply is ":"
/// For Bulk Strings, the first byte of the reply is "$"
/// For Arrays, the first byte of the reply is "*"
///
///
///

type ResultXd = Result<RespType, RespError>;
type ResultXp<'a> = Result<RespTypeScan<'a>, RespError>;

#[derive(Debug, PartialEq)]
pub enum RespError {
    None,
    Other,
}

#[derive(Debug, PartialEq)]
pub enum RespType {
    SimpleString(Vec<u8>),
    Error(Vec<u8>),
    Integer(i64),
    BulkString(Vec<u8>),
    Array(Vec<RespType>),
    NullString,
    NullArray,
}

impl RespType {
    pub fn parse(input: &[u8]) -> ResultXd {
        if input.len() == 0 {
            return Err(RespError::Other);
        }

        match input[0] {
            b'+' => Ok(RespType::SimpleString(Self::parse_simple_string(
                &input[1..],
            )?)),
            b'-' => Ok(RespType::Error(Self::parse_simple_string(&input[1..])?)),
            b':' => Ok(RespType::Integer(Self::parse_int(&input[1..])?)),
            b'$' => match Self::parse_int(&input[1..])? {
                -1 => Ok(RespType::NullString),
                0 => Ok(RespType::BulkString(Vec::new())),
                x if x > 0 => {
                    let newline_index = input
                        .windows(2)
                        .position(|s| s == b"\r\n")
                        .ok_or_else(|| RespError::Other)?;
                    let rest = &input[(newline_index + 2)..];

                    Ok(RespType::BulkString(rest[0..x as usize].to_vec()))
                }
                _ => return Err(RespError::Other),
            },
            _ => return Err(RespError::Other),
        }
    }

    fn parse_int(input: &[u8]) -> Result<i64, RespError> {
        let data =
            String::from_utf8(Self::parse_simple_string(&input)?).map_err(|_| RespError::Other)?;
        let int = data.parse().map_err(|_| RespError::Other)?;
        Ok(int)
    }

    fn parse_simple_string(input: &[u8]) -> Result<Vec<u8>, RespError> {
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
            Ok(data)
        } else {
            Err(RespError::Other)
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum RespTypeScan<'a> {
    SimpleString(&'a [u8]),
    Error(&'a [u8]),
    Integer(i64),
    BulkString(&'a [u8]),
    NullString,
    NullArray,
    ArrayStart(usize),
}

impl<'a> RespTypeScan<'a> {
    pub fn iter(data: &'a [u8]) -> RespTypeIter<'a> {
        RespTypeIter::new(data)
    }

    fn get_line_range(input: &[u8], start_at: usize) -> Result<(usize, usize), RespError> {
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
            Err(RespError::Other)
        }
    }

    fn get_line_range_as_integer(
        input: &[u8],
        start_at: usize,
    ) -> Result<(usize, usize, i64), RespError> {
        let (start, end) = Self::get_line_range(input, start_at)?;
        let s = std::str::from_utf8(&input[start..end - 2]).map_err(|_| RespError::Other)?;
        let integer = s.parse().map_err(|_| RespError::Other)?;
        Ok((start, end, integer))
    }
}

#[derive(Debug)]
pub struct RespTypeIter<'a> {
    data: &'a [u8],
    closed: bool,
    scan_start: usize,
    length: usize,
}

impl<'a> RespTypeIter<'a> {
    pub fn new(data: &'a [u8]) -> RespTypeIter<'a> {
        RespTypeIter {
            data,
            closed: false,
            scan_start: 0,
            length: 1,
        }
    }

    fn inner_loop(&mut self) -> ResultXp<'a> {
        let out = match self.data[self.scan_start] {
            b'+' => {
                let (start, end) = RespTypeScan::get_line_range(self.data, self.scan_start + 1)?;
                self.scan_start = end + 1;
                RespTypeScan::SimpleString(&self.data[start..end - 2])
            }
            b'-' => {
                let (start, end) = RespTypeScan::get_line_range(self.data, self.scan_start + 1)?;
                self.scan_start = end + 1;
                RespTypeScan::Error(&self.data[start..end - 2])
            }
            b':' => {
                let (_start, end, integer) =
                    RespTypeScan::get_line_range_as_integer(self.data, self.scan_start + 1)?;
                self.scan_start = end;
                RespTypeScan::Integer(integer)
            }
            b'$' => {
                let (_start, end, integer) =
                    RespTypeScan::get_line_range_as_integer(self.data, self.scan_start + 1)?;
                self.scan_start = end;

                match integer {
                    -1 => RespTypeScan::NullString,
                    0 => {
                        self.scan_start += 2;
                        RespTypeScan::BulkString(b"")
                    }
                    x if x > 0 => {
                        let start = self.scan_start;
                        self.scan_start += x as usize;
                        let data = &self.data[start..self.scan_start];
                        self.scan_start += 2;
                        RespTypeScan::BulkString(data)
                    }
                    _ => return Err(RespError::Other),
                }
            }
            b'*' => {
                let (_start, end, integer) =
                    RespTypeScan::get_line_range_as_integer(self.data, self.scan_start + 1)?;
                self.scan_start = end;
                match integer {
                    -1 => RespTypeScan::NullArray,
                    0 => {
                        self.length = 0;
                        self.scan_start += 2;
                        return Err(RespError::None);
                    }
                    x if x > 0 => {
                        self.length = x as usize;
                        RespTypeScan::ArrayStart(self.length)
                    }
                    _ => return Err(RespError::Other),
                }
            }
            _ => return Err(RespError::Other),
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

        if self.length == 0 {
            return None;
        }

        let out = self.inner_loop();
        if let Ok(RespTypeScan::ArrayStart(_)) = out {
            ()
        } else {
            self.length = self.length.saturating_sub(1);
        }

        if Err(RespError::None) == out {
            return None;
        }

        Some(out)
    }
}

#[test]
fn parse_test_1() {
    assert_eq!(
        RespType::SimpleString(b"OK".to_vec()),
        RespType::parse(b"+OK\r\n").unwrap()
    );
    assert!(RespType::parse(b"+OK\r").is_err());
}

#[test]
fn parse_test_2() {
    assert_eq!(
        RespType::Error(b"ERROR".to_vec()),
        RespType::parse(b"-ERROR\r\n").unwrap()
    );
    assert!(RespType::parse(b"-ERROR\r").is_err());
}

#[test]
fn parse_test_3() {
    assert_eq!(
        RespType::Integer(1234),
        RespType::parse(b":1234\r\n").unwrap()
    );
    assert!(RespType::parse(b":1234\r").is_err());
}

#[test]
fn parse_test_4() {
    assert_eq!(
        RespType::BulkString(b"hello".to_vec()),
        RespType::parse(b"$5\r\nhello\r\n").unwrap()
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

#[test]
fn iter_test_4() {
    let mut a = RespTypeIter::new(b"$5\r\nhello\r\n");
    let item = a.next().unwrap().unwrap();

    assert_eq!(RespTypeScan::BulkString(b"hello"), item);
    assert_eq!(None, a.next());
}

#[test]
fn iter_test_5() {
    let mut a = RespTypeIter::new(b"$0\r\n\r\n");
    let item = a.next().unwrap().unwrap();

    assert_eq!(RespTypeScan::BulkString(b""), item);
    assert_eq!(None, a.next());
}

#[test]
fn iter_test_6() {
    let mut a = RespTypeIter::new(b"$-1\r\n");
    let item = a.next().unwrap().unwrap();

    assert_eq!(RespTypeScan::NullString, item);
    assert_eq!(None, a.next());
}

#[test]
fn iter_test_7() {
    let mut a = RespTypeIter::new(b"*3\r\n:1\r\n:2\r\n:3\r\n");
    let item = a.next().unwrap().unwrap();
    assert_eq!(RespTypeScan::ArrayStart(3), item);
    let item = a.next().unwrap().unwrap();
    assert_eq!(RespTypeScan::Integer(1), item);
    let item = a.next().unwrap().unwrap();
    assert_eq!(RespTypeScan::Integer(2), item);
    let item = a.next().unwrap().unwrap();
    assert_eq!(RespTypeScan::Integer(3), item);
    assert_eq!(None, a.next());
}

#[test]
fn iter_test_8() {
    let mut a = RespTypeIter::new(b"*0\r\n");
    assert_eq!(None, a.next());
}

#[test]
fn iter_test_9() {
    let mut a = RespTypeIter::new(b"*-1\r\n");
    let item = a.next().unwrap().unwrap();
    assert_eq!(RespTypeScan::NullArray, item);
}
