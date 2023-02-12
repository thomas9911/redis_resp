use crate::lexer::{Token, TokenType};
use crate::resp_type::RespTypeRefType;
use crate::Lexer;
use crate::{ParseError, RespErrorType, RespTypeRef};

use std::iter::Peekable;

pub struct Parser<'a> {
    lexer: Peekable<Lexer<'a>>,
}

impl<'a> Parser<'a> {
    pub fn new_from_bytes(data: &'a [u8]) -> Parser<'a> {
        Parser::new(Lexer::new(data))
    }

    pub fn new(lexer: Lexer<'a>) -> Parser<'a> {
        Parser {
            lexer: lexer.peekable(),
        }
    }

    pub fn peek_known(&mut self) -> Option<RespTypeRefType> {
        self.lexer
            .peek()
            .and_then(|token| token.tokentype.as_known_type())
    }

    pub fn parse(&mut self) -> Result<RespTypeRef<'a>, ParseError<'a>> {
        match self.lexer.next() {
            Some(token) if token.tokentype == TokenType::SimpleStringStart => {
                return self.parse_simple_string()
            }
            Some(token) if token.tokentype == TokenType::ErrorStart => return self.parse_error(),
            Some(token) if token.tokentype == TokenType::IntegerStart => {
                return self.parse_integer()
            }
            Some(token) if token.tokentype == TokenType::BulkStringStart => {
                return self.parse_bulk_string()
            }
            Some(token) if token.tokentype == TokenType::ArrayStart => return self.parse_array(),
            Some(token) => Err(ParseError {
                error_type: RespErrorType::InvalidStart,
                token: Some(token),
            }),
            None => Err(ParseError {
                error_type: RespErrorType::InvalidStart,
                token: None,
            }),
        }
    }

    fn parse_simple_string(&mut self) -> Result<RespTypeRef<'a>, ParseError<'a>> {
        match self.lexer.next() {
            Some(Token {
                tokentype: TokenType::SimpleString,
                data,
                ..
            }) => {
                self.check_newline()?;
                return Ok(RespTypeRef::SimpleString(data));
            }
            Some(token) => Err(ParseError {
                error_type: RespErrorType::InvalidData,
                token: Some(token),
            }),
            None => Err(ParseError {
                error_type: RespErrorType::InvalidData,
                token: None,
            }),
        }
    }

    fn parse_error(&mut self) -> Result<RespTypeRef<'a>, ParseError<'a>> {
        match self.lexer.next() {
            Some(Token {
                tokentype: TokenType::Error,
                data,
                ..
            }) => {
                self.check_newline()?;
                return Ok(RespTypeRef::Error(data));
            }
            Some(token) => Err(ParseError {
                error_type: RespErrorType::InvalidData,
                token: Some(token),
            }),
            None => Err(ParseError {
                error_type: RespErrorType::InvalidData,
                token: None,
            }),
        }
    }

    fn parse_integer(&mut self) -> Result<RespTypeRef<'a>, ParseError<'a>> {
        match self.lexer.next() {
            Some(token) if token.tokentype == TokenType::Integer => {
                self.check_newline()?;

                let integer = Self::_parse_integer_bytes(token.data).map_err(|_| ParseError {
                    error_type: RespErrorType::InvalidInteger,
                    token: Some(token),
                })?;

                return Ok(RespTypeRef::Integer(integer));
            }
            Some(token) => Err(ParseError {
                error_type: RespErrorType::InvalidData,
                token: Some(token),
            }),
            None => Err(ParseError {
                error_type: RespErrorType::InvalidData,
                token: None,
            }),
        }
    }

    fn _parse_integer_bytes(data: &[u8]) -> Result<i64, Box<dyn std::error::Error>> {
        let str_data = std::str::from_utf8(data)?;
        let int = str_data.parse()?;
        Ok(int)
    }

    fn parse_bulk_string(&mut self) -> Result<RespTypeRef<'a>, ParseError<'a>> {
        let size = self.parse_bulk_string_size()?;
        if size == -1 {
            return Ok(RespTypeRef::NullString);
        }

        match self.lexer.next() {
            Some(token) if token.tokentype == TokenType::BulkString => {
                self.check_newline()?;

                if token.data.len() != size.try_into().unwrap() {
                    return Err(ParseError {
                        error_type: RespErrorType::InvalidSize,
                        token: Some(token),
                    });
                }

                return Ok(RespTypeRef::BulkString(token.data));
            }
            Some(token) if token.tokentype == TokenType::Newline && size == 0 => {
                return Ok(RespTypeRef::BulkString(b""));
            }
            Some(token) => Err(ParseError {
                error_type: RespErrorType::InvalidData,
                token: Some(token),
            }),
            None => Err(ParseError {
                error_type: RespErrorType::InvalidData,
                token: None,
            }),
        }
    }

    fn parse_bulk_string_size(&mut self) -> Result<i64, ParseError<'a>> {
        self._parse_size(TokenType::BulkStringSize)
    }

    fn parse_array(&mut self) -> Result<RespTypeRef<'a>, ParseError<'a>> {
        let size = self.parse_array_size()?;
        if size == -1 {
            return Ok(RespTypeRef::NullArray);
        }
        let mut array: Vec<_> = Vec::new();

        for _ in 0..size {
            let item = self.parse()?;
            array.push(item)
        }

        Ok(RespTypeRef::Array(array))
    }

    fn parse_array_size(&mut self) -> Result<i64, ParseError<'a>> {
        self._parse_size(TokenType::ArraySize)
    }

