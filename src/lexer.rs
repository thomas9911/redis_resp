use crate::resp_type::RespTypeRefType;
use memchr::memmem;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TokenType {
    SimpleStringStart,
    SimpleString,
    ErrorStart,
    Error,
    IntegerStart,
    Integer,
    BulkStringStart,
    BulkStringSize,
    BulkString,
    ArrayStart,
    ArraySize,
    Newline,
}

impl TokenType {
    pub fn as_known_type(&self) -> Option<RespTypeRefType> {
        use TokenType::*;

        match self {
            SimpleStringStart => Some(RespTypeRefType::SimpleString),
            SimpleString => None,
            ErrorStart => Some(RespTypeRefType::Error),
            Error => None,
            IntegerStart => Some(RespTypeRefType::Integer),
            Integer => None,
            BulkStringStart => Some(RespTypeRefType::BulkString),
            BulkStringSize => None,
            BulkString => None,
            ArrayStart => Some(RespTypeRefType::Array),
            ArraySize => None,
            Newline => None,
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct Token<'a> {
    pub start: usize,
    pub end: usize,
    pub data: &'a [u8],
    pub tokentype: TokenType,
}

fn find_newline(input: &[u8]) -> Option<usize> {
    memmem::find(input, b"\r\n")
}

impl<'a> Token<'a> {
    fn take(input: &'a [u8], previous: &Option<TokenType>) -> (usize, Option<TokenType>) {
        use TokenType::*;

        if input.len() == 0 {
            return (0, None);
        };

        if input.len() >= 2 {
            if &input[0..=1] == b"\r\n" {
                return (2, Some(Newline));
            }
        };

        match input[0] {
            _ if previous == &Some(TokenType::SimpleStringStart) => {
                if let Some(found) = find_newline(input) {
                    (found, Some(SimpleString))
                } else {
                    (0, None)
                }
            }
            _ if previous == &Some(TokenType::ErrorStart) => {
                if let Some(found) = find_newline(input) {
                    (found, Some(Error))
                } else {
                    (0, None)
                }
            }
            _ if previous == &Some(TokenType::IntegerStart) => {
                if let Some(found) = find_newline(input) {
                    (found, Some(Integer))
                } else {
                    (0, None)
                }
            }
            x if previous == &Some(TokenType::BulkStringStart)
                && ((b'0'..b'9').contains(&x) || x == b'-') =>
            {
                if let Some(found) = find_newline(input) {
                    (found, Some(BulkStringSize))
                } else {
                    (0, None)
                }
            }
            x if previous == &Some(TokenType::ArrayStart)
                && ((b'0'..b'9').contains(&x) || x == b'-') =>
            {
                if let Some(found) = find_newline(input) {
                    (found, Some(ArraySize))
                } else {
                    (0, None)
                }
            }
            _ if previous == &Some(TokenType::BulkStringSize) => {
                if let Some(found) = find_newline(input) {
                    (found, Some(BulkString))
                } else {
                    (0, None)
                }
            }
            b'+' => (1, Some(SimpleStringStart)),
            b'-' => (1, Some(ErrorStart)),
            b':' => (1, Some(IntegerStart)),
            b'$' => (1, Some(BulkStringStart)),
            b'*' => (1, Some(ArrayStart)),
            _ => (0, None),
        }
    }
}

pub struct Lexer<'a> {
    data: &'a [u8],
    start: usize,
    previous: Option<TokenType>,
}

impl<'a> Lexer<'a> {
    pub fn new(data: &'a [u8]) -> Self {
        Lexer {
            data,
            start: 0,
            previous: None,
        }
    }
}

impl<'a> Iterator for Lexer<'a> {
    type Item = Token<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        match Token::take(&self.data[self.start..], &self.previous) {
            (_, None) => None,
            (length, Some(tokentype)) => {
                let end = self.start + length;
                let data = &self.data[self.start..end];
                let token = Token {
                    start: self.start,
                    end,
                    data,
                    tokentype,
                };

                self.start = end;
                if tokentype != TokenType::Newline {
                    self.previous = Some(tokentype);
                }

                Some(token)
            }
        }
    }
}

#[test]
fn lexer_test_1() {
    let tokenizer = Lexer::new(b"+OK\r\n");
    let tokens: Vec<_> = tokenizer.collect();

    assert_eq!(
        tokens,
        vec![
            Token {
                start: 0,
                end: 1,
                data: b"+",
                tokentype: TokenType::SimpleStringStart
            },
            Token {
                start: 1,
                end: 3,
                data: b"OK",
                tokentype: TokenType::SimpleString
            },
            Token {
                start: 3,
                end: 5,
                data: b"\r\n",
                tokentype: TokenType::Newline
            }
        ]
    );
}

#[test]
fn lexer_test_2() {
    let tokenizer = Lexer::new(b"-ERROR\r\n");
    let tokens: Vec<_> = tokenizer.collect();

    assert_eq!(
        tokens,
        vec![
            Token {
                start: 0,
                end: 1,
                data: b"-",
                tokentype: TokenType::ErrorStart
            },
            Token {
                start: 1,
                end: 6,
                data: b"ERROR",
                tokentype: TokenType::Error
            },
            Token {
                start: 6,
                end: 8,
                data: b"\r\n",
                tokentype: TokenType::Newline
            }
        ]
    );
}