    fn _parse_size(&mut self, token_type: TokenType) -> Result<i64, ParseError<'a>> {
        match self.lexer.next() {
            Some(token) if token.tokentype == token_type => {
                self.check_newline()?;

                let size = Self::_parse_integer_bytes(token.data).map_err(|_| ParseError {
                    error_type: RespErrorType::InvalidInteger,
                    token: Some(token.clone()),
                })?;

                if size < -1 {
                    return Err(ParseError {
                        error_type: RespErrorType::InvalidSize,
                        token: Some(token.clone()),
                    });
                }

                Ok(size)
            }
            Some(token) => Err(ParseError {
                error_type: RespErrorType::InvalidData,
                token: Some(token),
            }),
            None => Err(ParseError {
                error_type: RespErrorType::InvalidData,
                token: None,
            }),
        }
    }

    fn check_newline(&mut self) -> Result<(), ParseError<'a>> {
        match self.lexer.next() {
            Some(Token {
                tokentype: TokenType::Newline,
                ..
            }) => Ok(()),
            Some(token) => Err(ParseError {
                error_type: RespErrorType::NewLineMissing,
                token: Some(token),
            }),
            None => Err(ParseError {
                error_type: RespErrorType::NewLineMissing,
                token: None,
            }),
        }
    }
}

#[test]
fn parse_test_1() {
    let lexer = Lexer::new(b"+OK\r\n");
    let mut parser = Parser::new(lexer);

    assert_eq!(RespTypeRef::SimpleString(b"OK"), parser.parse().unwrap())
}

#[test]
fn lexer_test_2() {
    let mut parser = Parser::new_from_bytes(b"-ERROR\r\n");
    assert_eq!(RespTypeRef::Error(b"ERROR"), parser.parse().unwrap())
}

#[test]
fn lexer_test_3() {
    let mut parser = Parser::new_from_bytes(b":1234\r\n");
    assert_eq!(RespTypeRef::Integer(1234), parser.parse().unwrap())
}

#[test]
fn lexer_test_4() {
    let mut parser = Parser::new_from_bytes(b"$5\r\nhello\r\n");
    assert_eq!(RespTypeRef::BulkString(b"hello"), parser.parse().unwrap())
}

#[test]
fn lexer_test_5() {
    let mut parser = Parser::new_from_bytes(b"$0\r\n\r\n");
    assert_eq!(RespTypeRef::BulkString(b""), parser.parse().unwrap())
}

#[test]
fn lexer_test_6() {
    let mut parser = Parser::new_from_bytes(b"$-1\r\n");
    assert_eq!(RespTypeRef::NullString, parser.parse().unwrap())
}

#[test]
fn parser_test_7() {
    let mut parser = Parser::new_from_bytes(b"*3\r\n:1\r\n:2\r\n:3\r\n");
    assert_eq!(
        RespTypeRef::Array(vec![
            RespTypeRef::Integer(1),
            RespTypeRef::Integer(2),
            RespTypeRef::Integer(3)
        ]),
        parser.parse().unwrap()
    )
}

#[test]
fn parse_test_8() {
    let mut parser = Parser::new_from_bytes(b"*0\r\n");

    assert_eq!(RespTypeRef::Array(vec![]), parser.parse().unwrap())
}

#[test]
fn parse_test_9() {
    let mut parser = Parser::new_from_bytes(b"*-1\r\n");

    assert_eq!(RespTypeRef::NullArray, parser.parse().unwrap())
}