#[test]
fn lexer_test_3() {
    let tokenizer = Lexer::new(b":1234\r\n");
    let tokens: Vec<_> = tokenizer.collect();

    assert_eq!(
        tokens,
        vec![
            Token {
                start: 0,
                end: 1,
                data: b":",
                tokentype: TokenType::IntegerStart
            },
            Token {
                start: 1,
                end: 5,
                data: b"1234",
                tokentype: TokenType::Integer
            },
            Token {
                start: 5,
                end: 7,
                data: b"\r\n",
                tokentype: TokenType::Newline
            }
        ]
    );
}

#[test]
fn lexer_test_4() {
    let tokenizer = Lexer::new(b"$5\r\nhello\r\n");
    let tokens: Vec<_> = tokenizer.collect();

    assert_eq!(
        tokens,
        vec![
            Token {
                start: 0,
                end: 1,
                data: b"$",
                tokentype: TokenType::BulkStringStart
            },
            Token {
                start: 1,
                end: 2,
                data: b"5",
                tokentype: TokenType::BulkStringSize
            },
            Token {
                start: 2,
                end: 4,
                data: b"\r\n",
                tokentype: TokenType::Newline
            },
            Token {
                start: 4,
                end: 9,
                data: b"hello",
                tokentype: TokenType::BulkString
            },
            Token {
                start: 9,
                end: 11,
                data: b"\r\n",
                tokentype: TokenType::Newline
            }
        ]
    );
}

#[test]
fn lexer_test_5() {
    let tokenizer = Lexer::new(b"$0\r\n\r\n");
    let tokens: Vec<_> = tokenizer.collect();

    assert_eq!(
        tokens,
        vec![
            Token {
                start: 0,
                end: 1,
                data: b"$",
                tokentype: TokenType::BulkStringStart
            },
            Token {
                start: 1,
                end: 2,
                data: b"0",
                tokentype: TokenType::BulkStringSize
            },
            Token {
                start: 2,
                end: 4,
                data: b"\r\n",
                tokentype: TokenType::Newline
            },
            Token {
                start: 4,
                end: 6,
                data: b"\r\n",
                tokentype: TokenType::Newline
            }
        ]
    );
}

#[test]
fn lexer_test_6() {
    let tokenizer = Lexer::new(b"$-1\r\n");
    let tokens: Vec<_> = tokenizer.collect();

    assert_eq!(
        tokens,
        vec![
            Token {
                start: 0,
                end: 1,
                data: b"$",
                tokentype: TokenType::BulkStringStart
            },
            Token {
                start: 1,
                end: 3,
                data: b"-1",
                tokentype: TokenType::BulkStringSize
            },
            Token {
                start: 3,
                end: 5,
                data: b"\r\n",
                tokentype: TokenType::Newline
            }
        ]
    );
}

#[test]
fn lexer_test_7() {
    let tokenizer = Lexer::new(b"*3\r\n:1\r\n:2\r\n:3\r\n");
    let tokens: Vec<_> = tokenizer.collect();

    assert_eq!(
        tokens,
        vec![
            Token {
                start: 0,
                end: 1,
                data: b"*",
                tokentype: TokenType::ArrayStart
            },
            Token {
                start: 1,
                end: 2,
                data: b"3",
                tokentype: TokenType::ArraySize
            },
            Token {
                start: 2,
                end: 4,
                data: b"\r\n",
                tokentype: TokenType::Newline
            },
            Token {
                start: 4,
                end: 5,
                data: b":",
                tokentype: TokenType::IntegerStart
            },
            Token {
                start: 5,
                end: 6,
                data: b"1",
                tokentype: TokenType::Integer
            },
            Token {
                start: 6,
                end: 8,
                data: b"\r\n",
                tokentype: TokenType::Newline
            },
            Token {
                start: 8,
                end: 9,
                data: b":",
                tokentype: TokenType::IntegerStart
            },
            Token {
                start: 9,
                end: 10,
                data: b"2",
                tokentype: TokenType::Integer
            },
            Token {
                start: 10,
                end: 12,
                data: b"\r\n",
                tokentype: TokenType::Newline
            },
            Token {
                start: 12,
                end: 13,
                data: b":",
                tokentype: TokenType::IntegerStart
            },
            Token {
                start: 13,
                end: 14,
                data: b"3",
                tokentype: TokenType::Integer
            },
            Token {
                start: 14,
                end: 16,
                data: b"\r\n",
                tokentype: TokenType::Newline
            },
        ]
    );
}

#[test]
fn lexer_test_8() {
    let tokenizer = Lexer::new(b"*0\r\n");
    let tokens: Vec<_> = tokenizer.collect();

    assert_eq!(
        tokens,
        vec![
            Token {
                start: 0,
                end: 1,
                data: b"*",
                tokentype: TokenType::ArrayStart
            },
            Token {
                start: 1,
                end: 2,
                data: b"0",
                tokentype: TokenType::ArraySize
            },
            Token {
                start: 2,
                end: 4,
                data: b"\r\n",
                tokentype: TokenType::Newline
            }
        ]
    );
}

#[test]
fn lexer_test_9() {
    let tokenizer = Lexer::new(b"*-1\r\n");
    let tokens: Vec<_> = tokenizer.collect();

    assert_eq!(
        tokens,
        vec![
            Token {
                start: 0,
                end: 1,
                data: b"*",
                tokentype: TokenType::ArrayStart
            },
            Token {
                start: 1,
                end: 3,
                data: b"-1",
                tokentype: TokenType::ArraySize
            },
            Token {
                start: 3,
                end: 5,
                data: b"\r\n",
                tokentype: TokenType::Newline
            }
        ]
    );
}
